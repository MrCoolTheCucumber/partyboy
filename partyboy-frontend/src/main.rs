use std::path::PathBuf;
use std::sync::Arc;
use std::thread::JoinHandle;

use emu_thread::EmuThreadHandle;
use input::try_into_gameboy_input;
use logging::init_logger;
use msgs::MsgFromGb;
use partyboy_core::ppu::rgb::Rgb;

use clap::Parser;
use crossbeam::channel::{Receiver, Sender};
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use saves::{read_save_file, write_save_file};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::{Window, WindowId},
};

use crate::msgs::MsgToGb;

mod emu_thread;
mod input;
mod logging;
mod msgs;
mod saves;

pub const SCALE: u32 = 2;
pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;

#[derive(Parser, Debug)]
#[command(version = "1.0", about = "A Gameboy color emulator")]
struct Args {
    /// The path to the rom to load.
    #[arg(short, long)]
    rom: Option<String>,

    /// The path to the bios to use.
    #[arg(short, long)]
    bios: Option<String>,

    /// Enables file logging.
    #[arg(short, long)]
    log: bool,
}

struct App {
    args: Args,
    tx: Sender<MsgToGb>,
    rx: Receiver<MsgFromGb>,
    handle: Option<JoinHandle<Option<Box<[u8]>>>>,
    frame_to_draw: Option<Vec<Rgb>>,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let size = LogicalSize::new((WIDTH * SCALE) as f64, (HEIGHT * SCALE) as f64);
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Partyboy 🎉")
                        .with_inner_size(size)
                        .with_resizable(false),
                )
                .expect("Unable to create window"),
        );

        let window_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.clone());
        let pixels = PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
            .enable_vsync(false)
            .build()
            .expect("Unable to create pixel buffer");

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if window_id != window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let pixels = self.pixels.as_mut().expect("pixels not initialized");
                if let Some(frame) = &self.frame_to_draw {
                    let flat_frame = frame
                        .iter()
                        .flat_map(|px| [px.r, px.g, px.b, 0xFF])
                        .collect::<Vec<_>>();
                    pixels.frame_mut().copy_from_slice(flat_frame.as_slice());
                }

                if let Err(e) = pixels.render() {
                    log::error!("pixels.render() failed: {}", e);
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput { event, .. } if !event.repeat => {
                let key = event.key_without_modifiers();
                if let Some(gb_input) = try_into_gameboy_input(key.as_ref()) {
                    match event.state {
                        ElementState::Pressed => self.tx.send(MsgToGb::KeyDown(gb_input)).unwrap(),
                        ElementState::Released => self.tx.send(MsgToGb::KeyUp(gb_input)).unwrap(),
                    }
                }

                match key.as_ref() {
                    Key::Named(NamedKey::Escape) => {
                        event_loop.exit();
                    }
                    Key::Named(NamedKey::Space) => {
                        self.tx
                            .send(MsgToGb::Turbo(event.state.is_pressed()))
                            .unwrap();
                    }
                    Key::Character("q") => {
                        self.tx
                            .send(MsgToGb::Rewind(event.state.is_pressed()))
                            .unwrap();
                    }
                    Key::Character("c") => self.tx.send(MsgToGb::SaveSnapshot).unwrap(),
                    Key::Character("v") => self.tx.send(MsgToGb::LoadSnapshot).unwrap(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        for msg in self.rx.try_iter() {
            match msg {
                MsgFromGb::Frame(fb) => self.frame_to_draw = Some(fb),
                MsgFromGb::Fps(fps) => window.set_title(format!("{:.2}", fps).as_str()),
            }
        }

        window.request_redraw();
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.tx
            .send(MsgToGb::Shutdown)
            .expect("Unable to signal emu thread to stop");

        let ram = self
            .handle
            .take()
            .expect("Emulator JoinHandle should not be empty")
            .join()
            .expect("Unable to join emu thread");

        if let Some(ram) = ram {
            if let Some(path) = self.args.rom.as_ref() {
                write_save_file(&PathBuf::from(path), &ram);
            }
        } else {
            log::warn!("Emulator was unable to read cart ram, no save file was (over)written");
        }
    }
}

fn main() {
    let args = Args::parse();

    #[cfg(debug_assertions)]
    init_logger(args.log);

    let rom = args
        .rom
        .as_ref()
        .map(|path| std::fs::read(path).expect("Unable to read game file"));

    let ram = args
        .rom
        .as_ref()
        .and_then(|path| read_save_file(&PathBuf::from(path)));

    let bios = args
        .bios
        .as_ref()
        .map(|path| std::fs::read(path).expect("Unable to read bios file"));

    let EmuThreadHandle { tx, rx, handle } = emu_thread::new(rom, bios, ram);

    let event_loop = EventLoop::new().expect("Unable to create event loop");
    let mut app = App {
        args,
        tx,
        rx,
        handle: Some(handle),
        frame_to_draw: None,
        window: None,
        pixels: None,
    };

    event_loop
        .run_app(&mut app)
        .expect("Unable to start event loop");
}
