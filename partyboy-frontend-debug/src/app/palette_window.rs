use eframe::{
    egui::{self, Layout, Sense, Ui},
    emath::{Align, Vec2},
    epaint::Color32,
};
use egui_extras::{Size, TableBuilder};
use gameboy::{debug::CgbCompatibility, ppu::rgb::Rgb};

use super::DebuggerApp;

const PALETTE_SIZE: f32 = 15.0;

impl DebuggerApp {
    pub(super) fn show_palette_window(&mut self, ctx: &egui::Context) {
        if !self.toggle_state.palletes {
            return;
        }

        egui::Window::new("Palettes")
            .default_width(200.0)
            .resizable(false)
            .show(ctx, |ui| {
                self.render_palette_window_display(ctx, ui);
            });
    }

    fn render_palette_window_display(&self, _: &egui::Context, ui: &mut Ui) {
        ui.columns(2, |cols| {
            cols[0].heading("BG");
            let bg_palette_data = match self.gb_debug_info.compatibility_mode {
                CgbCompatibility::CgbOnly => self.gb_debug_info.palette.bg,
                _ => {
                    let mut data = self.gb_debug_info.palette.bg;
                    data[0] = self
                        .gb_debug_info
                        .palette
                        .bg_palette_dmg_map
                        .map(|idx| data[0][idx]);
                    data
                }
            };
            Self::render_palette_table(&mut cols[0], bg_palette_data, "bg");

            cols[1].heading("Obj");
            let sprite_palette_data = match self.gb_debug_info.compatibility_mode {
                CgbCompatibility::CgbOnly => self.gb_debug_info.palette.bg,
                _ => {
                    let mut data = self.gb_debug_info.palette.bg;
                    data[0] = self.gb_debug_info.palette.sprite_palette_dmg_map[0]
                        .map(|idx| data[0][idx]);
                    data[1] = self.gb_debug_info.palette.sprite_palette_dmg_map[1]
                        .map(|idx| data[1][idx]);
                    data
                }
            };
            Self::render_palette_table(&mut cols[1], sprite_palette_data, "obj");
        });
    }

    fn render_palette_table(ui: &mut Ui, palette_data: [[Rgb; 4]; 8], id: &str) {
        ui.push_id(id, |ui| {
            TableBuilder::new(ui)
                .cell_layout(Layout::left_to_right().with_cross_align(Align::Center))
                .resizable(false)
                .column(Size::Absolute {
                    initial: PALETTE_SIZE,
                    range: (PALETTE_SIZE, PALETTE_SIZE),
                })
                .column(Size::Absolute {
                    initial: PALETTE_SIZE,
                    range: (PALETTE_SIZE, PALETTE_SIZE),
                })
                .column(Size::Absolute {
                    initial: PALETTE_SIZE,
                    range: (PALETTE_SIZE, PALETTE_SIZE),
                })
                .column(Size::Absolute {
                    initial: PALETTE_SIZE,
                    range: (PALETTE_SIZE, PALETTE_SIZE),
                })
                .body(|body| {
                    body.rows(PALETTE_SIZE, 8, |row_idx, mut row| {
                        for palette_col in palette_data[row_idx] {
                            row.col(|ui| {
                                let (rect, _) = ui.allocate_at_least(
                                    Vec2::new(PALETTE_SIZE, PALETTE_SIZE),
                                    Sense::hover(),
                                );
                                let color =
                                    Color32::from_rgb(palette_col.r, palette_col.g, palette_col.b);
                                ui.painter().rect_filled(rect, 0.0, color);
                            });
                        }
                    })
                });
        });
    }
}
