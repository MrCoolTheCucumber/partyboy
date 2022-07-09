use crossbeam::channel::{Receiver, Sender};
use eframe::egui;
use gameboy::input::Keycode;
use gameboy::ppu::rgb::Rgb;

use crate::{channel_log::Log, MessageFromGb, MessageToGB};

mod gb_display;
mod log_window;
mod menu_bar;

const KEYS: [egui::Key; 8] = [
    egui::Key::W,
    egui::Key::A,
    egui::Key::S,
    egui::Key::D,
    egui::Key::O,
    egui::Key::K,
    egui::Key::M,
    egui::Key::N,
];

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

    fn handle_input(&self, ctx: &eframe::egui::Context) {
        let input = ctx.input();
        let key_downs = input
            .keys_down
            .iter()
            .filter_map(|key| Self::into_gb_keycode(key))
            .collect::<Vec<_>>();
        let _ = self.to_gb_tx.send(MessageToGB::KeyDown(key_downs));

        let key_ups = KEYS
            .iter()
            .filter(|key| input.key_released(**key))
            .filter_map(|key| Self::into_gb_keycode(key))
            .collect::<Vec<_>>();
        let _ = self.to_gb_tx.send(MessageToGB::KeyUp(key_ups));
    }

    fn into_gb_keycode(key: &egui::Key) -> Option<Keycode> {
        match key {
            egui::Key::W => Some(Keycode::Up),
            egui::Key::A => Some(Keycode::Left),
            egui::Key::S => Some(Keycode::Down),
            egui::Key::D => Some(Keycode::Right),
            egui::Key::O => Some(Keycode::A),
            egui::Key::K => Some(Keycode::B),
            egui::Key::M => Some(Keycode::Start),
            egui::Key::N => Some(Keycode::Select),
            _ => None,
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

        self.handle_input(ctx);

        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            self.show_menu(ui);
        });
        self.show_gb_display_window(ctx);
        self.show_log_window(ctx);

        // TODO:
        // - Tile/Map/Sprite viewer
        // - Memory Viewer
        // - Dissassembly
        // - Rom selector?
        // - ppu event viewer
        // - SideBar of general info? (speed mode, are we in hdma, fps, etc)

        egui::CentralPanel::default().show(ctx, |_| {});

        ctx.request_repaint();
    }
}
