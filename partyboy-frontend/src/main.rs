use input::try_into_gameboy_input;
use logging::init_logger;
use msgs::MsgFromGb;
use partyboy_core::ppu::rgb::Rgb;

use clap::clap_app;
use pixels::{wgpu::Backends, PixelsBuilder, SurfaceTexture};
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

pub const SCALE: u32 = 2;
pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;

struct Args {
    rom_path: Option<String>,
    bios_path: Option<String>,
    enable_file_logging: bool,
}

fn parse_args() -> Args {
    let matches = clap_app!(partyboy =>
        (version: "1.0")
        (about: "A Gameboy color emulator")
        (@arg rom_path: -r --rom +takes_value "The path to the rom to load.")
        (@arg bios_path: -b --bios +takes_value "The path to the bios to use.")
        (@arg enable_file_logging: -l --log "Enables file logging.")
    )
    .get_matches();

    let rom_path = matches.value_of("rom_path").map(|str| str.to_owned());
    let bios_path = matches.value_of("bios_path").map(|str| str.to_owned());
    let enable_file_logging = matches.is_present("enable_file_logging");

    Args {
        rom_path,
        bios_path,
        enable_file_logging,
    }
}

// fn get_save_file_path_from_rom_path(path: &Path) -> PathBuf {
//     let mut save_file_path = PathBuf::from(path);
//     let file_name = save_file_path
//         .file_stem()
//         .unwrap()
//         .to_str()
//         .unwrap()
//         .to_owned();
//     save_file_path.pop();
//     save_file_path.push(format!("{}.sav", file_name));
//     save_file_path
// }

fn main() {
    let args = parse_args();

    #[cfg(debug_assertions)]
    init_logger(args.enable_file_logging);

    let rom = args
        .rom_path
        .map(|path| std::fs::read(path).expect("Unable to read game file"));

    let bios = args
        .bios_path
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
            .wgpu_backend(Backends::VULKAN)
            .build()
            .unwrap()
    };

    let (s, r) = emu_thread::new(rom, bios);
    let mut frame_to_draw: Option<Vec<Rgb>> = None;

    event_loop
        .run(move |event, elwt| {
            let msgs: Vec<MsgFromGb> = r.try_iter().collect();
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
                                        s.send(MsgToGb::KeyDown(gb_input)).unwrap()
                                    }
                                    ElementState::Released => {
                                        s.send(MsgToGb::KeyUp(gb_input)).unwrap()
                                    }
                                }
                            }

                            match key.as_ref() {
                                Key::Named(NamedKey::Escape) => {
                                    elwt.exit();
                                    return;
                                }
                                Key::Named(NamedKey::Space) => {
                                    s.send(MsgToGb::Turbo(event.state.is_pressed())).unwrap();
                                }
                                Key::Character("q") => {
                                    s.send(MsgToGb::Rewind(event.state.is_pressed())).unwrap();
                                }
                                Key::Character("c") => s.send(MsgToGb::SaveSnapshot).unwrap(),
                                Key::Character("v") => s.send(MsgToGb::LoadSnapshot).unwrap(),
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            window.request_redraw();
        })
        .expect("Unable to start event loop?");
}
