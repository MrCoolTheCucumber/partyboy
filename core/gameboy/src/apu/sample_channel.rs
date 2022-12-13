use super::{
    frame_sequencer::SteppedComponents,
    length::{Length, LengthMode},
    Sample,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SampleChannel {
    enabled: bool,
    playing: bool,
    samples: [u8; 32],
    sample_index: usize,
    length: Length,
    length_mode: LengthMode,

    nrx1: u8,
    nr32: u8,
    written_freqency: u16,
    written_freq_flag: bool,
    nr34: u8,

    frequency: u16,
    frequency_timer: u16,
}

impl SampleChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            playing: false,
            samples: [0; 32],
            sample_index: 0,
            length: Length::new(256),
            length_mode: LengthMode::Infinite,
            nrx1: 0,
            nr32: 0,
            written_freqency: 0,
            written_freq_flag: false,
            nr34: 0,
            frequency: 0,
            frequency_timer: 0,
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0xFF1A => (self.enabled as u8) << 7,
            0xFF1B => self.nrx1,
            0xFF1C => self.nr32 | 0b1001_0000,
            0xFF1D => 0xFF,
            0xFF1E => self.nr34 | 0b1011_1111,
            0xFF30..=0xFF3F => {
                let index = ((addr - 0xFF30) * 2) as usize;
                let sample_hi = self.samples[index];
                let sample_lo = self.samples[index + 1];
                (sample_hi << 4) | sample_lo
            }
            _ => unreachable!(
                "Channel 3 doesn't support reading from address: {:#06X}",
                addr
            ),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF1A => {
                self.enabled = val & 0b1000_0000 != 0;
                if !self.enabled {
                    self.playing = false;
                }
            }
            0xFF1B => self.nrx1 = val,
            0xFF1C => self.nr32 = val,
            0xFF1D => {
                self.written_freqency =
                    (self.written_freqency & 0b1111_1111_0000_0000) | (val as u16);
                self.written_freq_flag = true;
            }
            0xFF1E => {
                let freq_high_3_bits = val & 0b0000_0111;
                self.written_freqency = (self.written_freqency & 0b0000_0000_1111_1111)
                    | ((freq_high_3_bits as u16) << 8);
                self.written_freq_flag = true;

                let length_enabled = val & 0b0100_0000 != 0;
                self.length_mode = (length_enabled as u8).into();

                let channel_triggered = val & 0b1000_0000 != 0;
                if channel_triggered {
                    self.enabled = true;
                    self.playing = true;
                    self.frequency_timer = (2048 - self.frequency) * 2;
                    if length_enabled {
                        self.length.init(self.nrx1 & val & 0b0011_1111);
                    }
                }
            }
            0xFF30..=0xFF3F => {
                let index = ((addr - 0xFF30) * 2) as usize;
                let sample_hi = val >> 4;
                let sample_lo = val & 0b0000_1111;

                self.samples[index] = sample_hi;
                self.samples[index] = sample_lo;
            }
            _ => unreachable!(
                "Channel 3 doesn't support writing to address: {:#06X}",
                addr
            ),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    fn tick_frequency(&mut self) {
        match self.frequency_timer {
            0 => {
                self.sample_index = (self.sample_index + 1) % 32;

                if self.written_freq_flag && self.sample_index % 2 == 0 {
                    self.frequency = self.written_freqency;
                }

                self.frequency_timer = (2048 - self.frequency) * 2;
            }
            _ => self.frequency_timer -= 1,
        }
    }

    pub fn tick(&mut self, stepped_components: &SteppedComponents) {
        if stepped_components.length_crtl && matches!(self.length_mode, LengthMode::Timed) {
            self.enabled = !self.length.tick();
            self.playing = self.enabled;
        }

        if !self.playing {
            return;
        }

        self.tick_frequency();
    }

    pub fn sample(&self) -> Sample {
        if !self.playing {
            return 0.0;
        }

        let current_sample = self.samples[self.sample_index];
        let output_level = (self.nr32 & 0b0110_0000) >> 5;

        let output = match output_level {
            0 => 0,
            1 => current_sample,
            2 => current_sample / 2,
            3 => current_sample / 4,
            _ => unreachable!(),
        };

        (output as Sample / 7.5) - 1.0
    }
}
