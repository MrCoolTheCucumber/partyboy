use super::DebuggerApp;
use eframe::egui::{self, Ui};

impl DebuggerApp {
    pub fn show_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("general_info").show(ctx, |ui| {
            self.render_side_panel_display(ui);
        });
    }

    fn render_side_panel_display(&mut self, ui: &mut Ui) {
        ui.heading("PartyBoy Debug");

        ui.separator();

        self.fps = self.gb_debug_info.fps.unwrap_or(self.fps);
        ui.label(format!("FPS: {:.2}", self.fps));
    }
}
