use eframe::egui;
use rfd::FileDialog;

use crate::{DebuggerApp, MessageToGB};

impl DebuggerApp {
    pub(super) fn show_menu(&mut self, ui: &mut egui::Ui) {
        use egui::menu;

        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    self.handle_open();
                }
            });

            ui.separator();

            if ui.button("Start").clicked() {
                _ = self.to_gb_tx.send(MessageToGB::Start);
            }

            if ui.button("Stop").clicked() {
                _ = self.to_gb_tx.send(MessageToGB::Stop);
            }

            ui.separator();

            ui.toggle_value(&mut self.toggle_state.log, "Log");
            ui.toggle_value(&mut self.toggle_state.palletes, "Palletes");
            ui.toggle_value(&mut self.toggle_state.tile, "Tiles");
        });
    }

    fn handle_open(&mut self) {
        if let Some(rom_path) = FileDialog::new().pick_file() {
            let _ = self
                .to_gb_tx
                .send(MessageToGB::New(rom_path.to_str().unwrap().to_owned()));
        }
    }
}
