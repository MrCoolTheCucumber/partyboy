use std::{env, time::Duration};

use gameboy::GameBoy;
use gl::types::GLuint;
use log::{log_enabled, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

mod gameboy;
mod render;

pub const SCALE: u32 = 2;
pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;

fn init_logger() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    if log_enabled!(log::Level::Debug) {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
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
        env_logger::init();
    }

    log_panics::init();
}

fn main() {
    init_logger();

    let mut gb = GameBoy::new("/mnt/i/Dev/gb-rs/cpu_instrs.gb");
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
    gl::load_with(|s| video.gl_get_proc_address(s) as _);

    let mut event_pump = sdl.event_pump().unwrap();

    let mut fb_id: GLuint = 0;
    let mut tex_id: GLuint = 0;
    render::init_gl_state(&mut tex_id, &mut fb_id);

    'running: loop {
        use sdl2::event::Event;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(0.4549, 0.92549, 0.968627, 0.7);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        std::thread::sleep(Duration::from_millis(16));

        gb.tick();
    }
}
