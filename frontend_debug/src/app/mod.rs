use crossbeam::channel::{Receiver, Sender};
use eframe::egui;
use gameboy::ppu::rgb::Rgb;

use crate::{channel_log::Log, MessageFromGb, MessageToGB};

mod gb_display;
mod log_window;
mod menu_bar;

pub struct DebugerApp {
    gb_frame_buffer: Option<Vec<Rgb>>,
    logs: Vec<Log>,

    log_rx: Receiver<Log>,
    to_gb_tx: Sender<MessageToGB>,
    from_gb_rx: Receiver<MessageFromGb>,
}

impl DebugerApp {
    pub fn new(
        _: &eframe::CreationContext,
        log_rx: Receiver<Log>,
        to_gb_tx: Sender<MessageToGB>,
        from_gb_rx: Receiver<MessageFromGb>,
    ) -> Self {
        Self {
            gb_frame_buffer: None,
            logs: Vec::new(),
            log_rx,
            to_gb_tx,
            from_gb_rx,
        }
    }
}

impl eframe::App for DebugerApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        if let Ok(msg) = self.from_gb_rx.try_recv() {
            match msg {
                MessageFromGb::Draw(fb) => self.gb_frame_buffer = Some(fb),
            }
        }

        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            self.show_menu(ui);
        });
        self.show_log_window(ctx);
        self.show_gb_display_window(ctx);
        egui::CentralPanel::default().show(ctx, |_| {});
    }
}
