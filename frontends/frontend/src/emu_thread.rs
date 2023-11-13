use std::{
    collections::VecDeque,
    fs::File,
    io::{Read, Write},
    time::Duration,
};

use common::{bitpacked::BitPackedState, loop_helper::LoopHelper};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleRate, StreamConfig,
};
use crossbeam::channel::{Receiver, Sender};
use gameboy::{GameBoy, SPEED};
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

use crate::msgs::{MsgFromGb, MsgToGb};

const FPS_REPORT_RATE_MS: u64 = 500;

fn take_snapshot(gb: &GameBoy) -> BitPackedState {
    let encoded = bincode::serialize(gb).unwrap();
    let compressed = compress_prepend_size(&encoded);
    let return_val = BitPackedState::pack(compressed);

    // check if snapshot was stable
    {
        let unpacked = return_val.unpack();
        let decompressed = decompress_size_prepended(&unpacked).unwrap();
        let gb_deser: GameBoy = bincode::deserialize(&decompressed).unwrap();
        assert!(*gb == gb_deser);
    }

    return_val
}

fn apply_snapshot(gb: &mut GameBoy, snapshot: &BitPackedState) {
    let mut file = File::create("log/last_snapshot_applied.bin")
        .expect("to be able to open the last snapshot file");
    file.write_all(&snapshot.unpack())
        .expect("to be able to write the last snapshot");

    let unpacked = snapshot.unpack();
    let decompressed = decompress_size_prepended(&unpacked).unwrap();
    let state: GameBoy = bincode::deserialize(&decompressed).unwrap();
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
        let device = host
            .default_output_device()
            .expect("no output device found");

        log::info!("Using audio device: {}", device.name().unwrap());

        let config = StreamConfig {
            channels: 2,
            sample_rate: SampleRate(48000),
            buffer_size: cpal::BufferSize::Fixed(512),
        };

        let (audio_s, audio_r) = crossbeam::channel::bounded::<(f32, f32)>(512 * 16);

        let audio_stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let rx = audio_r.clone();

                    let mut index = 0;
                    while index < data.len() {
                        let sample = rx.try_recv().ok().unwrap_or((0.0, 0.0));

                        data[index] = sample.0;
                        data[index + 1] = sample.1;
                        index += 2;
                    }
                },
                move |_| {},
                None,
            )
            .unwrap();

        audio_stream.play().unwrap();

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

        // DEBUG
        // {
        //     let mut bad_snapshot_file = File::open("log/known_bad_snapshot.bin").unwrap();
        //     let mut snapshot = Vec::new();
        //     let _ = bad_snapshot_file.read_to_end(&mut snapshot).unwrap();

        //     let decompressed = decompress_size_prepended(&snapshot).unwrap();
        //     let state: GameBoy = bincode::deserialize(&decompressed).unwrap();
        //     gb.load_snapshot(state);
        //     gb.release_all_keys();
        // }

        loop {
            // calculate how many ticks have elapsed
            let now = common::time::now();

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

                while audio_s.len() < 512 * 4 {
                    let sample = gb.tick();
                    if let Some(sample) = sample {
                        audio_s.try_send(sample).unwrap();
                    }

                    if gb.consume_draw_flag() {
                        let frame_msg = MsgFromGb::Frame(gb.get_frame_buffer().into());
                        let _ = s.try_send(frame_msg);
                        loop_helper.record_frame_draw();

                        // record state

                        history.push_front(take_snapshot(&gb));
                        if history.len() > 60 * 12 {
                            history.pop_back();
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
                        loop_helper.record_frame_draw();
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
