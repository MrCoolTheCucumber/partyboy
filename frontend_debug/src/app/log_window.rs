use eframe::{
    egui::{self, ScrollArea, TextFormat, Ui},
    epaint::{text::LayoutJob, Color32, FontId},
};

use crate::channel_log::Log;

use super::DebuggerApp;

impl DebuggerApp {
    pub(super) fn show_log_window(&mut self, ctx: &egui::Context) {
        let mut logs: Vec<Log> = self.log_rx.try_iter().collect();
        self.logs.append(&mut logs);

        egui::Window::new("Log")
            .resizable(false)
            .fixed_size([300.0, 400.0])
            .show(ctx, |ui| {
                self.render_log_window_display(ctx, ui);
            });
    }

    fn render_log_window_display(&self, _: &egui::Context, ui: &mut Ui) {
        ScrollArea::vertical().stick_to_bottom().show_rows(
            ui,
            14.0,
            self.logs.len(),
            |ui, row_range| {
                for row in row_range {
                    let log = &self.logs[row];
                    render_log_line(ui, log);
                }
            },
        );
        ui.allocate_space(ui.available_size());
    }
}

fn render_log_line(ui: &mut Ui, log: &Log) {
    let mut job = LayoutJob::default();

    job.append(
        "[",
        0.0,
        TextFormat {
            color: Color32::BLACK,
            font_id: FontId::monospace(12.0),
            ..Default::default()
        },
    );

    job.append(
        log.level().as_str(),
        0.0,
        TextFormat {
            color: log.level_color(),
            font_id: FontId::monospace(12.0),
            ..Default::default()
        },
    );

    job.append(
        " ",
        0.0,
        TextFormat {
            color: Color32::BLACK,
            font_id: FontId::monospace(12.0),
            ..Default::default()
        },
    );

    job.append(
        log.target(),
        0.0,
        TextFormat {
            color: Color32::BLACK,
            font_id: FontId::monospace(12.0),
            ..Default::default()
        },
    );

    job.append(
        "] ",
        0.0,
        TextFormat {
            color: Color32::BLACK,
            font_id: FontId::monospace(12.0),
            ..Default::default()
        },
    );

    job.append(
        log.msg().as_str(),
        0.0,
        TextFormat {
            color: Color32::BLACK,
            font_id: FontId::monospace(12.0),
            ..Default::default()
        },
    );

    ui.label(job);
}
