pub mod rgb;

use std::hint::unreachable_unchecked;

use crate::bus::CgbCompatibility;

use self::rgb::Rgb;

use super::interrupts::{InterruptFlag, Interrupts};

const CGB_PTR_PALETTE: [usize; 4] = [0, 1, 2, 3];

pub(crate) struct Ppu {
    gpu_vram: [[u8; 0x2000]; 2],
    gpu_vram_bank: u8,

    pub sprite_table: [u8; 0xA0],
    pub sprite_palette: [[usize; 4]; 2],

    frame_buffer: [Rgb; 160 * 144],
    draw_flag: bool,

    bg_palette: [usize; 4],

    mode: PpuMode,
    window_internal_line_counter: u8,
    console_compatibility_mode: CgbCompatibility,
    obj_prio_mode: ObjectPriorityMode,

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

    bg_color_palette_ram: [u8; 64],
    bg_color_palette: [[Rgb; 4]; 8],
    bg_color_palette_specification: u8, // FF68
    bg_color_palette_index: usize,
    bg_color_palette_auto_increment: bool,

    sprite_color_palette_ram: [u8; 64],
    sprite_color_palette: [[Rgb; 4]; 8],
    sprite_color_palette_specification: u8, // FF6A
    sprite_color_palette_index: usize,
    sprite_color_palette_auto_increment: bool,

    line_clock_cycles: u64,
    mode_clock_cycles: u64,
}

#[derive(Clone, Copy)]
pub enum LcdControlFlag {
    // 1: on, 0: off
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
#[allow(clippy::upper_case_acronyms)]
enum PpuMode {
    HBlank = 0, // mode 0
    VBlank = 1, // mode 1
    OAM = 2,    // mode 2
    VRAM = 3,   // mode 3
}

#[derive(Clone, Copy)]
enum ObjectPriorityMode {
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

impl Ppu {
    pub fn new() -> Self {
        Self {
            gpu_vram: [[0; 0x2000]; 2],
            gpu_vram_bank: 0b1111_1110,

            sprite_table: [0; 0xA0],
            sprite_palette: [
                [
                    CGB_PTR_PALETTE[0],
                    CGB_PTR_PALETTE[1],
                    CGB_PTR_PALETTE[2],
                    CGB_PTR_PALETTE[3],
                ],
                [
                    CGB_PTR_PALETTE[0],
                    CGB_PTR_PALETTE[1],
                    CGB_PTR_PALETTE[2],
                    CGB_PTR_PALETTE[3],
                ],
            ],

            frame_buffer: [Rgb::default(); 160 * 144],
            draw_flag: false,

            // ppu
            bg_palette: [
                CGB_PTR_PALETTE[0],
                CGB_PTR_PALETTE[1],
                CGB_PTR_PALETTE[2],
                CGB_PTR_PALETTE[3],
            ],

            mode: PpuMode::OAM,
            window_internal_line_counter: 0,
            console_compatibility_mode: CgbCompatibility::CgbOnly,
            obj_prio_mode: ObjectPriorityMode::OamOrder, // TODO: what is the default?

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

            sprite_color_palette_ram: [0xFF; 64],
            sprite_color_palette: [[Rgb::default(); 4]; 8],
            sprite_color_palette_specification: 0xFF,
            sprite_color_palette_index: 0,
            sprite_color_palette_auto_increment: false,

            line_clock_cycles: 0,
            mode_clock_cycles: 0,
        }
    }

    pub fn set_console_compatibility(&mut self, console_compatibility_mode: CgbCompatibility) {
        self.console_compatibility_mode = console_compatibility_mode;
    }

    pub fn get_frame_buffer(&self) -> &[Rgb] {
        &self.frame_buffer
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

            0xFF68 => self.bg_color_palette_specification,
            0xFF69 => self.bg_color_palette_ram[self.bg_color_palette_index],
            0xFF6A => self.sprite_color_palette_specification,
            0xFF6B => self.sprite_color_palette_ram[self.sprite_color_palette_index],

            0xFF6C => 0xFF,

            _ => panic!("Ppu doesnt handle reading from address: {:#06X}", addr),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF40 => {
                self.lcdc = val;

                // is the lcd now off?
                if self.lcdc & LcdControlFlag::LCDDisplayEnable as u8 == 0 {
                    self.ly = 0;

                    // TODO: oam/vram unlocking
                }
            }
            0xFF41 => {
                self.stat = (self.stat & 0b1000_0111) | (val & 0b0111_1000);

                // TODO: fire stat_irq
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
                    self.bg_palette[i] = CGB_PTR_PALETTE[((val >> (i * 2)) & 3) as usize];
                }
            }
            0xFF48 => {
                self.obp0 = val;
                for i in 0..4 {
                    self.sprite_palette[0][i] = CGB_PTR_PALETTE[((val >> (i * 2)) & 3) as usize];
                }
            }
            0xFF49 => {
                self.obp1 = val;
                for i in 0..4 {
                    self.sprite_palette[1][i] = CGB_PTR_PALETTE[((val >> (i * 2)) & 3) as usize];
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
                    self.bg_color_palette_index &= 0x3F // handle 5bit overflow
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
                    self.sprite_color_palette_index &= 0x3F // handle 5bit overflow
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

    #[inline(always)]
    fn access_vram(&self, index: usize) -> u8 {
        self.gpu_vram[(self.gpu_vram_bank & 1) as usize][index]
    }

    fn update_ly_lyc(&mut self) {
        if self.ly == self.lyc {
            self.stat |= 0b0000_0100;
        } else {
            self.stat &= 0b1111_1011;
        }
    }

    fn handle_ly_eq_lyc(&mut self, interrupts: &mut Interrupts) {
        self.update_ly_lyc();
        let lyc_check_enabled = self.stat & 0b0100_0000 != 0;
        if self.ly == self.lyc && lyc_check_enabled {
            interrupts.request_interupt(InterruptFlag::Stat);
        }
    }

    fn set_mode_stat(&mut self, mode: PpuMode) {
        self.stat = (self.stat & 0b1111_1100) | (mode as u8);
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
            let tile = self.access_vram(addr) as i8 as i16;
            if tile >= 0 {
                tile as u16 + 256
            } else {
                256 - (tile.abs() as u16)
            }
        } else {
            self.access_vram(addr) as u16
        }
    }

    fn hblank(&mut self, interrupts: &mut Interrupts) {
        if self.line_clock_cycles == 456 {
            self.mode_clock_cycles = 0;
            self.line_clock_cycles = 0;

            self.ly += 1;
            self.handle_ly_eq_lyc(interrupts);

            if self.ly == 144 {
                interrupts.request_interupt(InterruptFlag::VBlank);
                self.set_mode_stat(PpuMode::VBlank);
                self.mode = PpuMode::VBlank;

                self.draw_flag = true;
            } else {
                self.set_mode_stat(PpuMode::OAM);
                self.mode = PpuMode::OAM;
            }
        }
    }

    fn vblank(&mut self, interrupts: &mut Interrupts) {
        if self.line_clock_cycles == 456 {
            self.line_clock_cycles = 0;

            self.ly += 1;
            self.handle_ly_eq_lyc(interrupts);

            if self.ly == 154 {
                self.ly = 0;
                self.handle_ly_eq_lyc(interrupts);

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
            self.set_mode_stat(PpuMode::VRAM);
            self.mode = PpuMode::VRAM;
        }
    }

    fn vram(&mut self) {
        if self.mode_clock_cycles == 172 {
            self.mode_clock_cycles = 0;
            self.draw_scan_line(); // draw line!
            self.set_mode_stat(PpuMode::HBlank);
            self.mode = PpuMode::HBlank;
        }
    }

    pub fn tick(&mut self, interrupts: &mut Interrupts) {
        self.mode_clock_cycles += 1;
        self.line_clock_cycles += 1;

        match self.mode {
            PpuMode::HBlank => self.hblank(interrupts),
            PpuMode::VBlank => self.vblank(interrupts),
            PpuMode::OAM => self.oam(),
            PpuMode::VRAM => self.vram(),
        }
    }

    fn draw_scan_line(&mut self) {
        let mut scan_line_row: [ScanLinePxInfo; 160] = [ScanLinePxInfo::default(); 160];

        if self.console_compatibility_mode.is_cgb_mode()
            || self.lcdc & LcdControlFlag::BGEnable as u8 != 0
        {
            self.draw_background_2(&mut scan_line_row);
        }

        let skip_window_draw =
            self.wy > 166 || (self.wx.wrapping_sub(7)) > 143 || self.wy > self.ly;

        // TODO: skip window draw if if non cgb mode and if BGEnable is disabled?
        if self.lcdc & LcdControlFlag::WindowEnable as u8 != 0 && !skip_window_draw {
            self.draw_window(&mut scan_line_row);
        }

        if self.lcdc & LcdControlFlag::OBJEnable as u8 != 0 {
            self.draw_sprites(&mut scan_line_row);
        }
    }

    fn draw_background_2(&mut self, scan_line_row: &mut [ScanLinePxInfo; 160]) {
        struct TileData {
            color_index_row: [u8; 8],
            flags: BGMapFlags,
        }

        fn fetch_tile_data(ppu: &Ppu, x: u8, y: u8) -> TileData {
            let bg_map_start_addr = ppu.get_bg_map_start_addr();

            let tile_y = y / 8;
            let tile_x = x / 8;

            let bg_map_tile_index = ((tile_y as u16) * 32) + tile_x as u16;
            let signed_tile_addressing: bool =
                ppu.lcdc & LcdControlFlag::BGAndWindowTileData as u8 == 0;
            let tile_index = ppu.get_adjusted_tile_index(
                bg_map_start_addr + bg_map_tile_index,
                signed_tile_addressing,
            );

            let flags_addr = (bg_map_start_addr + bg_map_tile_index - 0x8000) as usize;
            let flags = BGMapFlags::from(ppu.gpu_vram[1][flags_addr]);

            let tile_local_y = match flags.vertical_flip {
                true => Ppu::flip_tile_value(y & 7),
                false => y & 7,
            };
            let tile_addr = (tile_index * 16) + (tile_local_y as u16 * 2);

            let bank_index = match ppu.console_compatibility_mode {
                CgbCompatibility::CgbOnly => flags.tile_bank,
                _ => 0,
            };

            let b1 = ppu.gpu_vram[bank_index][tile_addr as usize];
            let b2 = ppu.gpu_vram[bank_index][tile_addr as usize + 1];

            let mut color_index_row: [u8; 8] = [0; 8];

            if ppu.console_compatibility_mode.is_cgb_mode() && flags.horizontal_flip {
                for i in 0..8usize {
                    let shift_i = i as u8;
                    color_index_row[i] = ((b1 & (1 << shift_i)) >> shift_i)
                        | ((b2 & (1 << shift_i)) >> shift_i) << 1;
                }
            } else {
                for i in (0..8usize).rev() {
                    let shift_i = i as u8;
                    color_index_row[7 - i] = ((b1 & (1 << shift_i)) >> shift_i)
                        | ((b2 & (1 << shift_i)) >> shift_i) << 1;
                }
            }

            TileData {
                color_index_row,
                flags,
            }
        }

        let y = self.ly.wrapping_add(self.scy);
        let mut x = self.scx;
        let mut tile_local_x = x & 7;

        let mut tile_data = fetch_tile_data(self, x, y);
        let mut frame_buffer_offset = self.ly as usize * 160;
        let mut pixels_drawn_for_current_tile: u8 = 0;

        for px in scan_line_row.iter_mut() {
            let color_bit = tile_data.color_index_row[tile_local_x as usize];
            let color_index = self.bg_palette[color_bit as usize];
            let color_palette_index = match self.console_compatibility_mode {
                CgbCompatibility::CgbOnly => tile_data.flags.bg_palette_number,
                _ => 0,
            };

            let color = self.bg_color_palette[color_palette_index][color_index];
            *px = ScanLinePxInfo::new(color_index, tile_data.flags.bg_oam_prio);
            self.frame_buffer[frame_buffer_offset] = color;
            frame_buffer_offset += 1;

            tile_local_x += 1;
            pixels_drawn_for_current_tile += 1;

            if tile_local_x == 8 {
                tile_local_x = 0;
                x = x.wrapping_add(pixels_drawn_for_current_tile);
                pixels_drawn_for_current_tile = 0;

                tile_data = fetch_tile_data(self, x, y);
            }
        }
    }

    fn draw_background(&mut self, scan_line_row: &mut [ScanLinePxInfo; 160]) {
        let bg_map_start_addr = self.get_bg_map_start_addr();

        // top left coordinate of the view port
        // very important that these are u8's so overflowing
        // naturally handles "view port wrapping around"
        let y: u8 = self.ly.wrapping_add(self.scy);
        let mut x: u8 = self.scx;

        // get the "tile" (x, y), which 8x8 chunk is the above coordinate in
        let tile_y = y / 8;
        let mut tile_x = x / 8;

        // calculate tile index
        let mut tile_map_offset = (tile_y as u16 * 32) + tile_x as u16;
        // are we using signed addressing for accessing the tile data (not map)
        let signed_tile_addressing: bool =
            self.lcdc & LcdControlFlag::BGAndWindowTileData as u8 == 0;

        // get the tile index from the map
        let mut tile_index = self
            .get_adjusted_tile_index(bg_map_start_addr + tile_map_offset, signed_tile_addressing);

        // In CGB mode, BG tiles have attribute flags in bank 1 in the corresponding map area
        let mut flags_addr = (bg_map_start_addr + tile_map_offset - 0x8000) as usize;
        let mut flags = BGMapFlags::from(self.gpu_vram[1][flags_addr]);

        // above x and y are where we are relative to the whole bg map (256 * 256)
        // we need x and y to be relative to the tile we want to draw
        // so modulo by 8 (or & 7)
        // shadow over old x an y variables
        let mut tile_local_y = if flags.vertical_flip {
            Self::flip_tile_value(y & 7)
        } else {
            y & 7
        };
        let mut tile_local_x = if flags.horizontal_flip {
            Self::flip_tile_value(x & 7)
        } else {
            x & 7
        };

        let mut tile_address = (tile_index * 16) + (tile_local_y as u16 * 2);

        let (mut b1, mut b2) = match self.console_compatibility_mode {
            CgbCompatibility::CgbAndDmg | CgbCompatibility::None => (
                self.gpu_vram[0][tile_address as usize],
                self.gpu_vram[0][tile_address as usize + 1],
            ),
            CgbCompatibility::CgbOnly => (
                self.gpu_vram[flags.tile_bank][tile_address as usize],
                self.gpu_vram[flags.tile_bank][tile_address as usize + 1],
            ),
        };

        let mut frame_buffer_offset = self.ly as usize * 160;

        let mut pixels_drawn_for_current_tile: u8 = 0;
        for px in scan_line_row.iter_mut() {
            let bx = 7 - tile_local_x;

            let color_bit = ((b1 & (1 << bx)) >> bx) | ((b2 & (1 << bx)) >> bx) << 1;
            let color_index = self.bg_palette[color_bit as usize];
            let color_palette_index = if self.console_compatibility_mode.is_cgb_mode() {
                flags.bg_palette_number
            } else {
                0
            };
            let color = self.bg_color_palette[color_palette_index][color_index];

            *px = ScanLinePxInfo::new(color_index, flags.bg_oam_prio);
            self.frame_buffer[frame_buffer_offset] = color;
            frame_buffer_offset += 1;

            tile_local_x = if flags.horizontal_flip {
                tile_local_x - 1
            } else {
                tile_local_x + 1
            };
            pixels_drawn_for_current_tile += 1;

            if (tile_local_x == 8 && !flags.horizontal_flip)
                || (tile_local_x == 0 && flags.horizontal_flip)
            {
                // set up the next tile
                // need to be carefull here (i think?) becaucse the view port can
                // wrap around?

                x = x.wrapping_add(pixels_drawn_for_current_tile);
                pixels_drawn_for_current_tile = 0;

                tile_x = x / 8;
                tile_map_offset = (tile_y as u16 * 32) + tile_x as u16;
                tile_index = self.get_adjusted_tile_index(
                    bg_map_start_addr + tile_map_offset,
                    signed_tile_addressing,
                );
                flags_addr = (bg_map_start_addr + tile_map_offset - 0x8000) as usize;
                flags = BGMapFlags::from(self.gpu_vram[1][flags_addr]);

                tile_local_x = if flags.horizontal_flip { 7 } else { 0 };
                tile_local_y = if flags.vertical_flip {
                    Self::flip_tile_value(y & 7)
                } else {
                    y & 7
                };

                tile_address = (tile_index * 16) + (tile_local_y as u16 * 2);

                let (_b1, _b2) = match self.console_compatibility_mode {
                    CgbCompatibility::CgbAndDmg | CgbCompatibility::None => (
                        self.gpu_vram[0][tile_address as usize],
                        self.gpu_vram[0][tile_address as usize + 1],
                    ),
                    CgbCompatibility::CgbOnly => (
                        self.gpu_vram[flags.tile_bank][tile_address as usize],
                        self.gpu_vram[flags.tile_bank][tile_address as usize + 1],
                    ),
                };

                b1 = _b1;
                b2 = _b2;
            }
        }
    }

    fn draw_window(&mut self, scan_line_row: &mut [ScanLinePxInfo; 160]) {
        let wd_map_start_addr = self.get_window_map_start_addr();
        self.window_internal_line_counter += 1;

        let y = self.window_internal_line_counter - 1; //scan_line - window_y;
        let mut x = self.wx.wrapping_sub(7);

        let tile_y = y / 8;

        let mut tile_map_offset = tile_y as u16 * 32;
        // tile_map_offset = (tile_y as u16) + tile_x as u16;
        // are we using signed addressing for accessing the tile data (not map)
        let signed_tile_addressing: bool =
            self.lcdc & LcdControlFlag::BGAndWindowTileData as u8 == 0;

        // get the tile index from the map
        let mut tile_index = self
            .get_adjusted_tile_index(wd_map_start_addr + tile_map_offset, signed_tile_addressing);

        // above x and y are where we are relative to the whole bg map (256 * 256)
        // we need x and y to be relative to the tile we want to draw
        // so modulo by 8 (or & 7)
        // shadow over old x an y variables
        let tile_local_y = y & 7;
        let mut tile_local_x = x & 7;

        let mut tile_address = (tile_index * 16) + (tile_local_y as u16 * 2);
        let mut b1 = self.access_vram(tile_address as usize);
        let mut b2 = self.access_vram(tile_address as usize + 1);

        let mut frame_buffer_offset = (self.ly as usize * 160) + x as usize;

        let mut pixels_drawn_for_current_tile: u8 = 0;
        let start = x;
        for i in start..160 {
            let bx = 7 - tile_local_x;
            let color_bit = ((b1 & (1 << bx)) >> bx) | ((b2 & (1 << bx)) >> bx) << 1;
            let color_index = self.bg_palette[color_bit as usize];
            let color = self.bg_color_palette[0][color_index];

            scan_line_row[i as usize] = ScanLinePxInfo::new(color_index, false);
            self.frame_buffer[frame_buffer_offset] = color;
            frame_buffer_offset += 1;

            tile_local_x += 1;
            pixels_drawn_for_current_tile += 1;

            if tile_local_x == 8 {
                tile_local_x = 0;

                // set up the next tile
                // need to be carefull here (i think?) becaucse the view port can
                // wrap around?

                x = x.wrapping_add(pixels_drawn_for_current_tile);
                pixels_drawn_for_current_tile = 0;

                tile_map_offset += 1;
                tile_index = self.get_adjusted_tile_index(
                    wd_map_start_addr + tile_map_offset,
                    signed_tile_addressing,
                );

                tile_address = (tile_index * 16) + (tile_local_y as u16 * 2);
                b1 = self.access_vram(tile_address as usize);
                b2 = self.access_vram(tile_address as usize + 1);
            }
        }
    }

    fn draw_sprites(&mut self, scan_line_row: &mut [ScanLinePxInfo; 160]) {
        #[derive(Clone, Copy)]
        struct SpriteData {
            y: i32,
            x: i32,
            tile_num: u16,
            flags: u8,
        }

        impl Default for SpriteData {
            fn default() -> Self {
                Self {
                    y: Default::default(),
                    x: Default::default(),
                    tile_num: Default::default(),
                    flags: Default::default(),
                }
            }
        }

        fn fetch_sprites(ppu: &Ppu, sprite_size: i32) -> [SpriteData; 40] {
            let mut sprites = [SpriteData::default(); 40];

            for i in 0..40 {
                let sprite_addr = (i as usize) * 4;
                sprites[i] = SpriteData {
                    y: ppu.sprite_table[sprite_addr] as u16 as i32 - 16,
                    x: ppu.sprite_table[sprite_addr + 1] as u16 as i32 - 8,
                    tile_num: (ppu.sprite_table[sprite_addr + 2]
                        & (if sprite_size == 16 { 0xFE } else { 0xFF }))
                        as u16,
                    flags: ppu.sprite_table[sprite_addr + 3],
                };
            }

            sprites
        }

        let sprite_size: i32 = if self.lcdc & LcdControlFlag::OBJSize as u8 != 0 {
            16
        } else {
            8
        };

        let mut total_objects_drawn = 0;
        let mut obj_prio_arr = [i32::MAX; 160];

        let sprites = fetch_sprites(self, sprite_size);

        for (i, sprite) in sprites.iter().enumerate() {
            let cgb_sprite_palette = (sprite.flags & 0b0000_0111) as usize;
            let tile_vram_bank = ((sprite.flags & 0b0000_1000) >> 3) as usize;

            let sprite_palette: usize = if sprite.flags & (1 << 4) != 0 { 1 } else { 0 };
            let xflip: bool = sprite.flags & (1 << 5) != 0;
            let yflip: bool = sprite.flags & (1 << 6) != 0;
            let bg_wd_prio: bool = sprite.flags & (1 << 7) != 0;

            let scan_line = self.ly as i32;

            // exit early if sprite is off screen
            if scan_line < sprite.y || scan_line >= sprite.y + sprite_size {
                continue;
            }
            if sprite.x < -7 || sprite.x >= 160 {
                continue;
            }

            total_objects_drawn += 1;
            if total_objects_drawn > 10 {
                break;
            }

            // fetch sprite tile
            let tile_y: u16 = if yflip {
                (sprite_size - 1 - (scan_line - sprite.y)) as u16
            } else {
                (scan_line - sprite.y) as u16
            };

            let tile_address = sprite.tile_num * 16 + tile_y * 2;

            let (b1, b2) = match self.console_compatibility_mode {
                CgbCompatibility::CgbAndDmg | CgbCompatibility::None => (
                    self.gpu_vram[0][tile_address as usize],
                    self.gpu_vram[0][tile_address as usize + 1],
                ),
                CgbCompatibility::CgbOnly => (
                    self.gpu_vram[tile_vram_bank][tile_address as usize],
                    self.gpu_vram[tile_vram_bank][tile_address as usize + 1],
                ),
            };

            // draw each pixel of the sprite tile
            'inner: for x in 0..8 {
                if sprite.x + x < 0 || sprite.x + x >= 160 {
                    continue;
                }

                let cgb_sprite_alawys_display = if self.console_compatibility_mode.is_cgb_mode() {
                    let always_display_sprite = self.lcdc & LcdControlFlag::BGEnable as u8 == 0;
                    if !always_display_sprite
                        && scan_line_row[(sprite.x + x) as usize].bg_prio_set
                        && scan_line_row[(sprite.x + x) as usize].color_index != 0
                    {
                        continue 'inner;
                    }

                    always_display_sprite
                } else {
                    false
                };

                if !cgb_sprite_alawys_display
                    && bg_wd_prio
                    && scan_line_row[(sprite.x + x) as usize].color_index != 0
                {
                    continue 'inner;
                }

                // has another sprite already been drawn on this pixel
                // that has a lower or eq sprite_x val?

                // handle obj prio
                match self.obj_prio_mode {
                    ObjectPriorityMode::OamOrder => {
                        if obj_prio_arr[(sprite.x + x) as usize] <= i as i32 {
                            continue 'inner;
                        }
                    }
                    ObjectPriorityMode::CoordinateOrder => {
                        if obj_prio_arr[(sprite.x + x) as usize] <= sprite.x {
                            continue 'inner;
                        }
                    }
                }

                let xbit = 1 << (if xflip { x } else { 7 - x } as u32);
                let colnr =
                    (if b1 & xbit != 0 { 1 } else { 0 }) | (if b2 & xbit != 0 { 2 } else { 0 });

                // LCDControl Mater Priority cleared still means we skip sprites if they are the bit 0 ("transparent")
                if colnr == 0 {
                    continue;
                }

                let color = match self.console_compatibility_mode {
                    CgbCompatibility::None | CgbCompatibility::CgbAndDmg => {
                        let color_bit = self.sprite_palette[sprite_palette][colnr];
                        self.sprite_color_palette[sprite_palette][color_bit]
                    }
                    CgbCompatibility::CgbOnly => {
                        self.sprite_color_palette[cgb_sprite_palette][colnr]
                    }
                };

                let pixel_offset = ((scan_line * 160) + sprite.x + x) as usize;

                self.frame_buffer[pixel_offset] = color;
                match self.obj_prio_mode {
                    ObjectPriorityMode::OamOrder => {
                        obj_prio_arr[(sprite.x + x) as usize] = i as i32
                    }
                    ObjectPriorityMode::CoordinateOrder => {
                        obj_prio_arr[(sprite.x + x) as usize] = sprite.x
                    }
                }
            }
        }
    }
}
