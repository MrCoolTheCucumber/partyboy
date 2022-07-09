// https://github.com/vojty/feather-gb

use std::env;

use app::DebugerApp;
use channel_log::ChannelLog;
use eframe::{emath::Vec2, NativeOptions};

mod app;
mod channel_log;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    let (channel_log, rx) = ChannelLog::new();
    let _ = flexi_logger::Logger::try_with_env()
        .unwrap()
        .log_to_writer(Box::new(channel_log))
        .start()
        .unwrap();

    let options = NativeOptions {
        maximized: true,
        resizable: true,
        initial_window_size: Some(Vec2::new(1000.0, 1000.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Partyboy",
        options,
        Box::new(|cc| Box::new(DebugerApp::new(cc, rx))),
    );
}
