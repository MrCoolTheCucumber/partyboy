use std::env;

use gameboy::GameBoy;
use log::{log_enabled, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

mod gameboy;

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

    loop {
        gb.tick();
    }
}
