pub mod cgb_palette;
pub mod rgb;

use std::{collections::VecDeque, hint::unreachable_unchecked};

use self::rgb::Rgb;
use super::interrupts::{InterruptFlag, Interrupts};
use crate::{bus::CgbCompatibility, common::D2Array, dma::hdma::Hdma};

#[cfg(feature = "serde")]
use {
    serde::{Deserialize, Serialize},
    serde_big_array::BigArray,
};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct Ppu {
    pub gpu_vram: D2Array<u8, 0x2000, 2>,
    pub gpu_vram_bank: u8,

    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    pub sprite_table: [u8; 0xA0],
    pub sprite_palette: [[usize; 4]; 2],

    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    pub frame_buffer: [Rgb; 160 * 144],
    draw_flag: bool,

    bg_palette: [usize; 4],

    mode: PpuMode,
    stat_irq_state: bool,
    window_internal_line_counter: u8,
    console_compatibility_mode: CgbCompatibility,
    obj_prio_mode: ObjectPriorityMode,

    ly_153_early: bool,
    stat_change_offset: u64,
    handle_lcd_powered_off: bool,

    pub hdma: Hdma,

    // io registers
    pub lcdc: u8, // FF40
    pub stat: u8, // FF41
    pub scy: u8,  // FF42
    pub scx: u8,  // FF43
    pub ly: u8,   // FF44
    pub lyc: u8,  // FF45
    pub dma: u8,  // FF46
    pub bgp: u8,  // FF47
    pub obp0: u8, // FF48
    pub obp1: u8, // FF49
    pub wy: u8,   // FF4A
    pub wx: u8,   // FF4B

    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    bg_color_palette_ram: [u8; 64],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    bg_color_palette: [[Rgb; 4]; 8],
    bg_color_palette_specification: u8, // FF68
    bg_color_palette_index: usize,
    bg_color_palette_auto_increment: bool,

    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    sprite_color_palette_ram: [u8; 64],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    sprite_color_palette: [[Rgb; 4]; 8],
    sprite_color_palette_specification: u8, // FF6A
    sprite_color_palette_index: usize,
    sprite_color_palette_auto_increment: bool,

    line_clock_cycles: u64,
    mode_clock_cycles: u64,

    // fifo
    scanned_sprites: VecDeque<SpriteInfo>,
    scanned_sprites_peek: Option<SpriteInfo>,
}

#[derive(Clone, Copy)]
pub enum LcdControlFlag {
    // 1: on, 0: off
    // TODO: this apparently controls the window? because the window relys on the bg_fetcher
    BGEnable = 0b0000_0001,

    // Display sprites (obj):
    // 1: on
    // 0: off
    OBJEnable = 0b0000_0010,

    // Sprite Size:
    // 0: 8x8
    // 1: 8x16
    OBJSize = 0b0000_0100,

    // Where the BG tiles are mapped: 0: 0x9800-0x9BFF, 1: 0x9C00-0x9FFF
    BGTileMapAddress = 0b0000_1000,

    // Location of Tiles for BG and window:
    // 0: 0x8800-0x97FF
    // 1: 0x8000-0x87FF (Same location as the sprites (OBJ) (They overlap))
    BGAndWindowTileData = 0b0001_0000,

    // Render window as part of the draw
    // 0: off
    // 1: on
    WindowEnable = 0b0010_0000,

    // where window tiles are mapped
    // 0: 0x9800-0x9BFF
    // 1: 0x9C00-0x9FFF
    WindowTileMapAddress = 0b0100_0000,

    LCDDisplayEnable = 0b1000_0000,
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(clippy::upper_case_acronyms)]
pub enum PpuMode {
    HBlank = 0, // mode 0
    VBlank = 1, // mode 1
    OAM = 2,    // mode 2
    VRAM = 3,   // mode 3
}

impl From<u8> for PpuMode {
    fn from(val: u8) -> Self {
        match val {
            0 => PpuMode::HBlank,
            1 => PpuMode::VBlank,
            2 => PpuMode::OAM,
            3 => PpuMode::VRAM,

            _ => panic!("Invalid ppu mode value"),
        }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ObjectPriorityMode {
    OamOrder,
    CoordinateOrder,
}

struct BGMapFlags {
    bg_oam_prio: bool, // true=use bg bit, false=use oam bit
    vertical_flip: bool,
    horizontal_flip: bool,
    tile_bank: usize,
    bg_palette_number: usize,
}

impl From<u8> for BGMapFlags {
    fn from(val: u8) -> Self {
        BGMapFlags {
            bg_oam_prio: val & 0b1000_0000 != 0,
            vertical_flip: val & 0b0100_0000 != 0,
            horizontal_flip: val & 0b0010_0000 != 0,
            tile_bank: ((val & 0b000_1000) >> 3) as usize,
            bg_palette_number: (val & 0b0000_0111) as usize,
        }
    }
}

struct TileData {
    color_index_row: [u8; 8],
    flags: BGMapFlags,
}

#[derive(Clone, Copy)]
struct ScanLinePxInfo {
    color_index: usize,
    bg_prio_set: bool,
}

impl ScanLinePxInfo {
    pub fn new(color_index: usize, bg_prio_set: bool) -> Self {
        ScanLinePxInfo {
            color_index,
            bg_prio_set,
        }
    }
}

impl Default for ScanLinePxInfo {
    fn default() -> Self {
        Self {
            color_index: 255,
            bg_prio_set: false,
        }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct SpriteInfo {
    y: i32,
    x: i32,
    tile_num: u16,
    flags: u8,
}

impl From<(&[u8], i32)> for SpriteInfo {
    fn from((data, sprite_size): (&[u8], i32)) -> Self {
        Self {
            y: data[0] as u16 as i32 - 16,
            x: data[1] as u16 as i32 - 8,
            tile_num: (data[2] & (if sprite_size == 16 { 0xFE } else { 0xFF })) as u16,
            flags: data[3],
        }
    }
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            gpu_vram: [[0; 0x2000]; 2].into(),
            gpu_vram_bank: 0b1111_1110,

            sprite_table: [0; 0xA0],
            sprite_palette: [[0, 1, 2, 3], [0, 1, 2, 3]],

            frame_buffer: [Rgb::default(); 160 * 144],
            draw_flag: false,

            bg_palette: [0, 1, 2, 3],

            mode: PpuMode::OAM,
            stat_irq_state: false,
            window_internal_line_counter: 0,
            console_compatibility_mode: CgbCompatibility::CgbOnly,
            obj_prio_mode: ObjectPriorityMode::OamOrder, // TODO: what is the default?

            ly_153_early: false,
            stat_change_offset: 0,
            handle_lcd_powered_off: true,

            hdma: Hdma::default(),

            lcdc: 0x0,
            stat: 0x0,
            scy: 0x0,
            scx: 0x0,
            ly: 0x0,
            lyc: 0x0,
            dma: 0x0,
            bgp: 0x0,
            obp0: 0x0,
            obp1: 0x0,
            wy: 0x0,
            wx: 0x0,

            bg_color_palette_ram: [0xFF; 64],
            bg_color_palette: [[Rgb::default(); 4]; 8],
            bg_color_palette_specification: 0xFF,
            bg_color_palette_index: 0,
            bg_color_palette_auto_increment: false,

            // TODO: confirm below is conformed to
            // "Note that while 4 colors are stored per OBJ palette, color #0 is never used,
            // as it’s always transparent. It’s thus fine to write garbage values,
            // or even leave color #0 uninitialized." - Pandocs
            sprite_color_palette_ram: [0xFF; 64],
            sprite_color_palette: [[Rgb::default(); 4]; 8],
            sprite_color_palette_specification: 0xFF,
            sprite_color_palette_index: 0,
            sprite_color_palette_auto_increment: false,

            line_clock_cycles: 0,
            mode_clock_cycles: 0,

            // fifo
            scanned_sprites: VecDeque::new(),
            scanned_sprites_peek: None,
        }
    }

    pub fn override_color_palettes(&mut self, palettes: &[Rgb; 12]) {
        self.bg_color_palette[0].copy_from_slice(&palettes[0..4]);
        self.sprite_color_palette[0].copy_from_slice(&palettes[4..8]);
        self.sprite_color_palette[1].copy_from_slice(&palettes[8..12]);
    }

    pub fn override_obj_prio_mode(&mut self, mode: ObjectPriorityMode) {
        self.obj_prio_mode = mode;
    }

    #[cfg(feature = "debug_info")]
    pub fn bg_color_palette(&self) -> [[Rgb; 4]; 8] {
        self.bg_color_palette
    }

    #[cfg(feature = "debug_info")]
    pub fn sprite_color_palette(&self) -> [[Rgb; 4]; 8] {
        self.sprite_color_palette
    }

    #[cfg(feature = "debug_info")]
    pub fn bg_palette(&self) -> [usize; 4] {
        self.bg_palette
    }

    #[cfg(feature = "debug_info")]
    pub fn sprite_palette(&self) -> [[usize; 4]; 2] {
        self.sprite_palette
    }

    pub fn set_console_compatibility(&mut self, mode: CgbCompatibility) {
        self.console_compatibility_mode = mode;
        self.hdma.set_console_compatibility(mode);

        if let CgbCompatibility::None = mode {
            self.stat_change_offset = 4;
        }
    }

    #[cfg(not(feature = "web"))]
    pub fn get_frame_buffer(&self) -> &[Rgb] {
        &self.frame_buffer
    }

    #[cfg(feature = "web")]
    pub fn get_frame_buffer(&self) -> [Rgb; 160 * 144] {
        self.frame_buffer
    }

    pub fn consume_draw_flag(&mut self) -> bool {
        let flag = self.draw_flag;
        self.draw_flag = false;
        flag
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        self.gpu_vram[(self.gpu_vram_bank & 1) as usize][addr as usize]
    }

    pub fn write_vram(&mut self, addr: u16, val: u8) {
        self.gpu_vram[(self.gpu_vram_bank & 1) as usize][addr as usize] = val;
    }

    fn reset(&mut self) {
        self.handle_lcd_powered_off = true;
        self.ly = 0;
        self.line_clock_cycles = 0;
        self.mode_clock_cycles = 0;
        self.mode = PpuMode::OAM;
        self.frame_buffer = [Rgb::const_mono(255); 160 * 144];
        self.stat &= 0b1111_1100;
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => 0b1000_0000 | self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => self.dma,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,

            0xFF4F => self.gpu_vram_bank,

            0xFF68 => self.bg_color_palette_specification | 0b1100_0000,
            0xFF69 => self.bg_color_palette_ram[self.bg_color_palette_index],
            0xFF6A => self.sprite_color_palette_specification | 0b1100_0000,
            0xFF6B => self.sprite_color_palette_ram[self.sprite_color_palette_index],

            0xFF6C => 0xFF,

            _ => panic!("Ppu doesnt handle reading from address: {:#06X}", addr),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8, interrupts: &mut Interrupts) {
        match addr {
            0xFF40 => {
                // was the ppu off, and now turning on?
                if (self.lcdc & LcdControlFlag::LCDDisplayEnable as u8) == 0
                    && (val & LcdControlFlag::LCDDisplayEnable as u8) != 0
                {
                    log::debug!("Powering LCD ON");
                    debug_assert!(self.handle_lcd_powered_off);
                    self.handle_lcd_powered_off = false;

                    // TODO: ly0 when power on is shorter by a few cycles
                }

                self.lcdc = val;

                // is the ppu turning off
                if self.lcdc & LcdControlFlag::LCDDisplayEnable as u8 == 0 {
                    log::debug!("Powering LCD OFF");
                    self.reset();
                    // TODO: oam/vram unlocking
                }
            }
            0xFF41 => {
                self.stat = (self.stat & 0b1000_0111) | (val & 0b0111_1000);
                self.update_stat_irq_conditions(interrupts);
            }
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => { /* LY is read only */ }
            0xFF45 => {
                self.lyc = val;

                if self.lcdc & LcdControlFlag::LCDDisplayEnable as u8 != 0 {
                    self.update_ly_lyc();
                }
            }
            0xFF46 => self.dma = val,
            0xFF47 => {
                self.bgp = val;
                for i in 0..4 {
                    self.bg_palette[i] = ((val >> (i * 2)) & 0b11) as usize;
                }
            }
            0xFF48 => {
                self.obp0 = val;
                for i in 0..4 {
                    self.sprite_palette[0][i] = ((val >> (i * 2)) & 0b11) as usize;
                }
            }
            0xFF49 => {
                self.obp1 = val;
                for i in 0..4 {
                    self.sprite_palette[1][i] = ((val >> (i * 2)) & 0b11) as usize;
                }
            }
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,

            0xFF4F => self.gpu_vram_bank = val | 0b1111_1110,

            0xFF68 => {
                self.bg_color_palette_specification = val;
                self.bg_color_palette_index =
                    (self.bg_color_palette_specification & 0b0011_1111) as usize;
                self.bg_color_palette_auto_increment =
                    self.bg_color_palette_specification & 0b1000_0000 != 0;
            }
            0xFF69 => {
                let index = self.bg_color_palette_index;
                self.bg_color_palette_ram[self.bg_color_palette_index] = val;

                let bgr555: u16 = ((self.bg_color_palette_ram[index | 1] as u16) << 8)
                    | (self.bg_color_palette_ram[index & !1] as u16);
                let rgb = Rgb::from_bgr555(bgr555);

                let palette_index = self.bg_color_palette_index >> 3;
                let palette_color_bit = (self.bg_color_palette_index & 7) >> 1;
                self.bg_color_palette[palette_index][palette_color_bit] = rgb;

                if self.bg_color_palette_auto_increment {
                    self.bg_color_palette_index += 1;
                    self.bg_color_palette_index &= 0x3F; // handle 5bit overflow

                    self.bg_color_palette_specification = self.bg_color_palette_index as u8;
                }
            }

            0xFF6A => {
                self.sprite_color_palette_specification = val;
                self.sprite_color_palette_index =
                    (self.sprite_color_palette_specification & 0b0011_1111) as usize;
                self.sprite_color_palette_auto_increment =
                    self.sprite_color_palette_specification & 0b1000_0000 != 0;
            }
            0xFF6B => {
                let index = self.sprite_color_palette_index;
                self.sprite_color_palette_ram[self.sprite_color_palette_index] = val;

                let bgr555: u16 = ((self.sprite_color_palette_ram[index | 1] as u16) << 8)
                    | (self.sprite_color_palette_ram[index & !1] as u16);
                let rgb = Rgb::from_bgr555(bgr555);

                let palette_index = self.sprite_color_palette_index >> 3;
                let palette_color_bit = (self.sprite_color_palette_index & 7) >> 1;
                self.sprite_color_palette[palette_index][palette_color_bit] = rgb;

                if self.sprite_color_palette_auto_increment {
                    self.sprite_color_palette_index += 1;
                    self.sprite_color_palette_index &= 0x3F; // handle 5bit overflow

                    self.sprite_color_palette_specification = self.sprite_color_palette_index as u8;
                }
            }

            0xFF6C => match val & 0b0000_0001 {
                0 => self.obj_prio_mode = ObjectPriorityMode::OamOrder,
                1 => self.obj_prio_mode = ObjectPriorityMode::CoordinateOrder,
                _ => unsafe { unreachable_unchecked() },
            },

            _ => panic!("Ppu doesnt handle writing to address: {:#06X}", addr),
        }
    }

    fn update_ly_lyc(&mut self) {
        if self.ly == self.lyc {
            self.stat |= 0b0000_0100;
        } else {
            self.stat &= 0b1111_1011;
        }
    }

    fn set_mode_stat(&mut self, mode: PpuMode) {
        self.stat = (self.stat & 0b1111_1100) | (mode as u8);
    }

    pub fn get_mode_stat(&self) -> PpuMode {
        (self.stat & 0b0000_0011).into()
    }

    fn flip_tile_value(val: u8) -> u8 {
        #[cfg(debug_assertions)]
        assert!(val <= 7);

        match val {
            0 => 7,
            1 => 6,
            2 => 5,
            3 => 4,
            4 => 3,
            5 => 2,
            6 => 1,
            7 => 0,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    fn get_bg_map_start_addr(&self) -> u16 {
        match (self.lcdc & LcdControlFlag::BGTileMapAddress as u8) != 0 {
            true => 0x9C00,
            false => 0x9800,
        }
    }

    fn get_window_map_start_addr(&self) -> u16 {
        match (self.lcdc & LcdControlFlag::WindowTileMapAddress as u8) != 0 {
            true => 0x9C00,
            false => 0x9800,
        }
    }

    fn get_adjusted_tile_index(&self, addr: u16, signed_tile_index: bool) -> u16 {
        let addr = (addr - 0x8000) as usize;
        if signed_tile_index {
            let tile = self.gpu_vram[0][addr] as i8 as i16;
            if tile >= 0 {
                tile as u16 + 256
            } else {
                256 - (tile.unsigned_abs() as u16)
            }
        } else {
            self.gpu_vram[0][addr] as u16
        }
    }

    fn get_sprite_size(&self) -> i32 {
        if self.lcdc & LcdControlFlag::OBJSize as u8 != 0 {
            16
        } else {
            8
        }
    }

    fn hblank(&mut self, interrupts: &mut Interrupts) {
        if self.line_clock_cycles == 456 {
            self.mode_clock_cycles = 0;
            self.line_clock_cycles = 0;

            self.ly += 1;
            self.update_ly_lyc();

            if self.ly == 144 {
                interrupts.request_interupt(InterruptFlag::VBlank);
                self.mode = PpuMode::VBlank;
                self.set_mode_stat(PpuMode::VBlank);

                self.draw_flag = true;
            } else {
                self.set_mode_stat(PpuMode::OAM);
                self.mode = PpuMode::OAM;
            }
        }
    }

    fn vblank(&mut self, _: &mut Interrupts) {
        if self.ly == 153 && self.line_clock_cycles == 4 {
            self.ly = 0;
            self.update_ly_lyc();
            self.ly_153_early = true;
        }

        if self.line_clock_cycles == 456 {
            self.line_clock_cycles = 0;

            if self.ly < 153 && !self.ly_153_early {
                self.ly += 1;
                self.update_ly_lyc();
            } else {
                self.ly_153_early = false;

                self.mode_clock_cycles = 0;
                self.window_internal_line_counter = 0;
                self.set_mode_stat(PpuMode::OAM);
                self.mode = PpuMode::OAM;
            }
        }
    }

    fn oam(&mut self) {
        if self.mode_clock_cycles == 80 {
            self.mode_clock_cycles = 0;
            self.mode = PpuMode::VRAM;
            self.set_mode_stat(PpuMode::VRAM);

            // technically each sprite check takes 2 cycles, with 40 objs 40 * 2 = 80 cycles
            // hence why oam is 80 cycles long
            // but for now lets just wait till the end and then scan the oam all at once
            // this is technically fine if oam memory is write locked, which I believe it is at this point

            let sprite_size = self.get_sprite_size();
            let mut sprites: Vec<SpriteInfo> = self
                .sprite_table
                .chunks(4)
                .filter_map(|raw_obj_data| {
                    let sprite_info = SpriteInfo::from((raw_obj_data, sprite_size));
                    if sprite_info.x > 0
                        && self.ly + 16 >= sprite_info.y as u8
                        && self.ly as i32 + 16 < sprite_info.y + sprite_size
                    {
                        return Some(sprite_info);
                    }

                    None
                })
                .collect();

            if matches!(self.obj_prio_mode, ObjectPriorityMode::CoordinateOrder) {
                sprites.sort_by_key(|sprite| sprite.x);
            }

            self.scanned_sprites = VecDeque::from(sprites);
            self.scanned_sprites_peek = self.scanned_sprites.pop_front();
        }
    }

    fn vram(&mut self) {
        if self.mode_clock_cycles == 172 {
            self.mode_clock_cycles = 0;
            //self.draw_scan_line(); // draw line!
            self.mode = PpuMode::HBlank;
            self.set_mode_stat(PpuMode::HBlank);
        }
    }

    fn update_stat_irq_conditions(&mut self, interrupts: &mut Interrupts) {
        let mut stat_irq_state = false;
        let ppu_mode_stat = PpuMode::from(self.stat & 0b0000_0011);

        match ppu_mode_stat {
            PpuMode::HBlank => {
                if self.stat & 0b0000_1000 != 0 {
                    stat_irq_state = true;
                }
            }

            PpuMode::VBlank => {
                if self.stat & 0b0001_0000 != 0 {
                    stat_irq_state = true;
                }
            }

            PpuMode::OAM => {
                if self.stat & 0b0010_0000 != 0 {
                    stat_irq_state = true;
                }
            }
            PpuMode::VRAM => {}
        }

        if self.stat & 0b0100_0000 != 0 && self.ly == self.lyc {
            stat_irq_state = true;
        }

        if !self.stat_irq_state && stat_irq_state {
            interrupts.request_interupt(InterruptFlag::Stat);
        }

        self.stat_irq_state = stat_irq_state;
    }

    pub fn tick(&mut self, interrupts: &mut Interrupts) {
        // is the ppu off?
        if self.lcdc & LcdControlFlag::LCDDisplayEnable as u8 == 0 {
            return;
        }

        self.mode_clock_cycles += 1;
        self.line_clock_cycles += 1;

        match self.mode {
            PpuMode::HBlank => self.hblank(interrupts),
            PpuMode::VBlank => self.vblank(interrupts),
            PpuMode::OAM => self.oam(),
            PpuMode::VRAM => self.vram(),
        }

        self.update_stat_irq_conditions(interrupts);
    }
}
