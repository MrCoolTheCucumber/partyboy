use std::env;

use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

// TODO: why am I using two logger libs?
pub fn init_logger(enable_file_logging: bool) {
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
        env_logger::builder()
            .format_timestamp(None)
            .filter_module("wgpu_core", LevelFilter::Warn)
            .init();
    }

    log_panics::init();
}
