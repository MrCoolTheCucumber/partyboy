use std::time::Duration;

use crossbeam::channel::Receiver;
use eframe::egui;
use gameboy::GameBoy;
use spin_sleep::LoopHelper;

use crate::channel_log::Log;

use self::menu_bar::RecentRomInfo;

mod gb_display;
mod log_window;
mod menu_bar;

pub struct DebugerApp {
    gameboy: Option<GameBoy>,
    recent_roms: Vec<RecentRomInfo>,
    loop_helper: LoopHelper,

    log_rx: Receiver<Log>,
    logs: Vec<Log>,
}

impl DebugerApp {
    pub fn new(_: &eframe::CreationContext, log_rx: Receiver<Log>) -> Self {
        let loop_helper = LoopHelper::builder()
            .report_interval(Duration::from_millis(500))
            .build_with_target_rate(59.73);

        Self {
            gameboy: None,
            recent_roms: Vec::new(),
            loop_helper,
            log_rx,
            logs: Vec::new(),
        }
    }

    fn tick_to_next_frame_draw(&mut self) {
        if let Some(gb) = &mut self.gameboy {
            loop {
                gb.tick();
                if gb.consume_draw_flag() {
                    break;
                }
            }
        }
    }
}

impl eframe::App for DebugerApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
        let _ = self.loop_helper.loop_start();

        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            self.show_menu(ui);
        });
        self.show_log_window(ctx);
        self.tick_to_next_frame_draw();
        self.show_gb_display_window(ctx);
        egui::CentralPanel::default().show(ctx, |_| {});

        self.loop_helper.loop_sleep();
        ctx.request_repaint();
    }
}
