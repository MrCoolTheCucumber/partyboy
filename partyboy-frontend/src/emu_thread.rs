use std::{collections::VecDeque, time::Duration};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleRate, StreamConfig,
};
use crossbeam::channel::{Receiver, Sender};
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use partyboy_common::loop_helper::LoopHelper as ReportHelper;
use partyboy_core::{GameBoy, SPEED};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

use crate::msgs::{MsgFromGb, MsgToGb};

const FPS_REPORT_RATE_MS: u64 = 500;

fn take_snapshot(gb: &GameBoy) -> Vec<u8> {
    let encoded = rmp_serde::to_vec(&gb).unwrap();
    compress_prepend_size(&encoded)
}

fn apply_snapshot(gb: &mut GameBoy, snapshot: &[u8]) {
    let decompressed = decompress_size_prepended(snapshot).unwrap();
    let state: GameBoy = rmp_serde::from_slice(&decompressed).unwrap();
    gb.load_snapshot(state);
    gb.release_all_keys();
}

pub fn new(rom: Option<Vec<u8>>, bios: Option<Vec<u8>>) -> (Sender<MsgToGb>, Receiver<MsgFromGb>) {
    let (s_to_gb, r_from_ui) = crossbeam::channel::bounded::<MsgToGb>(32);
    let (s_to_ui, r_from_gb) = crossbeam::channel::bounded::<MsgFromGb>(128);

    std::thread::spawn(move || {
        // set up audio
        let hosts = cpal::available_hosts().len();
        log::debug!("{}", hosts);

        let host = cpal::default_host();

        let audio_devices_found = match host.output_devices() {
            Ok(devices) => devices.count(),
            Err(_) => 0,
        };

        log::info!("Audio devices found: {audio_devices_found}");

        let (audio_s, audio_r) = crossbeam::channel::bounded::<(f32, f32)>(512 * 16);

        let _: Option<_> = if audio_devices_found > 0 {
            let device = host
                .default_output_device()
                .expect("no output device found");

            log::info!("Using audio device: {}", device.name().unwrap());

            let config = StreamConfig {
                channels: 2,
                sample_rate: SampleRate(48000),
                buffer_size: cpal::BufferSize::Fixed(512),
            };

            let audio_stream = device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], i: &cpal::OutputCallbackInfo| {
                        println!("?");
                        let audio_r = audio_r.clone();
                        let mut index = 0;
                        while index < data.len() {
                            let sample = audio_r.try_recv().ok().unwrap_or((0.0, 0.0));

                            data[index] = sample.0;
                            data[index + 1] = sample.1;
                            index += 2;
                        }
                    },
                    move |e| {
                        println!("Stream error: {e:?}");
                    },
                    Some(Duration::from_secs(1)),
                )
                .unwrap();

            audio_stream.play().unwrap();

            log::info!("created audio stream");
            Some(audio_stream)
        } else {
            log::info!("No audio device found, dropping all audio produced");
            None
        };

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
        let mut snapshot: Option<Vec<u8>> = None;

        let mut history = VecDeque::new();
        let mut rewind = false;

        let mut report_helper = ReportHelper::new(FPS_REPORT_RATE_MS, SPEED);
        #[allow(deprecated)]
        let mut loop_helper = spin_sleep::LoopHelper::builder()
            .report_interval(Duration::from_millis(500))
            .build_with_target_rate(59.73);

        let mut last_8_frames = ConstGenericRingBuffer::<_, 8>::new();

        loop {
            loop_helper.loop_start();

            // calculate how many ticks have elapsed
            let now = partyboy_common::time::now();

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

            'tick_emulator: {
                if rewind || turbo {
                    break 'tick_emulator;
                }

                if audio_devices_found > 0 {
                    while audio_s.len() < 512 * 4 {
                        let sample = gb.tick();
                        if let Some(sample) = sample {
                            audio_s.try_send(sample).unwrap();
                        }

                        if gb.consume_draw_flag() {
                            let frame_msg = MsgFromGb::Frame(gb.get_frame_buffer().into());
                            let _ = s.try_send(frame_msg);
                            report_helper.record_frame_draw();

                            // record state
                            if !turbo {
                                history.push_front(take_snapshot(&gb));
                                if history.len() > 60 * 12 {
                                    history.pop_back();
                                }
                            }
                        }
                    }
                } else {
                    loop {
                        gb.tick();
                        if gb.consume_draw_flag() {
                            let frame_msg = MsgFromGb::Frame(gb.get_frame_buffer().into());
                            let _ = s.try_send(frame_msg);
                            report_helper.record_frame_draw();

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
            }

            if turbo && !rewind {
                loop {
                    gb.tick();
                    if gb.consume_draw_flag() {
                        let frame_msg = MsgFromGb::Frame(gb.get_frame_buffer().into());
                        let _ = s.try_send(frame_msg);
                        report_helper.record_frame_draw();
                        break;
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
            if let Some(fps) = report_helper.report_fps(now) {
                last_8_frames.push(fps);
                let fps = last_8_frames.iter().sum::<f64>() / last_8_frames.len() as f64;
                let _ = s.try_send(MsgFromGb::Fps(fps));
            }

            if !turbo && audio_devices_found > 0 {
                std::thread::sleep(Duration::from_millis(1));
            } else if !turbo && audio_devices_found == 0 {
                loop_helper.loop_sleep();
            }
        }
    });

    (s_to_gb, r_from_gb)
}
