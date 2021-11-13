const PALETTE: [u8; 4] = [255, 192, 96, 0];

pub struct Ppu {
    gpu_vram: [u8; 0x2000],
    sprite_table: [u8; 0xA0],
    sprite_palette: [[u8; 4]; 2],

    tileset: [[[u8; 8]; 8]; 384],
    bg_palette: [u8; 4],
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

            // ppu
            tileset: [[[0; 8]; 8]; 384],
            bg_palette: [PALETTE[0], PALETTE[1], PALETTE[2], PALETTE[3]],
        }
    }
}
