use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

pub struct Rom {
    pub bytes: Vec<u8>,
    pub path: PathBuf,
}

pub fn rom_display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

pub fn get_all_roms(path: &Path) -> Result<Vec<Rom>> {
    fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                return None;
            }
            Some(entry.path())
        })
        .map(|path| Ok(Rom { bytes: fs::read(&path)?, path }))
        .collect()
}

/// Skip ROMs already processed by a previous run: those with both success screenshots
/// (`{stem}_40.png` + `{stem}_120.png`) or a recorded failure marker (`{stem}.failed`).
pub fn filter_completed_for_resume(mut roms: Vec<Rom>, output_dir: &Path) -> Vec<Rom> {
    roms.retain(|rom| {
        let Some(stem) = rom.path.file_stem().and_then(|s| s.to_str()) else {
            return true;
        };
        let p40 = output_dir.join(format!("{stem}_40.png"));
        let p120 = output_dir.join(format!("{stem}_120.png"));
        let failed = output_dir.join(format!("{stem}.failed"));
        !((p40.exists() && p120.exists()) || failed.exists())
    });
    roms
}
