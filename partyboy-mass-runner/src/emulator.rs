use std::time::{Duration, Instant};

use image::{ImageBuffer, RgbImage};
use partyboy_core::builder::GameBoyBuilder;
use partyboy_core::input::Keycode;
use partyboy_core::ppu::rgb::Rgb;
use partyboy_core::GameBoy;

use crate::rom::{rom_display_name, Rom};
use crate::types::{RunResult, WorkerStatus};

pub fn into_img(fb: Vec<Rgb>) -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(160, 144);
    fb.into_iter()
        .flat_map(|rgb| [rgb.r, rgb.g, rgb.b])
        .zip(img.iter_mut())
        .for_each(|(px, img_px)| *img_px = px);
    img
}

/// Run `ticks` emulator ticks, optionally throttling to `speed_factor` * real time.
/// `status` is updated with absolute emulated seconds (offset by `base_seconds` for
/// multi-phase runs).
pub fn run_emulated_ticks<F>(
    gb: &mut GameBoy,
    ticks: u64,
    speed_factor: f64,
    mut on_tick: F,
    status: Option<&WorkerStatus>,
    base_seconds: f64,
) where
    F: FnMut(&mut GameBoy, u64),
{
    if speed_factor <= 0.0 {
        let update_every = partyboy_core::SPEED / 4;
        for i in 0..ticks {
            let _ = gb.tick();
            on_tick(gb, i);
            if let Some(s) = status {
                if i % update_every == 0 {
                    let secs = base_seconds + (i as f64 / partyboy_core::SPEED as f64);
                    s.update_emulated(secs as u64);
                }
            }
        }
        return;
    }

    let start = Instant::now();
    let check_every = partyboy_core::SPEED;

    for i in 0..ticks {
        let _ = gb.tick();
        on_tick(gb, i);

        if i % check_every == 0 {
            let emulated_in_phase = i as f64 / partyboy_core::SPEED as f64;
            let absolute = base_seconds + emulated_in_phase;

            if let Some(s) = status {
                s.update_emulated(absolute as u64);
            }

            let desired = emulated_in_phase / speed_factor;
            let real = start.elapsed().as_secs_f64();
            if real < desired {
                std::thread::sleep(Duration::from_secs_f64(desired - real));
            }
        }
    }
}

/// Run one ROM through the QA pass (40s warm-up + 80s with button mashing) and capture
/// a screenshot at each phase boundary.
pub fn run_one_rom(
    rom: Rom,
    bios: Option<&[u8]>,
    speed_factor: f64,
    status: Option<&WorkerStatus>,
) -> RunResult {
    let rom_name = rom_display_name(&rom.path);

    let result = std::panic::catch_unwind(|| {
        let mut builder = GameBoyBuilder::new();
        if let Some(b) = bios {
            builder = builder.bios(b.to_vec());
        }
        let mut gb = builder.rom(rom.bytes).build().unwrap();

        // Phase 1: 0-40s warm-up (no inputs)
        run_emulated_ticks(
            &mut gb,
            partyboy_core::SPEED * 40,
            speed_factor,
            |_, _| {},
            status,
            0.0,
        );
        let fourty = gb.get_frame_buffer().to_vec();

        // Phase 2: 40-120s alternating Start / A every half second
        let mut pressed = false;
        run_emulated_ticks(
            &mut gb,
            partyboy_core::SPEED * 80,
            speed_factor,
            |gb, tick| {
                if tick % (partyboy_core::SPEED / 2) == 0 {
                    if pressed {
                        gb.key_down(Keycode::A);
                        gb.key_up(Keycode::Start);
                    } else {
                        gb.key_up(Keycode::A);
                        gb.key_down(Keycode::Start);
                    }
                    pressed = !pressed;
                }
            },
            status,
            40.0,
        );
        let onetwenty = gb.get_frame_buffer().to_vec();

        RunResult::Success {
            fourty_seconds_frame_buffer: fourty,
            onetwenty_seconds_frame_buffer: onetwenty,
            rom_path: rom.path,
        }
    });

    match result {
        Ok(r) => r,
        Err(payload) => RunResult::Fail {
            rom_name,
            error: payload,
        },
    }
}
