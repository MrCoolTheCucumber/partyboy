#![allow(dead_code)]

use std::env;

use gameboy::GameBoy;

mod gameboy;

fn main() {
    #[cfg(debug_assertions)]
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug")
    }

    env_logger::init();
    log_panics::init();

    let mut gb = GameBoy::new("/mnt/i/Dev/gb-rs/tetris.gb");
    log::info!("Initialized gameboy.");

    loop {
        gb.tick();
    }
}
