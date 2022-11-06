use std::time::Duration;

use crossbeam::channel::{Receiver, Sender};
use gameboy::GameBoy;
use spin_sleep::LoopHelper;

use crate::msgs::{MsgFromGb, MsgToGb};

pub fn new(rom: Vec<u8>) -> (Sender<MsgToGb>, Receiver<MsgFromGb>) {
    let (s_to_gb, r_from_ui) = crossbeam::channel::bounded::<MsgToGb>(32);
    let (s_to_ui, r_from_gb) = crossbeam::channel::bounded::<MsgFromGb>(128);

    {
        std::thread::spawn(move || {
            let (s, r) = (s_to_ui, r_from_ui);

            // TODO: make this an option and be able to set rom via msg
            let mut gb = GameBoy::builder()
                .rom(rom)
                .build()
                .expect("Internal error: unable to construct emulator instance");

            // TODO: don't do this. Instead, calculate number of cycles to iterate
            // based on a time delta. Will still need to figure out when "full draws" happen
            let mut loop_helper = LoopHelper::builder()
                .report_interval(Duration::from_millis(500))
                .build_with_target_rate(59.73);

            let mut turbo = false;

            loop {
                loop_helper.loop_start();

                let msgs: Vec<MsgToGb> = r.try_iter().collect();
                for msg in msgs {
                    match msg {
                        MsgToGb::Load => todo!(),
                        MsgToGb::KeyDown(keys) => {
                            log::debug!("{:?}", keys);
                            keys.into_iter().for_each(|key| gb.key_down(key))
                        }
                        MsgToGb::KeyUp(keys) => keys.into_iter().for_each(|key| gb.key_up(key)),
                        MsgToGb::Turbo(state) => turbo = state,
                    }
                }

                while !gb.consume_draw_flag() {
                    gb.tick();
                }

                let frame_msg = MsgFromGb::Frame(gb.get_frame_buffer().into());
                let _ = s.try_send(frame_msg);

                if let Some(fps) = loop_helper.report_rate() {
                    let fps_msg = MsgFromGb::Fps(fps);
                    let _ = s.try_send(fps_msg);
                }

                if !turbo {
                    loop_helper.loop_sleep();
                }
            }
        });
    }

    (s_to_gb, r_from_gb)
}
