use std::{env, time::Duration};

use app::DebuggerApp;
use channel_log::ChannelLog;
use crossbeam::channel::{Receiver, Sender};
use eframe::{egui::Context, emath::Vec2, NativeOptions};
use messages::{MessageFromGb, MessageToGB};
use partyboy_core::{builder::GameBoyBuilder, GameBoy};
use spin_sleep_util::{MissedTickBehavior, RateReporter};

mod app;
mod channel_log;
mod messages;

pub static mut CYCLE_COUNT: u64 = 0;

fn gb_loop(to_gb_rx: Receiver<MessageToGB>, from_gb_tx: Sender<MessageFromGb>, ctx: Context) -> ! {
    let mut gb: Option<GameBoy> = None;

    let mut interval =
        spin_sleep_util::interval(Duration::from_millis((1000.0f64 / 59.73f64) as u64))
            .with_missed_tick_behavior(MissedTickBehavior::Burst);
    let mut reporter = RateReporter::new(Duration::from_millis(500));

    let mut run = true;
    let mut turbo = false;

    loop {
        let inbound_messages = to_gb_rx.try_iter();
        for msg in inbound_messages {
            match msg {
                MessageToGB::New(rom_path) => {
                    drop(gb);
                    // TODO: handle saving
                    let rom = std::fs::read(rom_path).expect("Unable to read rom path");
                    let bios = include_bytes!("../../bin/_cgb_boot.bin");
                    gb = GameBoyBuilder::new()
                        .rom(rom)
                        .bios(bios.to_vec())
                        .build()
                        .map_err(|e| log::error!("{}", e))
                        .ok();

                    unsafe { CYCLE_COUNT = 0 }
                }
                MessageToGB::Start => {
                    interval.reset();
                    run = true;
                }
                MessageToGB::Stop => {
                    run = false;
                }
                MessageToGB::KeyDown(keys) => {
                    use eframe::egui::Key;
                    if let Some(gb) = &mut gb {
                        keys.iter().for_each(|input| match input {
                            app::InputType::GBInput(keycode) => gb.key_down(*keycode),
                            app::InputType::Other(key) => {
                                if let Key::Space = key {
                                    turbo = true
                                }
                            }
                        });
                    }
                }
                MessageToGB::KeyUp(keys) => {
                    use eframe::egui::Key;
                    if let Some(gb) = &mut gb {
                        keys.iter().for_each(|input| match input {
                            app::InputType::GBInput(keycode) => gb.key_up(*keycode),
                            app::InputType::Other(key) => {
                                if let Key::Space = key {
                                    turbo = false;
                                    interval.reset();
                                }
                            }
                        });
                    }
                }
            }
        }

        if run {
            if let Some(gb) = &mut gb {
                loop {
                    unsafe { CYCLE_COUNT += 1 }
                    gb.tick();
                    if gb.consume_draw_flag() {
                        let _ =
                            from_gb_tx.send(MessageFromGb::Draw(gb.get_frame_buffer().to_vec()));

                        let mut debug_info = gb.debug_info();

                        if let Some(fps) = reporter.increment_and_report() {
                            debug_info.fps = Some(fps);
                        }

                        let _ = from_gb_tx.send(MessageFromGb::DebugInfo(Box::new(debug_info)));

                        ctx.request_repaint();
                        break;
                    }
                }

                if !turbo {
                    interval.tick();
                }
            }
        } else {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    let (channel_log, log_rx) = ChannelLog::new();
    log_panics::init();
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
        "Partyboy Debug",
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
