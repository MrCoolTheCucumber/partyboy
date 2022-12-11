use common::bitpacked::BitPackedState;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use wasm_bindgen::prelude::wasm_bindgen;

pub use gameboy::input::Input;
pub use gameboy::ppu::rgb::Rgb;
pub use gameboy::GameBoy;

pub use common;

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
pub fn take_snapshot(gb: &mut GameBoy) -> BitPackedState {
    let encoded = rmp_serde::to_vec(&gb).unwrap();
    let compressed = compress_prepend_size(&encoded);
    BitPackedState::pack(compressed)
}

#[wasm_bindgen]
pub fn load_snapshot(gb: &mut GameBoy, snapshot: &BitPackedState) {
    let unpacked = snapshot.unpack();
    let decompressed = decompress_size_prepended(&unpacked).unwrap();
    let state: GameBoy = rmp_serde::from_slice(&decompressed).unwrap();
    gb.load_snapshot(state);
    gb.release_all_keys();
}
