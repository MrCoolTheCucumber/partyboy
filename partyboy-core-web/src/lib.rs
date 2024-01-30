use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use wasm_bindgen::prelude::*;

pub use partyboy_core::input::Input;
pub use partyboy_core::ppu::rgb::Rgb;
pub use partyboy_core::GameBoy;

pub use partyboy_common as common;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Run the tick loop in wasm. If draw flag is consumed then
/// it returns
///
/// returns remaining ticks if stopped due to draw flag being consumed
#[wasm_bindgen]
pub fn batch_ticks(gb: &mut GameBoy, ticks: u64) -> u64 {
    for i in 0..ticks {
        gb.tick();
        if gb.consume_draw_flag() {
            return ticks - i;
        }
    }

    0
}

#[wasm_bindgen]
pub fn handle_ticks(gb: &mut GameBoy) -> Vec<f32> {
    let mut samples = Vec::new();

    while samples.len() < 512 {
        if let Some(frame) = gb.tick() {
            samples.push(frame[0]);
            samples.push(frame[1]);
        }
    }

    samples
}

#[wasm_bindgen]
pub fn take_snapshot(gb: &mut GameBoy) -> Vec<u8> {
    let encoded = rmp_serde::to_vec(&gb).unwrap();
    compress_prepend_size(&encoded)
}

#[wasm_bindgen]
pub fn load_snapshot(gb: &mut GameBoy, snapshot: &[u8]) {
    let decompressed = decompress_size_prepended(snapshot).unwrap();
    let state: GameBoy = rmp_serde::from_slice(&decompressed).unwrap();
    gb.load_snapshot(state);
    gb.release_all_keys();
}
