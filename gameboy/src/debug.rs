#![allow(dead_code)]

use crate::{ppu::rgb::Rgb, GameBoy};

#[derive(Default)]
pub struct GBDebugInfo {
    pub fps: Option<f64>,
    pub palette: GBPalleteData,
    pub tiles: GBTileData,
}

#[derive(Default)]
pub struct GBPalleteData {
    pub bg: [[Rgb; 4]; 8],
    pub sprite: [[Rgb; 4]; 8],
}

#[derive(Default, Clone, Copy)]
pub struct GBTile {
    pub data: [[u8; 8]; 8],
}

pub struct GBTileData {
    pub bank_0: Vec<GBTile>,
    pub bank_1: Vec<GBTile>,
}

impl Default for GBTileData {
    fn default() -> Self {
        let default_bank = [GBTile::default(); 384];
        Self {
            bank_0: default_bank.to_vec(),
            bank_1: default_bank.to_vec(),
        }
    }
}

impl GameBoy {
    pub fn debug_info(&self) -> GBDebugInfo {
        GBDebugInfo {
            fps: None,
            palette: self.color_palettes(),
            tiles: self.tile_data(),
        }
    }

    fn color_palettes(&self) -> GBPalleteData {
        let ppu = &self.bus().ppu;
        GBPalleteData {
            bg: ppu.bg_color_palette(),
            sprite: ppu.sprite_color_palette(),
        }
    }

    // Tile data is stored in 0x8000 - 0x97FF
    // if in CGB mode then vram bank [1] also has
    // tiles in that location

    // in vram bank [0], 0x9800 - 0x9BFF and 0x9C00 - 0x9FFF
    // contain tile maps

    // in vram bank [1], 0x9800 - 0x9FFF contain attributes for the
    // to apply to the corresponding tile map at vram bank [0]

    fn get_tile_data_from_bank(&self, bank: usize) -> Vec<GBTile> {
        let ppu = &self.bus().ppu;
        let mut bank_tiles = Vec::new();

        let mut index = 0x8000;
        while index < 0x9800 {
            let mut tile_data_index = 0;
            let mut tile_row_index = 0;
            let mut tile = GBTile::default();

            while tile_data_index < 16 {
                let b1 = ppu.gpu_vram[bank][index + tile_data_index - 0x8000];
                let b2 = ppu.gpu_vram[bank][index + tile_data_index + 1 - 0x8000];

                let mut tile_row = [0u8; 8];
                for i in (0..8usize).rev() {
                    let shift_i = i as u8;
                    tile_row[7 - i] = ((b1 & (1 << shift_i)) >> shift_i)
                        | ((b2 & (1 << shift_i)) >> shift_i) << 1;
                }

                tile.data[tile_row_index] = tile_row;

                tile_data_index += 2;
                tile_row_index += 1;
            }

            bank_tiles.push(tile);
            index += 16;
        }

        bank_tiles
    }

    fn tile_data(&self) -> GBTileData {
        GBTileData {
            bank_0: self.get_tile_data_from_bank(0),
            bank_1: self.get_tile_data_from_bank(1),
        }
    }

    // we are already sending all of the tile and palette data,
    // so technically we can just send all the map data and
    // reconstruct the maps with that

    fn map_data(&self) {
        todo!()
    }
}
