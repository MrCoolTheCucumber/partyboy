#![allow(dead_code)]

use gameboy::ppu::rgb::Rgb;
use image::Rgb as iRGB;

pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;
pub const APPROX_CYCLES_PER_SCREEN_DRAW: u64 = 70_224;

fn is_px_eq(fb_px: Rgb, img_px: &iRGB<u8>) -> bool {
    img_px.0[0] == fb_px.r && img_px.0[1] == fb_px.g && img_px.0[2] == fb_px.b
}

pub fn compare_fb_to_img(fb: &[Rgb], path: String) -> bool {
    let img = image::io::Reader::open(path).unwrap().decode().unwrap();
    let img = img.as_rgb8().unwrap();

    for px in img.enumerate_pixels() {
        let fb_px = fb[((px.1 * WIDTH) + px.0) as usize];
        if !is_px_eq(fb_px, px.2) {
            return false;
        }
    }

    true
}
