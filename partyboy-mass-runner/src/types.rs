use std::any::Any;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use partyboy_core::ppu::rgb::Rgb;

#[derive(Debug)]
pub enum RunResult {
    Success {
        fourty_seconds_frame_buffer: Vec<Rgb>,
        onetwenty_seconds_frame_buffer: Vec<Rgb>,
        rom_path: PathBuf,
    },
    Fail {
        rom_name: String,
        error: Box<dyn Any + Send>,
    },
}

/// Live status of one worker thread, polled by the UI driver to keep the per-worker bars
/// in sync without letting workers touch ProgressBar objects directly.
pub struct WorkerStatus {
    rom_name: Mutex<Option<String>>,
    emulated_seconds: AtomicU64,
}

impl WorkerStatus {
    pub fn new() -> Self {
        Self {
            rom_name: Mutex::new(None),
            emulated_seconds: AtomicU64::new(0),
        }
    }

    pub fn set_rom(&self, name: Option<String>) {
        *self.rom_name.lock().unwrap() = name;
        self.emulated_seconds.store(0, Ordering::Relaxed);
    }

    pub fn update_emulated(&self, seconds: u64) {
        self.emulated_seconds.store(seconds, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> (Option<String>, u64) {
        let name = self.rom_name.lock().unwrap().clone();
        (name, self.emulated_seconds.load(Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownState {
    Running,
    Draining,
    ForceKill,
}

pub type Shutdown = Arc<AtomicU8>;

pub fn new_shutdown() -> Shutdown {
    Arc::new(AtomicU8::new(ShutdownState::Running as u8))
}

pub fn request_shutdown(shutdown: &Shutdown, state: ShutdownState) {
    shutdown.store(state as u8, Ordering::SeqCst);
}

pub fn current_shutdown(shutdown: &Shutdown) -> ShutdownState {
    match shutdown.load(Ordering::SeqCst) {
        0 => ShutdownState::Running,
        1 => ShutdownState::Draining,
        _ => ShutdownState::ForceKill,
    }
}
