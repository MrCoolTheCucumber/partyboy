use std::{env, fs::File};

use log::LevelFilter;

// TODO: why am I using two logger libs?
pub fn init_logger(enable_file_logging: bool) {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    let mut builder = env_logger::builder();
    let mut builder_ref = builder
        .format_timestamp(None)
        .filter_module("wgpu_core", LevelFilter::Warn)
        .filter_module("wgpu_hal", LevelFilter::Error)
        .filter_module("naga", LevelFilter::Warn);

    if enable_file_logging {
        let file = Box::new(File::create("log/output.log").unwrap());
        builder_ref = builder_ref.target(env_logger::Target::Pipe(file));
    }

    builder_ref.init();

    log_panics::init();
}
