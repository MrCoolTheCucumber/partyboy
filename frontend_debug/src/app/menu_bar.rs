use std::path::PathBuf;

use eframe::egui;
use gameboy::builder::GameBoyBuilder;
use rfd::FileDialog;

use crate::DebugerApp;

pub struct RecentRomInfo {
    name: String,
    path: PathBuf,
}

impl RecentRomInfo {
    fn new(rom_path: PathBuf) -> Self {
        Self {
            name: rom_path.file_name().unwrap().to_str().unwrap().to_owned(),
            path: rom_path,
        }
    }
}

impl DebugerApp {
    pub(super) fn show_menu(&mut self, ui: &mut egui::Ui) {
        use egui::menu;

        menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    self.handle_open();
                }
            })
        });
    }

    fn handle_open(&mut self) {
        if let Some(rom_path) = FileDialog::new().pick_file() {
            let gb_result = GameBoyBuilder::new()
                .rom_path(rom_path.to_str().unwrap())
                .build();

            match gb_result {
                Ok(gb) => {
                    self.gameboy = Some(gb);
                    self.recent_roms.push(RecentRomInfo::new(rom_path));
                }
                Err(error) => log::error!("{}", error),
            }
        }
    }
}
