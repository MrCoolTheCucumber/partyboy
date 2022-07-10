#![allow(dead_code)]

use crate::{ppu::rgb::Rgb, GameBoy};

pub struct BgData;

#[derive(Default)]
pub struct GBDebugInfo {
    pub fps: Option<f64>,
    pub palette: GBPalleteData,
}

#[derive(Default)]
pub struct GBPalleteData {
    pub bg: [[Rgb; 4]; 8],
    pub sprite: [[Rgb; 4]; 8],
}

impl GameBoy {
    pub fn debug_info(&self) -> GBDebugInfo {
        GBDebugInfo {
            fps: None,
            palette: self.color_palettes(),
        }
    }

    fn color_palettes(&self) -> GBPalleteData {
        let ppu = &self.bus().ppu;
        GBPalleteData {
            bg: ppu.bg_color_palette(),
            sprite: ppu.sprite_color_palette(),
        }
    }
}
