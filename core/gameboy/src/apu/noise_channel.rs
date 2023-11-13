use super::{
    envelope::Envelope,
    frame_sequencer::SteppedComponents,
    length::{Length, LengthMode},
    Sample,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq)]
pub struct NoiseChannel {
    enabled: bool,
    white_noise_generator: WhiteNoiseGenerator,
    envelope: Envelope,
    length: Length,
    length_mode: LengthMode,

    nrx1: u8,
    nrx2: u8,
    nr43: u8,
    nrx4: u8,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            white_noise_generator: WhiteNoiseGenerator::new(0xFF),
            envelope: Envelope::default(),
            length: Length::new(64),
            length_mode: LengthMode::Infinite,
            nrx1: 0,
            nrx2: 0,
            nr43: 0,
            nrx4: 0,
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0xFF20 => self.nrx1,
            0xFF21 => self.nrx2,
            0xFF22 => self.nr43,
            0xFF23 => self.nrx4 | 0b1011_1111,
            _ => panic!(),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF20 => self.nrx1 = val,
            0xFF21 => self.nrx2 = val,
            0xFF22 => {
                self.nr43 = val;
                self.white_noise_generator = WhiteNoiseGenerator::new(self.nr43);
            }
            0xFF23 => {
                self.nrx4 = val;

                let length_enabled = val & 0b0100_0000 != 0;
                self.length_mode = (length_enabled as u8).into();
                if length_enabled {
                    self.length.init(self.nrx1 & 0b0011_1111);
                }

                let channel_triggered = val & 0b1000_0000 != 0;
                if channel_triggered {
                    self.enabled = true;
                    self.envelope.init(self.nrx2);
                    self.white_noise_generator = WhiteNoiseGenerator::new(self.nr43);
                }
            }
            _ => panic!(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn tick(&mut self, stepped_components: &SteppedComponents) {
        if !self.enabled {
            return;
        }

        if matches!(self.length_mode, LengthMode::Timed) && stepped_components.length_crtl {
            self.enabled = !self.length.tick();
        }

        if stepped_components.vol_envelope {
            self.envelope.tick();
        }

        self.white_noise_generator.tick();
    }

    pub fn sample(&self) -> Sample {
        if self.enabled {
            let input = self.white_noise_generator.sample() * self.envelope.current_vol();
            (input as f32 / 7.5) - 1.0
        } else {
            0.0
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq)]
enum CounterWidth {
    Width15 = 0,
    Width7 = 1,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq)]
struct WhiteNoiseGenerator {
    divisor_code: u8,
    shift_amount: u8,
    width_mode: CounterWidth,
    val: u8,
    lsfr: u16,
    freq_timer: u32,
}

impl WhiteNoiseGenerator {
    pub fn new(val: u8) -> Self {
        let divisor_code = val & 0b0000_0111;
        let shift_amount = (val & 0b1111_0000) >> 4;
        let width_mode = match val & 0b0000_1000 == 0 {
            true => CounterWidth::Width15,
            false => CounterWidth::Width7,
        };

        Self {
            divisor_code,
            shift_amount,
            width_mode,
            val,
            lsfr: 0b0111_1111_1111_1111,
            freq_timer: Self::into_divisor(divisor_code) << shift_amount,
        }
    }

    fn into_divisor(divisor_code: u8) -> u32 {
        match divisor_code {
            0 => 8,
            x => (x as u32) << 4,
        }
    }

    pub fn tick(&mut self) {
        self.freq_timer -= 1;
        if self.freq_timer != 0 {
            return;
        }

        self.freq_timer = Self::into_divisor(self.divisor_code) << self.shift_amount;

        let bit = (self.lsfr & 0b01) ^ ((self.lsfr & 0b10) >> 1);
        match self.width_mode {
            CounterWidth::Width7 => self.lsfr = ((self.lsfr >> 1) & !0x40) | (bit << 6),
            CounterWidth::Width15 => self.lsfr = ((self.lsfr >> 1) & !0x4000) | (bit << 14),
        }
    }

    pub fn sample(&self) -> u8 {
        (!self.lsfr & 1) as u8
    }
}
