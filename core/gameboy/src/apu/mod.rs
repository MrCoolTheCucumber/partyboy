use crate::cpu::speed_controller::CpuSpeedMode;

use self::{
    frame_sequencer::FrameSequencer,
    square_channel::{Channel1IO, Channel2IO, SquareChannel},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod envelope;
mod frame_sequencer;
mod length;
mod square_channel;
mod sweep;

const SAMPLE_BUFFER_LEN: usize = 512;
const TICKS_PER_SAMPLE: u32 = 87;

pub type Sample = f32;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Apu {
    powered_on: bool,

    sample_buffer: Vec<f64>,
    sample_counter: u32,

    frame_sequencer: FrameSequencer,
    channel_1: SquareChannel,
    channel_2: SquareChannel,

    nr50_raw: u8,
    left_vol: u8,
    right_vol: u8,

    /// Channel panning/mixing
    nr51: u8,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            powered_on: false,
            sample_buffer: Vec::with_capacity(SAMPLE_BUFFER_LEN),
            sample_counter: TICKS_PER_SAMPLE,
            frame_sequencer: FrameSequencer::new(),
            channel_1: SquareChannel::new(),
            channel_2: SquareChannel::new(),
            nr50_raw: 0,
            left_vol: 0,
            right_vol: 0,
            nr51: 0,
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0xFF10..=0xFF14 => self.channel_1.read_u8::<Channel1IO>(addr),
            0xFF16..=0xFF19 => self.channel_2.read_u8::<Channel2IO>(addr),

            0xFF24 => self.nr50_raw,
            0xFF25 => self.nr51,
            0xFF26 => {
                let bit_7 = (self.powered_on as u8) << 7;
                let bit_0 = self.channel_1.enabled() as u8;
                let bit_1 = (self.channel_2.enabled() as u8) << 1;
                bit_7 | bit_0 | bit_1
            }
            _ => unreachable!("Apu doesn't handle reading from address: {:#06X}", addr),
        }
    }
    pub fn write_u8(&mut self, addr: u16, val: u8) {
        if !self.powered_on && addr != 0xFF26 {
            return;
        }

        match addr {
            0xFF10..=0xFF14 => self.channel_1.write_u8::<Channel1IO>(addr, val),
            0xFF16..=0xFF19 => self.channel_2.write_u8::<Channel2IO>(addr, val),

            0xFF24 => {
                self.nr50_raw = val;
                self.right_vol = val & 0b0000_0111;
                self.left_vol = (val & 0b0111_0000) >> 4;
            }
            0xFF25 => self.nr51 = val,
            0xFF26 => self.powered_on = (val & 0b1000_0000) != 0,
            _ => unreachable!("Apu doesn't handle writing to address: {:#06X}", addr),
        };
    }

    pub fn tick(&mut self, div: u8, speed: CpuSpeedMode) -> Option<Sample> {
        self.sample_counter -= 1;
        let sample_this_tick = self.sample_counter == 0;

        if self.sample_counter == 0 {
            self.sample_counter = TICKS_PER_SAMPLE;
        }

        if !self.powered_on {
            return sample_this_tick.then_some(0.0);
        }

        let stepped_components = self.frame_sequencer.tick(div, speed);
        self.channel_1.tick(&stepped_components);
        self.channel_2.tick(&stepped_components);

        sample_this_tick.then_some(self.sample())
    }

    pub fn tick_sample_only(&mut self) -> Option<Sample> {
        self.sample_counter -= 1;
        let sample_this_tick = self.sample_counter == 0;

        if self.sample_counter == 0 {
            self.sample_counter = TICKS_PER_SAMPLE;
        }

        sample_this_tick.then_some(0.0)
    }

    fn sample(&mut self) -> Sample {
        // channel 1
        let ch1_sample = if self.nr51 & 0b0001_0001 != 0 {
            self.channel_1.sample()
        } else {
            0.0
        };

        // channel 2
        let ch2_sample = if self.nr51 & 0b0010_0010 != 0 {
            self.channel_2.sample()
        } else {
            0.0
        };

        // master volume
        // TODO: just keep as normal for now
        let sample = (ch1_sample + ch2_sample) / 2.0;

        sample * 0.1
    }
}
