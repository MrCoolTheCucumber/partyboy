use crate::CYCLE_COUNT;

use super::DebuggerApp;
use eframe::egui::{self, Ui};

impl DebuggerApp {
    pub fn show_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("general_info").show(ctx, |ui| {
            self.render_side_panel_display(ui);
        });
    }

    fn render_side_panel_display(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("PartyBoy");
        });

        ui.separator();

        self.fps = self.gb_debug_info.fps.unwrap_or(self.fps);
        ui.label(format!("FPS: {:.2}", self.fps));
        ui.label(format!("Cycles: {}", unsafe { CYCLE_COUNT }));

        ui.separator();

        let ppu_info = &self.gb_debug_info.ppu_info;
        ui.label(format!("scx: {}", ppu_info.scx));
        ui.label(format!("scy: {}", ppu_info.scy));
        ui.label(format!("stat: {:#010b}", ppu_info.stat));
        ui.label(format!("lcdc: {:#010b}", ppu_info.lcdc));
    }
}
