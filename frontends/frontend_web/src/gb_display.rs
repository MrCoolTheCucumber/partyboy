use eframe::{
    egui,
    epaint::{Color32, ColorImage},
};
use gameboy::ppu::rgb::Rgb;

use crate::TemplateApp;

pub const DEFAULT_SCALE: f32 = 2.0;
pub const WIDTH: f32 = 160.0;
pub const HEIGHT: f32 = 144.0;
pub const SCALED_VEC_SIZE: usize =
    (WIDTH * DEFAULT_SCALE) as usize * (HEIGHT * DEFAULT_SCALE) as usize;

impl TemplateApp {
    pub(super) fn show_gb_display_window(&self, ctx: &egui::Context) {
        egui::Window::new("GB Display")
            .default_size([HEIGHT * DEFAULT_SCALE, WIDTH * DEFAULT_SCALE])
            .resizable(false)
            .show(ctx, |ui| {
                self.render_gb_window_display(ctx, ui);
            });
    }

    fn into_color_image(fb: Vec<Rgb>) -> ColorImage {
        let pixels: Vec<Color32> = fb
            .iter()
            .map(|px| Color32::from_rgb(px.r, px.g, px.b))
            .collect();

        ColorImage {
            size: [WIDTH as usize, HEIGHT as usize],
            pixels,
        }
    }

    fn blank_image() -> ColorImage {
        let pixels: [Color32; SCALED_VEC_SIZE] = [Color32::WHITE; SCALED_VEC_SIZE];
        ColorImage {
            size: [
                (WIDTH * DEFAULT_SCALE) as usize,
                (HEIGHT * DEFAULT_SCALE) as usize,
            ],
            pixels: pixels.to_vec(),
        }
    }

    fn render_gb_window_display(&self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(fb) = self.gb_frame_buffer() {
            let image = Self::into_color_image(fb);
            let texture = ctx.load_texture("gb_display", image, egui::TextureFilter::Nearest);
            ui.image(&texture, texture.size_vec2() * 2.0);
        } else {
            let image = Self::blank_image();
            let texture = ctx.load_texture("blank", image, egui::TextureFilter::Nearest);
            ui.image(&texture, texture.size_vec2());
        }
    }
}
