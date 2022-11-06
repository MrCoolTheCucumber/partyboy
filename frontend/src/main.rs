use gameboy::ppu::rgb::Rgb;
use input::{get_key_downs, get_key_ups};
use logging::init_logger;
use msgs::MsgFromGb;

use clap::clap_app;
use pixels::{PixelsBuilder, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

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
    enable_file_logging: bool,
}

fn parse_args() -> Args {
    let matches = clap_app!(partyboy =>
        (version: "1.0")
        (about: "A Gameboy color emulator")
        (@arg rom_path: -r --rom +takes_value "The path to the rom to load.")
        (@arg enable_file_logging: -l --log "Enables file logging.")
    )
    .get_matches();

    let rom_path = matches.value_of("rom_path").map(|str| str.to_owned());
    let enable_file_logging = matches.is_present("enable_file_logging");

    Args {
        rom_path,
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

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
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

    let (s, r) = emu_thread::new(rom);
    let mut frame_to_draw: Option<Vec<Rgb>> = None;

    event_loop.run(move |event, _, control_flow| {
        let msgs: Vec<MsgFromGb> = r.try_iter().collect();
        for msg in msgs {
            match msg {
                MsgFromGb::Frame(fb) => frame_to_draw = Some(fb),
                MsgFromGb::Fps(fps) => window.set_title(format!("{:.2}", fps).as_str()),
            }
        }

        match event {
            Event::RedrawRequested(_) => {
                if let Some(frame) = &frame_to_draw {
                    let flat_frame = frame
                        .iter()
                        .flat_map(|px| [px.r, px.g, px.b, 0xFF])
                        .collect::<Vec<_>>();
                    pixels
                        .get_frame_mut()
                        .copy_from_slice(flat_frame.as_slice());
                }

                if let Err(e) = pixels.render() {
                    log::error!("pixels.render() failed: {}", e);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            Event::LoopDestroyed => {
                *control_flow = ControlFlow::Exit;
                return;
            }
            _ => {}
        }

        if input.update(&event) {
            // TODO: use 1 message..
            let key_downs = get_key_downs(&mut input);
            let key_ups = get_key_ups(&mut input);

            if !key_downs.is_empty() {
                let keydown_msg = MsgToGb::KeyDown(key_downs);
                s.send(keydown_msg).unwrap();
            }

            if !key_ups.is_empty() {
                let keyup_msg = MsgToGb::KeyUp(key_ups);
                s.send(keyup_msg).unwrap();
            }

            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.key_pressed(VirtualKeyCode::Space) {
                s.send(MsgToGb::Turbo(true)).unwrap();
            }
            if input.key_released(VirtualKeyCode::Space) {
                s.send(MsgToGb::Turbo(false)).unwrap();
            }

            if input.key_pressed(VirtualKeyCode::C) {
                s.send(MsgToGb::SaveSnapshot).unwrap();
            }
            if input.key_released(VirtualKeyCode::V) {
                s.send(MsgToGb::LoadSnapshot).unwrap();
            }
        }

        window.request_redraw();
    });
}
