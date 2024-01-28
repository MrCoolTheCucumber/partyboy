use crossbeam::channel::{Receiver, Sender};
use eframe::egui;
use partyboy_core::ppu::rgb::Rgb;
use partyboy_core::{debug::GBDebugInfo, input::Keycode};

use crate::{channel_log::Log, MessageFromGb, MessageToGB};

use self::tile_window::TileBankState;

mod gb_display;
mod log_window;
mod map_window;
mod menu_bar;
mod palette_window;
mod side_panel;
mod tile_window;

const KEYS: [egui::Key; 9] = [
    egui::Key::W,
    egui::Key::A,
    egui::Key::S,
    egui::Key::D,
    egui::Key::O,
    egui::Key::K,
    egui::Key::M,
    egui::Key::N,
    egui::Key::Space,
];

pub enum InputType {
    GBInput(Keycode),
    Other(egui::Key),
}

pub struct ToggleState {
    log: bool,
    palletes: bool,
    tile_bank: TileBankState,
    tile: bool,
    maps: bool,
}

impl Default for ToggleState {
    fn default() -> Self {
        Self {
            log: true,
            palletes: true,
            tile: true,
            tile_bank: TileBankState::Bank0,
            maps: true,
        }
    }
}

pub struct DebuggerApp {
    gb_frame_buffer: Option<Vec<Rgb>>,
    logs: Vec<Log>,
    gb_debug_info: Box<GBDebugInfo>,
    fps: f64,

    toggle_state: ToggleState,

    log_rx: Receiver<Log>,
    to_gb_tx: Sender<MessageToGB>,
    from_gb_rx: Receiver<MessageFromGb>,
}

impl DebuggerApp {
    pub fn new(
        _: &eframe::CreationContext,
        log_rx: Receiver<Log>,
        to_gb_tx: Sender<MessageToGB>,
        from_gb_rx: Receiver<MessageFromGb>,
    ) -> Self {
        Self {
            gb_frame_buffer: None,
            logs: Vec::new(),
            gb_debug_info: Box::<GBDebugInfo>::default(),
            fps: 0.0,
            toggle_state: ToggleState::default(),
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
            .filter_map(Self::into_input)
            .collect::<Vec<_>>();
        let _ = self.to_gb_tx.send(MessageToGB::KeyDown(key_downs));

        let key_ups = KEYS
            .iter()
            .filter(|key| input.key_released(**key))
            .filter_map(Self::into_input)
            .collect::<Vec<_>>();
        let _ = self.to_gb_tx.send(MessageToGB::KeyUp(key_ups));
    }

    fn into_input(key: &egui::Key) -> Option<InputType> {
        match key {
            egui::Key::W => Some(InputType::GBInput(Keycode::Up)),
            egui::Key::A => Some(InputType::GBInput(Keycode::Left)),
            egui::Key::S => Some(InputType::GBInput(Keycode::Down)),
            egui::Key::D => Some(InputType::GBInput(Keycode::Right)),
            egui::Key::O => Some(InputType::GBInput(Keycode::A)),
            egui::Key::K => Some(InputType::GBInput(Keycode::B)),
            egui::Key::M => Some(InputType::GBInput(Keycode::Start)),
            egui::Key::N => Some(InputType::GBInput(Keycode::Select)),
            key => Some(InputType::Other(*key)),
        }
    }
}

impl eframe::App for DebuggerApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        let messages = self.from_gb_rx.try_iter();
        for msg in messages {
            match msg {
                MessageFromGb::Draw(fb) => self.gb_frame_buffer = Some(fb),
                MessageFromGb::DebugInfo(debug_info) => self.gb_debug_info = debug_info,
            }
        }

        self.handle_input(ctx);

        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            self.show_menu(ui);
        });
        self.show_side_panel(ctx);
        self.show_gb_display_window(ctx);
        self.show_log_window(ctx);
        self.show_palette_window(ctx);
        self.show_tile_window(ctx);
        self.show_map_window(ctx);

        // TODO:
        // - Tile/Map/Sprite viewer
        // - Memory Viewer
        // - Dissassembly
        // - Rom selector?
        // - ppu event viewer
        // - SideBar of general info? (speed mode, are we in hdma, fps, etc)
        //   - Current cycle?
        //   - Run until cycle?
        //   - Step forward by num of cycles?

        egui::CentralPanel::default().show(ctx, |_| {});

        ctx.request_repaint();
    }
}
