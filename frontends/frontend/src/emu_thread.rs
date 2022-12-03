use std::time::{Duration, Instant};

use crossbeam::channel::{Receiver, Sender};
use gameboy::GameBoy;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer, RingBufferExt, RingBufferWrite};

use crate::msgs::{MsgFromGb, MsgToGb};

const FPS_REPORT_RATE_MS: u64 = 500;

pub fn new(rom: Option<Vec<u8>>) -> (Sender<MsgToGb>, Receiver<MsgFromGb>) {
    let (s_to_gb, r_from_ui) = crossbeam::channel::bounded::<MsgToGb>(32);
    let (s_to_ui, r_from_gb) = crossbeam::channel::bounded::<MsgFromGb>(128);

    std::thread::spawn(move || {
        let (s, r) = (s_to_ui, r_from_ui);

        // TODO: make this an option and be able to set rom via msg
        let mut builder = GameBoy::builder();
        if let Some(rom) = rom {
            builder = builder.rom(rom);
        }
        let mut gb = builder
            .build()
            .expect("Internal error: unable to construct emulator instance");

        let mut turbo = false;
        let mut snapshot: Option<Vec<u8>> = None;

        let start = Instant::now();
        let mut last_loop = start;

        let mut last_fps_report = start;
        let mut frames_drawn = 0;

        let mut last_8_frames = ConstGenericRingBuffer::<_, 8>::new();

        loop {
            let msgs: Vec<MsgToGb> = r.try_iter().collect();
            for msg in msgs {
                match msg {
                    MsgToGb::Load => todo!(),
                    MsgToGb::KeyDown(keys) => {
                        log::debug!("{:?}", keys);
                        keys.into_iter().for_each(|key| gb.key_down(key))
                    }
                    MsgToGb::KeyUp(keys) => keys.into_iter().for_each(|key| gb.key_up(key)),
                    MsgToGb::Turbo(state) => {
                        turbo = state;
                        last_8_frames.clear();
                    }
                    MsgToGb::SaveSnapshot => {
                        let buf = rmp_serde::to_vec(&gb).unwrap();
                        log::info!("Snapshot taken: {}", buf.len());
                        snapshot = Some(buf);
                    }
                    MsgToGb::LoadSnapshot => {
                        if let Some(snapshot) = &snapshot {
                            let snapshot: GameBoy = rmp_serde::from_slice(snapshot).unwrap();
                            gb.load_snapshot(snapshot);
                            log::info!("Loaded snapshot")
                        }
                    }
                }
            }

            // calculate how many ticks have elapsed
            let now = Instant::now();
            let elapsed = now - last_loop;
            let ticks = if turbo {
                gameboy::SPEED
            } else {
                (elapsed.as_secs_f64() * (gameboy::SPEED as f64)) as u64
            };

            last_loop = now;

            for _ in 0..ticks {
                gb.tick();
                if gb.consume_draw_flag() {
                    let frame_msg = MsgFromGb::Frame(gb.get_frame_buffer().into());
                    let _ = s.try_send(frame_msg);
                    frames_drawn += 1;
                }
            }

            // check if we should report fps
            let elapsed_since_last_fps_report = now - last_fps_report;
            if elapsed_since_last_fps_report.as_millis() as u64 > FPS_REPORT_RATE_MS {
                last_fps_report = now;
                let fps = frames_drawn as f64 / elapsed_since_last_fps_report.as_secs_f64();
                last_8_frames.push(fps);

                let fps = last_8_frames.iter().sum::<f64>() / last_8_frames.len() as f64;
                let _ = s.try_send(MsgFromGb::Fps(fps));
                frames_drawn = 0;
            }

            if !turbo {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    });

    (s_to_gb, r_from_gb)
}
