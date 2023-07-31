use super::{
    envelope::Envelope,
    frame_sequencer::SteppedComponents,
    length::{Length, LengthMode},
    Sample,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
                if val & 0b1111_0000 == 0 {
                    self.enabled = false;
                }
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
        if stepped_components.length_crtl && matches!(self.length_mode, LengthMode::Timed) {
            self.enabled = !self.length.tick();
        }

        if !self.enabled {
            return;
        }

        if stepped_components.vol_envelope {
            self.envelope.tick();
        }

        self.white_noise_generator.tick();
    }

    pub fn sample(&self) -> Sample {
        let dac_input = match self.enabled && self.white_noise_generator.is_output_high() {
            true => self.envelope.current_vol() as f32,
            false => 0.0,
        };
        (dac_input / 7.5) - 1.0
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum CounterWidth {
    Width15 = 0,
    Width7 = 1,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct WhiteNoiseGenerator {
    dividing_ratio: u8,
    shift_clock: u8,
    counter_width: CounterWidth,
    val: u8,
    noise: u16,
    cycles: u32,
}

impl WhiteNoiseGenerator {
    pub fn new(val: u8) -> Self {
        let dividing_ratio = val & 0b0000_0111;
        let shift_clock = (val & 0b1111_0000) >> 4;
        let counter_width = match val & 0b0000_1000 == 0 {
            true => CounterWidth::Width15,
            false => CounterWidth::Width7,
        };

        let noise = match counter_width {
            CounterWidth::Width15 => (1 << 15) - 1,
            CounterWidth::Width7 => (1 << 7) - 1,
        };

        Self {
            dividing_ratio,
            shift_clock,
            counter_width,
            val,
            noise,

            cycles: 0,
        }
    }

    pub fn tick(&mut self) {
        let tot_cycles = match self.dividing_ratio {
            0 => 8 / 2,
            x => 8 * (x as u32),
        } << (self.shift_clock + 1);

        self.cycles = (self.cycles + 1) % tot_cycles;

        if self.cycles == 0 {
            let shift = self.noise >> 1;
            let carry = (self.noise ^ shift) & 1;

            self.noise = match self.counter_width {
                CounterWidth::Width15 => shift | (carry << 14),
                CounterWidth::Width7 => shift | (carry << 6),
            };
        }
    }

    pub fn is_output_high(&self) -> bool {
        !self.noise & 1 == 1
    }
}
