use std::path::PathBuf;

use emu_thread::EmuThreadHandle;
use input::try_into_gameboy_input;
use logging::init_logger;
use msgs::MsgFromGb;
use partyboy_core::ppu::rgb::Rgb;

use clap::Parser;
use pixels::{PixelsBuilder, SurfaceTexture};
use saves::{read_save_file, write_save_file};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::WindowBuilder,
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

fn parse_args() -> Args {
    Args::parse()
}

fn main() {
    let args = parse_args();

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
        .map(|path| std::fs::read(path).expect("Unable to read bios file"));

    let event_loop = EventLoop::new().expect("Unable to create event loop");
    let window = WindowBuilder::new()
        .with_title("Partyboy 🎉")
        .with_inner_size(LogicalSize::new(WIDTH * SCALE, HEIGHT * SCALE))
        .with_resizable(false)
        .build(&event_loop)
        .expect("Unable to create window");

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
            .enable_vsync(false)
            .build()
            .unwrap()
    };

    let EmuThreadHandle { tx, rx, handle } = emu_thread::new(rom, bios, ram);
    let mut handle = Some(handle);

    let mut frame_to_draw: Option<Vec<Rgb>> = None;

    event_loop
        .run(move |event, elwt| {
            let msgs: Vec<MsgFromGb> = rx.try_iter().collect();
            for msg in msgs {
                match msg {
                    MsgFromGb::Frame(fb) => frame_to_draw = Some(fb),
                    MsgFromGb::Fps(fps) => window.set_title(format!("{:.2}", fps).as_str()),
                }
            }

            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::RedrawRequested => {
                            if let Some(frame) = &frame_to_draw {
                                let flat_frame = frame
                                    .iter()
                                    .flat_map(|px| [px.r, px.g, px.b, 0xFF])
                                    .collect::<Vec<_>>();
                                pixels.frame_mut().copy_from_slice(flat_frame.as_slice());
                            }

                            if let Err(e) = pixels.render() {
                                log::error!("pixels.render() failed: {}", e);
                                elwt.exit();
                                return;
                            }
                        }
                        WindowEvent::KeyboardInput { event, .. } if !event.repeat => {
                            let key = event.key_without_modifiers();
                            if let Some(gb_input) = try_into_gameboy_input(key.as_ref()) {
                                match event.state {
                                    ElementState::Pressed => {
                                        tx.send(MsgToGb::KeyDown(gb_input)).unwrap()
                                    }
                                    ElementState::Released => {
                                        tx.send(MsgToGb::KeyUp(gb_input)).unwrap()
                                    }
                                }
                            }

                            match key.as_ref() {
                                Key::Named(NamedKey::Escape) => {
                                    elwt.exit();
                                    return;
                                }
                                Key::Named(NamedKey::Space) => {
                                    tx.send(MsgToGb::Turbo(event.state.is_pressed())).unwrap();
                                }
                                Key::Character("q") => {
                                    tx.send(MsgToGb::Rewind(event.state.is_pressed())).unwrap();
                                }
                                Key::Character("c") => tx.send(MsgToGb::SaveSnapshot).unwrap(),
                                Key::Character("v") => tx.send(MsgToGb::LoadSnapshot).unwrap(),
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                Event::LoopExiting => {
                    tx.send(MsgToGb::Shutdown)
                        .expect("Unable to signal emu thread to stop");

                    let ram = handle
                        .take()
                        .expect("Recieved close request twice?")
                        .join()
                        .expect("Unable to join emu thread to main thread");

                    if let Some(ram) = ram {
                        if let Some(path) = args.rom.as_ref() {
                            write_save_file(&PathBuf::from(path), &ram);
                        }
                    } else {
                        log::warn!("Emulator was unable to read cart ram?");
                    }
                }
                _ => {}
            }

            window.request_redraw();
        })
        .expect("Unable to start event loop?");
}
