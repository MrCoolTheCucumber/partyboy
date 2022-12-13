use crate::cpu::speed_controller::CpuSpeedMode;

use self::{
    frame_sequencer::FrameSequencer,
    noise_channel::NoiseChannel,
    sample_channel::SampleChannel,
    square_channel::{Channel1IO, Channel2IO, SquareChannel},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod envelope;
mod frame_sequencer;
mod length;
mod noise_channel;
mod sample_channel;
mod square_channel;
mod sweep;

const SAMPLE_BUFFER_LEN: usize = 512;
const TICKS_PER_SAMPLE: u32 = 87;

pub type Sample = f32;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Apu {
    powered_on: bool,
    capacitor: f32,

    sample_buffer: Vec<f64>,
    sample_counter: u32,

    frame_sequencer: FrameSequencer,
    channel_1: SquareChannel,
    channel_2: SquareChannel,
    channel_3: SampleChannel,
    channel_4: NoiseChannel,

    nr50: u8,
    /// Channel panning/mixing
    nr51: u8,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            powered_on: false,
            capacitor: 0.0,
            sample_buffer: Vec::with_capacity(SAMPLE_BUFFER_LEN),
            sample_counter: TICKS_PER_SAMPLE,
            frame_sequencer: FrameSequencer::new(),
            channel_1: SquareChannel::new(),
            channel_2: SquareChannel::new(),
            channel_3: SampleChannel::new(),
            channel_4: NoiseChannel::new(),
            nr50: 0xFF,
            nr51: 0xFF,
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0xFF10..=0xFF14 => self.channel_1.read_u8::<Channel1IO>(addr),
            0xFF16..=0xFF19 => self.channel_2.read_u8::<Channel2IO>(addr),
            0xFF1A..=0xFF1E => self.channel_3.read_u8(addr),
            0xFF20..=0xFF23 => self.channel_4.read_u8(addr),

            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => {
                let bit_7 = (self.powered_on as u8) << 7;
                let bit_0 = self.channel_1.enabled() as u8;
                let bit_1 = (self.channel_2.enabled() as u8) << 1;
                let bit_2 = (self.channel_3.enabled() as u8) << 2;
                let bit_3 = (self.channel_4.enabled() as u8) << 3;
                bit_7 | bit_0 | bit_1 | bit_2 | bit_3
            }

            0xFF30..=0xFF3F => self.channel_3.read_u8(addr),
            _ => unreachable!("Apu doesn't handle reading from address: {:#06X}", addr),
        }
    }
    pub fn write_u8(&mut self, addr: u16, val: u8) {
        if !self.powered_on && addr != 0xFF26 && addr < 0xFF30 {
            return;
        }

        match addr {
            0xFF10..=0xFF14 => self.channel_1.write_u8::<Channel1IO>(addr, val),
            0xFF16..=0xFF19 => self.channel_2.write_u8::<Channel2IO>(addr, val),
            0xFF1A..=0xFF1E => self.channel_3.write_u8(addr, val),
            0xFF20..=0xFF23 => self.channel_4.write_u8(addr, val),

            0xFF24 => self.nr50 = val,
            0xFF25 => self.nr51 = val,
            0xFF26 => self.powered_on = (val & 0b1000_0000) != 0,

            0xFF30..=0xFF3F => self.channel_3.write_u8(addr, val),
            _ => unreachable!("Apu doesn't handle writing to address: {:#06X}", addr),
        };
    }

    pub fn tick(&mut self, div: u8, speed: CpuSpeedMode) -> Option<(Sample, Sample)> {
        self.sample_counter -= 1;
        let sample_this_tick = self.sample_counter == 0;

        if self.sample_counter == 0 {
            self.sample_counter = TICKS_PER_SAMPLE;
        }

        if !self.powered_on {
            return sample_this_tick.then_some((0.0, 0.0));
        }

        let stepped_components = self.frame_sequencer.tick(div, speed);
        self.channel_1.tick(&stepped_components);
        self.channel_2.tick(&stepped_components);
        self.channel_3.tick(&stepped_components);
        self.channel_4.tick(&stepped_components);

        sample_this_tick.then_some(self.sample())
    }

    pub fn tick_sample_only(&mut self) -> Option<(Sample, Sample)> {
        self.sample_counter -= 1;
        let sample_this_tick = self.sample_counter == 0;

        if self.sample_counter == 0 {
            self.sample_counter = TICKS_PER_SAMPLE;
        }

        sample_this_tick.then_some((0.0, 0.0))
    }

    fn apply_vol_to_raw_sample(sample: Sample, vol: u8) -> Sample {
        let vol = 8.0 - (vol as f32);
        (sample * vol) / 8.0
    }

    fn apply_high_pass(&mut self, sample: Sample) -> Sample {
        let dacs_enabled = self.read_u8(0xFF26) & 0b0000_1111 != 0;
        let mut out = 0.0;
        if dacs_enabled {
            out = sample - self.capacitor;
            self.capacitor = sample - out * 0.998943;
        }
        out
    }

    fn sample(&mut self) -> (Sample, Sample) {
        // TODO: call sample on each channel once, then use those samples for each pan
        let ch1_sample = self.channel_1.sample();
        let ch2_sample = self.channel_2.sample();
        let ch3_sample = self.channel_3.sample();
        let ch4_sample = self.channel_4.sample();

        let mut left_sample = 0.0;
        let mut right_sample = 0.0;

        // Left samples
        if self.nr51 & 0b0001_0000 != 0 {
            left_sample += ch1_sample;
        }

        if self.nr51 & 0b0010_0000 != 0 {
            left_sample += ch2_sample;
        }

        if self.nr51 & 0b0100_0000 != 0 {
            left_sample += ch3_sample;
        }

        if self.nr51 & 0b1000_0000 != 0 {
            left_sample += ch4_sample;
        }

        // Right samples
        if self.nr51 & 0b0000_0001 != 0 {
            right_sample += ch1_sample;
        }

        if self.nr51 & 0b0000_0010 != 0 {
            right_sample += ch2_sample;
        }

        if self.nr51 & 0b0000_0100 != 0 {
            right_sample += ch3_sample;
        }

        if self.nr51 & 0b0000_1000 != 0 {
            right_sample += ch4_sample;
        }

        let left_vol = (self.nr50 & 0b0111_0000) >> 4;
        let right_vol = self.nr50 & 0b0000_0111;

        // left_sample /= 4.0;
        // right_sample /= 4.0;

        left_sample = Self::apply_vol_to_raw_sample(left_sample, left_vol);
        right_sample = Self::apply_vol_to_raw_sample(right_sample, right_vol);

        left_sample = self.apply_high_pass(left_sample);
        right_sample = self.apply_high_pass(right_sample);

        // mult by 0.3 to simulate physical volume slider only being slightly on
        (left_sample, right_sample)
    }
}
