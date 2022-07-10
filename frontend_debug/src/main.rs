use std::{env, time::Duration};

use app::DebuggerApp;
use channel_log::ChannelLog;
use crossbeam::channel::{Receiver, Sender};
use eframe::{egui::Context, emath::Vec2, NativeOptions};
use gameboy::{builder::GameBoyBuilder, GameBoy};
use messages::{MessageFromGb, MessageToGB};
use spin_sleep::LoopHelper;

mod app;
mod channel_log;
mod messages;

fn gb_loop(to_gb_rx: Receiver<MessageToGB>, from_gb_tx: Sender<MessageFromGb>, ctx: Context) -> ! {
    let mut gb: Option<GameBoy> = None;
    let mut loop_helper = LoopHelper::builder()
        .report_interval(Duration::from_millis(500))
        .build_with_target_rate(59.73);

    let mut run = true;

    loop {
        let _ = loop_helper.loop_start();

        let inbound_messages = to_gb_rx.try_iter();
        for msg in inbound_messages {
            match msg {
                MessageToGB::New(rom_path) => {
                    gb = GameBoyBuilder::new()
                        .rom_path(rom_path.as_str())
                        .build()
                        .map_err(|e| log::error!("{}", e))
                        .ok();
                }
                MessageToGB::Start => {
                    run = true;
                }
                MessageToGB::Stop => {
                    run = false;
                }
                MessageToGB::KeyDown(keys) => {
                    if let Some(gb) = &mut gb {
                        keys.iter().for_each(|key| gb.key_down(*key));
                    }
                }
                MessageToGB::KeyUp(keys) => {
                    if let Some(gb) = &mut gb {
                        keys.iter().for_each(|key| gb.key_up(*key));
                    }
                }
            }
        }

        if run {
            if let Some(gb) = &mut gb {
                loop {
                    gb.tick();
                    if gb.consume_draw_flag() {
                        let _ =
                            from_gb_tx.send(MessageFromGb::Draw(gb.get_frame_buffer().to_vec()));

                        let _ = from_gb_tx.send(MessageFromGb::DebugInfo(gb.debug_info()));
                        ctx.request_repaint();
                        break;
                    }
                }
            }
        }

        loop_helper.loop_sleep();
    }
}

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    let (channel_log, log_rx) = ChannelLog::new();
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

    let (to_gb_tx, to_gb_rx) = crossbeam::channel::unbounded::<MessageToGB>();
    let (from_gb_tx, from_gb_rx) = crossbeam::channel::unbounded::<MessageFromGb>();

    eframe::run_native(
        "Partyboy",
        options,
        Box::new(|cc| {
            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(|| {
                gb_loop(to_gb_rx, from_gb_tx, ctx);
            });
            Box::new(DebuggerApp::new(cc, log_rx, to_gb_tx, from_gb_rx))
        }),
    );
}
