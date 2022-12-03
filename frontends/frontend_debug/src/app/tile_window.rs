use eframe::{
    egui::{self, Layout, Ui},
    emath::Align,
    epaint::{Color32, ColorImage},
};
use egui_extras::{Size, TableBuilder};
use gameboy::debug::GBTile;

use super::DebuggerApp;

const TILE_SIZE: f32 = 15.0;
const PALETTE: [Color32; 4] = [
    Color32::from_rgb(255, 255, 255),
    Color32::from_rgb(192, 192, 192),
    Color32::from_rgb(96, 96, 96),
    Color32::from_rgb(0, 0, 0),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TileBankState {
    Bank0,
    Bank1,
}

impl DebuggerApp {
    pub fn show_tile_window(&mut self, ctx: &egui::Context) {
        if !self.toggle_state.tile {
            return;
        }

        egui::Window::new("Tiles")
            .default_size([300.0, 600.0])
            .resizable(false)
            .show(ctx, |ui| {
                self.render_tile_window_display(ctx, ui);
            });
    }

    fn render_tile_window_display(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.toggle_state.tile_bank,
                TileBankState::Bank0,
                "Bank 0",
            );
            ui.selectable_value(
                &mut self.toggle_state.tile_bank,
                TileBankState::Bank1,
                "Bank 1",
            );
        });

        ui.separator();

        match self.toggle_state.tile_bank {
            TileBankState::Bank0 => {
                ui.push_id("bank_0_tiles", |ui| {
                    Self::render_bg_tile_table(ctx, ui, &self.gb_debug_info.tiles.bank_0)
                });
            }
            TileBankState::Bank1 => {
                ui.push_id("bank_1_tiles", |ui| {
                    Self::render_bg_tile_table(ctx, ui, &self.gb_debug_info.tiles.bank_1)
                });
            }
        }
    }

    fn render_bg_tile_table(ctx: &egui::Context, ui: &mut Ui, tiles: &[GBTile]) {
        TableBuilder::new(ui)
            .cell_layout(Layout::left_to_right().with_cross_align(Align::Center))
            .columns(
                Size::Absolute {
                    initial: TILE_SIZE,
                    range: (TILE_SIZE, TILE_SIZE),
                },
                16,
            )
            .resizable(false)
            .body(|body| {
                let mut tile_idx = 0;
                body.rows(TILE_SIZE, 24, |_, mut row| {
                    for _ in 0..16 {
                        let tile_row = &tiles[tile_idx].data;
                        let image = tile_into_image(tile_row);
                        let texture = ctx.load_texture(
                            format!("tile-{}", tile_idx),
                            image,
                            egui::TextureFilter::Nearest,
                        );

                        row.col(|ui| {
                            ui.image(&texture, [TILE_SIZE, TILE_SIZE]);
                        });

                        tile_idx += 1;
                    }
                })
            })
    }
}

fn tile_into_image(tile: &[[u8; 8]; 8]) -> ColorImage {
    let mut pixels: Vec<Color32> = Vec::with_capacity(64);
    for row in tile {
        for col_idx in row {
            pixels.push(PALETTE[*col_idx as usize]);
        }
    }
    ColorImage {
        size: [8, 8],
        pixels,
    }
}
