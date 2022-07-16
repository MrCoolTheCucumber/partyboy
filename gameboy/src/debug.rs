#![allow(dead_code)]

use crate::{bus::CgbCompatibility, ppu::rgb::Rgb, GameBoy};

#[derive(Default)]
pub struct GBDebugInfo {
    pub fps: Option<f64>,
    pub compatibility_mode: CgbCompatibility,
    pub palette: GBPalleteData,
    pub tiles: GBTileData,
    pub ppu_info: GBPpuInfo,
    pub map_data: GBMapInfo,
    pub tile_attr_data: GBTileAttributeInfo,
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

#[derive(Default)]
pub struct GBPpuInfo {
    pub lcdc: u8,
    pub stat: u8,
    pub scy: u8,
    pub scx: u8,
}

#[allow(non_snake_case)]
pub struct GBMapInfo {
    pub bg_map_9800: [u8; 32 * 32],
    pub bg_map_9C00: [u8; 32 * 32],
}

impl Default for GBMapInfo {
    fn default() -> Self {
        let default_map = [0; 32 * 32];
        Self {
            bg_map_9800: default_map.clone(),
            bg_map_9C00: default_map,
        }
    }
}

#[allow(non_snake_case)]
pub struct GBTileAttributeInfo {
    pub tile_attr_9800: [u8; 32 * 32],
    pub tile_attr_9C00: [u8; 32 * 32],
}

impl Default for GBTileAttributeInfo {
    fn default() -> Self {
        let default_attr = [0; 32 * 32];
        Self {
            tile_attr_9800: default_attr.clone(),
            tile_attr_9C00: default_attr,
        }
    }
}

impl GameBoy {
    pub fn debug_info(&self) -> GBDebugInfo {
        GBDebugInfo {
            fps: None,
            compatibility_mode: self.bus().console_compatibility_mode,
            palette: self.color_palettes(),
            tiles: self.tile_data(),
            ppu_info: self.ppu_general(),
            map_data: self.map_data(),
            tile_attr_data: self.tile_attribute_data(),
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

    fn ppu_general(&self) -> GBPpuInfo {
        let ppu = &self.bus().ppu;
        GBPpuInfo {
            lcdc: ppu.lcdc,
            stat: ppu.stat,
            scx: ppu.scx,
            scy: ppu.scy,
        }
    }

    fn get_data_at_addr(&self, addr: usize, bank: usize) -> [u8; 32 * 32] {
        let ppu = &self.bus().ppu;
        let mut map = [0; 32 * 32];

        for i in 0..(32 * 32) {
            map[i] = ppu.gpu_vram[bank][addr + i - 0x8000]
        }

        map
    }

    fn map_data(&self) -> GBMapInfo {
        GBMapInfo {
            bg_map_9800: self.get_data_at_addr(0x9800, 0),
            bg_map_9C00: self.get_data_at_addr(0x9C00, 0),
        }
    }

    fn tile_attribute_data(&self) -> GBTileAttributeInfo {
        GBTileAttributeInfo {
            tile_attr_9800: self.get_data_at_addr(0x9800, 1),
            tile_attr_9C00: self.get_data_at_addr(0x9C00, 1),
        }
    }
}
