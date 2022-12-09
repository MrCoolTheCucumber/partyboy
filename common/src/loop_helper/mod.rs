#[cfg(feature = "web")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(feature = "web", wasm_bindgen)]
pub struct LoopHelper {
    last_loop: u64,
    last_fps_report: u64,
    frames_drawn: u64,
    ticks_per_second: u64,
    fps_report_rate_ms: u64,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl LoopHelper {
    pub fn new(fps_report_rate_ms: u64, ticks_per_second: u64) -> Self {
        let now = crate::time::now();
        Self {
            last_loop: now,
            last_fps_report: now,
            frames_drawn: 0,
            ticks_per_second,
            fps_report_rate_ms,
        }
    }

    pub fn reset(&mut self) {
        let now = crate::time::now();
        self.last_loop = now;
        self.last_fps_report = now;
        self.frames_drawn = 0;
    }

    pub fn calculate_ticks_to_run(&mut self, now: u64) -> u64 {
        let elapsed = now - self.last_loop;
        let ticks = (elapsed * self.ticks_per_second) / 1000;
        self.last_loop = now;
        ticks
    }

    pub fn record_frame_draw(&mut self) {
        self.frames_drawn += 1;
    }

    pub fn report_fps(&mut self, now: u64) -> Option<f64> {
        let elasped_since_last_fps_report = now - self.last_fps_report;
        (elasped_since_last_fps_report >= self.fps_report_rate_ms).then(|| {
            self.last_fps_report = now;
            let fps = (self.frames_drawn as f64) / (elasped_since_last_fps_report as f64 / 1000.0);
            self.frames_drawn = 0;
            fps
        })
    }
}
