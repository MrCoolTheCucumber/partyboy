use std::{collections::VecDeque, time::Duration};

use common::{bitpacked::BitPackedState, loop_helper::LoopHelper};
use crossbeam::channel::{Receiver, Sender};
use gameboy::{GameBoy, SPEED};
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer, RingBufferExt, RingBufferWrite};

use crate::msgs::{MsgFromGb, MsgToGb};

const FPS_REPORT_RATE_MS: u64 = 500;

fn take_snapshot(gb: &GameBoy) -> BitPackedState {
    let encoded = rmp_serde::to_vec(&gb).unwrap();
    let compressed = compress_prepend_size(&encoded);
    BitPackedState::pack(compressed)
}

fn apply_snapshot(gb: &mut GameBoy, snapshot: &BitPackedState) {
    let unpacked = snapshot.unpack();
    let decompressed = decompress_size_prepended(&unpacked).unwrap();
    let state: GameBoy = rmp_serde::from_slice(&decompressed).unwrap();
    gb.load_snapshot(state);
    gb.release_all_keys();
}

pub fn new(rom: Option<Vec<u8>>, bios: Option<Vec<u8>>) -> (Sender<MsgToGb>, Receiver<MsgFromGb>) {
    let (s_to_gb, r_from_ui) = crossbeam::channel::bounded::<MsgToGb>(32);
    let (s_to_ui, r_from_gb) = crossbeam::channel::bounded::<MsgFromGb>(128);

    std::thread::spawn(move || {
        let (s, r) = (s_to_ui, r_from_ui);

        // TODO: make this an option and be able to set rom via msg
        let mut builder = GameBoy::builder();
        // TODO: make builder take optionals?
        if let Some(rom) = rom {
            builder = builder.rom(rom);
        }
        if let Some(bios) = bios {
            builder = builder.bios(bios);
        }
        let mut gb = builder
            .build()
            .expect("Internal error: unable to construct emulator instance");

        let mut turbo = false;
        let mut snapshot: Option<BitPackedState> = None;

        let mut history = VecDeque::new();
        let mut rewind = false;

        let mut loop_helper = LoopHelper::new(FPS_REPORT_RATE_MS, SPEED);

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

                        if state {
                            history.clear();
                        }
                    }
                    MsgToGb::Rewind(state) => {
                        rewind = state;
                    }
                    MsgToGb::SaveSnapshot => {
                        let state = take_snapshot(&gb);
                        snapshot = Some(state);
                    }
                    MsgToGb::LoadSnapshot => {
                        if let Some(state) = &snapshot {
                            apply_snapshot(&mut gb, state);
                            log::info!("Loaded snapshot")
                        }
                    }
                }
            }

            // calculate how many ticks have elapsed
            let now = common::time::now();
            let ticks = loop_helper.calculate_ticks_to_run(now, turbo);

            'tick_emulator: {
                if rewind {
                    break 'tick_emulator;
                }

                for _ in 0..ticks {
                    gb.tick();
                    if gb.consume_draw_flag() {
                        let frame_msg = MsgFromGb::Frame(gb.get_frame_buffer().into());
                        let _ = s.try_send(frame_msg);
                        loop_helper.record_frame_draw();

                        // record state
                        if !turbo {
                            history.push_front(take_snapshot(&gb));
                            if history.len() > 60 * 12 {
                                history.pop_back();
                            }
                        }
                    }
                }
            }

            if rewind {
                if let Some(state) = history.pop_front() {
                    apply_snapshot(&mut gb, &state);
                    let frame_msg = MsgFromGb::Frame(gb.get_frame_buffer().into());
                    let _ = s.try_send(frame_msg);
                }
            }

            // check if we should report fps
            if let Some(fps) = loop_helper.report_fps(now) {
                last_8_frames.push(fps);
                let fps = last_8_frames.iter().sum::<f64>() / last_8_frames.len() as f64;
                let _ = s.try_send(MsgFromGb::Fps(fps));
            }

            if !turbo {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    });

    (s_to_gb, r_from_gb)
}
