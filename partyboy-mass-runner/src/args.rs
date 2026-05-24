use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub roms_dir: PathBuf,

    #[arg(short, long)]
    pub output: PathBuf,

    #[arg(short, long)]
    pub bios: Option<PathBuf>,

    /// Target emulation speed as a multiplier of real time (e.g. 3.0 = 3x real-time).
    /// Values <= 0 mean unlimited / original max-speed behavior.
    #[arg(long, default_value_t = 3.0)]
    pub speed_factor: f64,

    /// Resume a previous run. ROMs with both `{name}_40.png` and `{name}_120.png`,
    /// or a `{name}.failed` marker, are skipped.
    #[arg(long)]
    pub resume: bool,

    /// Number of concurrent emulator workers (default: logical CPUs, capped at 16).
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,
}
