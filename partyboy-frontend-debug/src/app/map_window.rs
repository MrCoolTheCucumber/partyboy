use eframe::{
    egui::{self, Ui},
    epaint::{Color32, ColorImage},
};
use partyboy_core::{
    debug::{GBPalleteData, GBPpuInfo},
    ppu::rgb::Rgb,
};

use super::DebuggerApp;

struct BGMapFlags {
    vertical_flip: bool,
    horizontal_flip: bool,
    tile_bank: usize,
    bg_palette_number: usize,
}

impl From<&u8> for BGMapFlags {
    fn from(val: &u8) -> Self {
        BGMapFlags {
            vertical_flip: val & 0b0100_0000 != 0,
            horizontal_flip: val & 0b0010_0000 != 0,
            tile_bank: ((val & 0b000_1000) >> 3) as usize,
            bg_palette_number: (val & 0b0000_0111) as usize,
        }
    }
}

fn into_unsigned_tile_addressing(idx: u8) -> usize {
    idx as usize
}

fn into_signed_tile_addressing(idx: u8) -> usize {
    let tile = idx as i8 as i16;
    if tile >= 0 {
        tile as usize + 256
    } else {
        256 - (tile.unsigned_abs() as usize)
    }
}

impl DebuggerApp {
    pub(super) fn show_map_window(&mut self, ctx: &egui::Context) {
        if !self.toggle_state.maps {
            return;
        }

        egui::Window::new("BG Map")
            .default_width(200.0)
            .resizable(false)
            .show(ctx, |ui| {
                self.render_map_window_display(ctx, ui);
            });
    }

    fn render_map_window_display(&self, ctx: &egui::Context, ui: &mut Ui) {
        let ppu_info = &self.gb_debug_info.ppu_info;

        let signed_tile_addressing = ppu_info.lcdc & 0b0001_0000 == 0;
        let (bg_map, attr_data) = if ppu_info.lcdc & 0b0000_1000 != 0 {
            (
                &self.gb_debug_info.map_data.bg_map_9C00,
                &self.gb_debug_info.tile_attr_data.tile_attr_9C00,
            )
        } else {
            (
                &self.gb_debug_info.map_data.bg_map_9800,
                &self.gb_debug_info.tile_attr_data.tile_attr_9800,
            )
        };

        let map_addr = if signed_tile_addressing {
            into_signed_tile_addressing
        } else {
            into_unsigned_tile_addressing
        };

        let tiles = &self.gb_debug_info.tiles;

        // iterate over the tile map
        // TODO: also iterate over the bg map attributes if we are
        // in cgb mode
        let tile_map_mapped = bg_map
            .iter()
            .zip(attr_data)
            .map(|(tile, attr_raw)| {
                let mapped_idx = map_addr(*tile);
                let attr_flags: BGMapFlags = attr_raw.into();
                let bank = if attr_flags.tile_bank == 0 {
                    &tiles.bank_0
                } else {
                    &tiles.bank_1
                };
                let tile_data = bank[mapped_idx].data;

                tile_into_image(
                    &tile_data,
                    attr_flags,
                    &self.gb_debug_info.palette,
                    self.gb_debug_info.compatibility_mode.is_cgb_mode(),
                )
            })
            .collect::<Vec<_>>();

        // lets build a 32 * 32 tile ColorImage
        // which will be the contents of the whole map

        let whole_img_pixels = [Color32::WHITE; (32 * 8) * (32 * 8)].to_vec();
        let mut bg_img = ColorImage {
            pixels: whole_img_pixels,
            size: [32 * 8, 32 * 8],
        };

        for y in 0..32 {
            for x in 0..32 {
                let idx = (y * 32) + x;
                let tile = &tile_map_mapped[idx];

                let x1 = x * 8;
                let y1 = y * 8;

                for j in 0..8 {
                    for i in 0..8 {
                        bg_img[(x1 + i, y1 + j)] = tile[(i, j)];
                    }
                }
            }
        }

        // draw bounds on the bg_img
        // screen is 160(w) * 144(h) px
        // 20 tiles * 18 tiles

        let GBPpuInfo { scx, scy, .. } = ppu_info;
        draw_square(&mut bg_img, *scy as usize, *scx as usize, 20 * 8, 18 * 8);

        let texture = ctx.load_texture("map", bg_img, egui::TextureFilter::Nearest);
        ui.image(&texture, [32.0 * 8.0, 32.0 * 8.0]);
    }
}

fn draw_square(img: &mut ColorImage, top: usize, left: usize, width: usize, height: usize) {
    let right_side_x = (left + width) % (32 * 8);
    for i in top..(top + height) {
        let i = i % (32 * 8);

        img[(left, i)] = Color32::RED;
        img[(right_side_x, i)] = Color32::RED;
    }

    let bottom_side_y = (top + height) % (32 * 8);
    for i in left..(left + width) {
        let i = i % (32 * 8);

        img[(i, top)] = Color32::RED;
        img[(i, bottom_side_y)] = Color32::RED;
    }
}

fn into_color32(rgb: &Rgb) -> Color32 {
    Color32::from_rgb(rgb.r, rgb.g, rgb.b)
}

fn tile_into_image(
    tile: &[[u8; 8]; 8],
    attr: BGMapFlags,
    palettes: &GBPalleteData,
    is_cgb_mode: bool,
) -> ColorImage {
    let palette = if is_cgb_mode {
        &palettes.bg[attr.bg_palette_number]
    } else {
        &palettes.bg[0]
    };

    let mut pixels: Vec<Color32> = Vec::with_capacity(64);
    for row in tile {
        for col_idx in row {
            let px = into_color32(&palette[*col_idx as usize]);
            pixels.push(px);
        }
    }
    let mut img = ColorImage {
        size: [8, 8],
        pixels,
    };

    if is_cgb_mode {
        if attr.horizontal_flip {
            for j in 0..8 {
                for i in 0..8 {
                    (img[(j, 7 - i)], img[(j, i)]) = (img[(j, i)], img[(j, 7 - i)]);
                }
            }
        }

        // not 100% sure if this is correct
        if attr.vertical_flip {
            for j in 0..8 {
                for i in 0..8 {
                    (img[(7 - j, i)], img[(j, i)]) = (img[(j, i)], img[(7 - j, i)]);
                }
            }
        }
    }

    img
}
