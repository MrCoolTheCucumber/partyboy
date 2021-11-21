use super::interrupts::{InterruptFlag, Interrupts};

const PALETTE: [u8; 4] = [255, 192, 96, 0];

pub struct Ppu {
    pub gpu_vram: [u8; 0x2000],
    pub sprite_table: [u8; 0xA0],
    pub sprite_palette: [[u8; 4]; 2],

    frame_buffer: [u8; 160 * 144],
    draw_flag: bool,

    tileset: [[[u8; 8]; 8]; 384],
    bg_palette: [u8; 4],

    mode: PpuMode,
    window_internal_line_counter: u8,

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
enum PpuMode {
    HBlank = 0, // mode 0
    VBlank = 1, // mode 1
    OAM = 2,    // mode 2
    VRAM = 3,   // mode 3
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            gpu_vram: [0; 0x2000],
            sprite_table: [0; 0xA0],
            sprite_palette: [
                [PALETTE[0], PALETTE[1], PALETTE[2], PALETTE[3]],
                [PALETTE[0], PALETTE[1], PALETTE[2], PALETTE[3]],
            ],

            frame_buffer: [0; 160 * 144],
            draw_flag: false,

            // ppu
            tileset: [[[0; 8]; 8]; 384],
            bg_palette: [PALETTE[0], PALETTE[1], PALETTE[2], PALETTE[3]],

            mode: PpuMode::OAM,
            window_internal_line_counter: 0,

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

            line_clock_cycles: 0,
            mode_clock_cycles: 0,
        }
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        &self.frame_buffer
    }

    pub fn consume_draw_flag(&mut self) -> bool {
        let flag = self.draw_flag;
        self.draw_flag = false;
        flag
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
                    self.bg_palette[i] = PALETTE[((val >> (i * 2)) & 3) as usize];
                }
            }
            0xFF48 => {
                self.obp0 = val;
                for i in 0..4 {
                    self.sprite_palette[0][i] = PALETTE[((val >> (i * 2)) & 3) as usize];
                }
            }
            0xFF49 => {
                self.obp1 = val;
                for i in 0..4 {
                    self.sprite_palette[1][i] = PALETTE[((val >> (i * 2)) & 3) as usize];
                }
            }
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,

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

    fn get_bg_map_start_addr(&self) -> u16 {
        match (self.lcdc & LcdControlFlag::BGTileMapAddress as u8) != 0 {
            true => 0x9C00,
            false => 0x9800,
        }
    }

    fn get_adjusted_tile_index(&self, addr: u16, signed_tile_index: bool) -> u16 {
        let addr = (addr - 0x8000) as usize;
        if signed_tile_index {
            let tile = self.gpu_vram[addr] as i8 as i16;
            if tile >= 0 {
                tile as u16 + 256
            } else {
                256 - (tile.abs() as u16)
            }
        } else {
            self.gpu_vram[addr] as u16
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
        let mut scan_line_row: [u8; 160] = [0; 160];

        if self.lcdc & LcdControlFlag::BGEnable as u8 != 0 {
            self.draw_background(&mut scan_line_row);
        }
    }

    fn draw_background(&mut self, scan_line_row: &mut [u8; 160]) {
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

        // above x and y are where we are relative to the whole bg map (256 * 256)
        // we need x and y to be relative to the tile we want to draw
        // so modulo by 8 (or & 7)
        // shadow over old x an y variables
        let tile_local_y = y & 7;
        let mut tile_local_x = x & 7;

        let mut tile_address = (tile_index * 16) + (tile_local_y as u16 * 2);
        let mut b1 = self.gpu_vram[tile_address as usize];
        let mut b2 = self.gpu_vram[tile_address as usize + 1];

        let mut frame_buffer_offset = self.ly as usize * 160;

        let mut pixels_drawn_for_current_tile: u8 = 0;
        for i in 0..160 {
            let bx = 7 - tile_local_x;
            let color_bit = ((b1 & (1 << bx)) >> bx) | ((b2 & (1 << bx)) >> bx) << 1;
            let color = self.bg_palette[color_bit as usize];

            scan_line_row[i] = color;
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

                tile_x = x / 8;
                tile_map_offset = (tile_y as u16 * 32) + tile_x as u16;
                tile_index = self.get_adjusted_tile_index(
                    bg_map_start_addr + tile_map_offset,
                    signed_tile_addressing,
                );

                tile_address = (tile_index * 16) + (tile_local_y as u16 * 2);
                b1 = self.gpu_vram[tile_address as usize];
                b2 = self.gpu_vram[tile_address as usize + 1];
            }
        }
    }
}
