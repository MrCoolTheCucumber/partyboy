use std::env;

use gameboy::GameBoy;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

mod gameboy;

fn main() {
    #[cfg(debug_assertions)]
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    #[cfg(debug_assertions)]
    {
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

        // log4rs::init_config(config).unwrap();
    }

    env_logger::init();
    log_panics::init();

    let mut gb = GameBoy::new("/mnt/i/Dev/gb-rs/cpu_instrs.gb");
    log::info!("Initialized gameboy.");

    loop {
        gb.tick();
    }
}
