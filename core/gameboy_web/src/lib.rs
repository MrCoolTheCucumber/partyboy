pub use gameboy::input::Input;
pub use gameboy::ppu::rgb::Rgb;
pub use gameboy::GameBoy;

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn take_snapshot(gb: &mut GameBoy) -> Vec<u8> {
    rmp_serde::to_vec(&gb).unwrap()
}

#[wasm_bindgen]
pub fn load_snapshot(snapshot: Vec<u8>, gameboy: &mut GameBoy) {
    let snapshot: GameBoy = rmp_serde::from_slice(&snapshot).unwrap();
    gameboy.load_snapshot(snapshot);
}
