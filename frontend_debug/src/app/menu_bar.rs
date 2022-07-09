use eframe::egui;
use rfd::FileDialog;

use crate::{DebugerApp, MessageToGB};

impl DebugerApp {
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
