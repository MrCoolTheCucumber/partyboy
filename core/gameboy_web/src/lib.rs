pub use gameboy::input::Input;
pub use gameboy::ppu::rgb::Rgb;
pub use gameboy::GameBoy;

use wasm_bindgen::prelude::wasm_bindgen;

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
pub fn take_snapshot(gb: &mut GameBoy) -> Vec<u8> {
    rmp_serde::to_vec(&gb).unwrap()
}

#[wasm_bindgen]
pub fn load_snapshot(snapshot: Vec<u8>, gameboy: &mut GameBoy) {
    let snapshot: GameBoy = rmp_serde::from_slice(&snapshot).unwrap();
    gameboy.load_snapshot(snapshot);
}
