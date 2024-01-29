use std::{
    fs,
    path::{Path, PathBuf},
};

fn get_save_file_path(rom_path: &Path) -> Option<PathBuf> {
    let file_stem = rom_path.file_stem()?.to_str()?;
    let path = rom_path.parent()?;

    if !path.is_dir() {
        return None;
    }

    Some(path.join(format!("{file_stem}.sav")))
}

pub fn read_save_file(rom_path: &Path) -> Option<Vec<u8>> {
    let save_file_path = get_save_file_path(rom_path)?;
    fs::read(save_file_path).ok()
}

pub fn write_save_file(rom_path: &Path, bytes: &[u8]) {
    let Some(save_file_path) = get_save_file_path(rom_path) else {
        return;
    };

    let _ = fs::write(save_file_path, bytes);
    log::info!("written save file...");
}
