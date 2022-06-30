use std::{env, time::Duration};

use crate::input::{handle_key_down, handle_key_up};
use clap::clap_app;
use gameboy::GameBoy;
use gl::types::GLuint;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use sdl2::video::SwapInterval;
use spin_sleep::LoopHelper;

mod input;
mod render;

pub const SCALE: u32 = 2;
pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;

struct Args {
    rom_path: String,
    enable_file_logging: bool,
}

fn init_logger(enable_file_logging: bool) {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    if enable_file_logging {
        const LOG_PATTERN: &str = "{m}\n";
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
            .build("log/output.log")
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(
                Root::builder()
                    .appender("logfile")
                    .build(LevelFilter::Debug),
            )
            .unwrap();

        log4rs::init_config(config).unwrap();
    } else {
        env_logger::builder().format_timestamp(None).init();
    }

    log_panics::init();
}

fn parse_args() -> Args {
    let matches = clap_app!(partyboy =>
        (version: "1.0")
        (about: "A Gameboy color emulator")
        (@arg rom_path: -r --rom +takes_value +required "The path to the rom to load.")
        (@arg enable_file_logging: -l --log "Enables file logging.")
    )
    .get_matches();

    let rom_path = matches.value_of("rom_path").unwrap().to_owned();
    let enable_file_logging = matches.is_present("enable_file_logging");

    Args {
        rom_path,
        enable_file_logging,
    }
}

fn main() {
    let args = parse_args();

    #[cfg(debug_assertions)]
    init_logger(args.enable_file_logging);

    // let mut gb = GameBoy::new(&args.rom_path);
    let mut gb = GameBoy::builder()
        .rom_path(args.rom_path.as_str())
        .build()
        .unwrap();
    log::info!("Initialized gameboy.");

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();

    {
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 0);
    }

    let mut window = video
        .window("Partyboy", WIDTH * SCALE, HEIGHT * SCALE)
        .position_centered()
        .opengl()
        .allow_highdpi()
        .build()
        .unwrap();

    let _gl_context = window
        .gl_create_context()
        .expect("Couldn't create GL context");

    let _ = video.gl_set_swap_interval(SwapInterval::Immediate);
    gl::load_with(|s| video.gl_get_proc_address(s) as _);

    let mut event_pump = sdl.event_pump().unwrap();

    let mut fb_id: GLuint = 0;
    let mut tex_id: GLuint = 0;
    render::init_gl_state(&mut tex_id, &mut fb_id);

    unsafe {
        gl::ClearColor(0.4549, 0.92549, 0.968627, 0.7);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }

    let mut loop_helper = LoopHelper::builder()
        .report_interval(Duration::from_millis(500))
        .build_with_target_rate(59.73);

    let mut turbo = false;

    'running: loop {
        use sdl2::event::Event;

        let _ = loop_helper.loop_start();

        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode, repeat, ..
                } => {
                    if !repeat {
                        if let Some(keycode) = keycode {
                            if matches!(keycode, sdl2::keyboard::Keycode::Tab) {
                                turbo = true;
                            } else {
                                handle_key_down(&mut gb, keycode);
                            }
                        }
                    }
                }

                Event::KeyUp {
                    keycode, repeat, ..
                } => {
                    if !repeat {
                        if let Some(keycode) = keycode {
                            if matches!(keycode, sdl2::keyboard::Keycode::Tab) {
                                turbo = false;
                            } else {
                                handle_key_up(&mut gb, keycode);
                            }
                        }
                    }
                }

                Event::Quit { .. } => break 'running,

                _ => {}
            }
        }

        while !gb.consume_draw_flag() {
            gb.tick();
        }

        render::render_gb(&gb, fb_id, tex_id);
        window.gl_swap_window();

        if let Some(fps) = loop_helper.report_rate() {
            let _ = window.set_title(format!("{:.2}", fps).as_str());
        }

        if !turbo {
            loop_helper.loop_sleep();
        }
    }
}
