use common::compare_fb_to_img;
use common::APPROX_CYCLES_PER_SCREEN_DRAW;
use gameboy::GameBoy;
use std::path::PathBuf;

mod common;

fn get_test_rom_root_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("test/test_roms/");
    path
}

fn get_expected_root_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("test/test_expected/acid");
    path
}

#[test]
fn dmg_acid2() {
    let mut path = get_test_rom_root_path();
    path.push("dmg-acid2.gb");
    let path = path.to_str().unwrap();
    let rom = std::fs::read(path).unwrap();

    let mut gb = GameBoy::builder().rom(rom).build().unwrap();

    for _ in 0..APPROX_CYCLES_PER_SCREEN_DRAW * 60 * 5 {
        gb.tick();
    }

    let mut expected_path = get_expected_root_path();
    expected_path.push("dmg_acid2_reference_cgb.png");

    let fb = gb.get_frame_buffer();
    let are_equal = compare_fb_to_img(fb, expected_path.to_str().unwrap().to_owned());
    assert!(are_equal);
}

#[test]
fn cgb_acid2() {
    let mut path = get_test_rom_root_path();
    path.push("cgb-acid2.gb");
    let path = path.to_str().unwrap();
    let rom = std::fs::read(path).unwrap();

    let mut gb = GameBoy::builder().rom(rom).build().unwrap();

    for _ in 0..APPROX_CYCLES_PER_SCREEN_DRAW * 60 * 5 {
        gb.tick();
    }

    let mut expected_path = get_expected_root_path();
    expected_path.push("cgb_acid2_reference.png");

    let fb = gb.get_frame_buffer();
    let are_equal = compare_fb_to_img(fb, expected_path.to_str().unwrap().to_owned());
    assert!(are_equal);
}
