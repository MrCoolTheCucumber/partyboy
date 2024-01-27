use super::{
    envelope::Envelope,
    frame_sequencer::SteppedComponents,
    length::{Length, LengthMode},
    sweep::Sweep,
    Sample,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

static DUTY_LUT: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [0, 0, 0, 0, 0, 0, 1, 1], // 25%
    [0, 0, 0, 0, 1, 1, 1, 1], // 50%
    [1, 1, 1, 1, 1, 1, 0, 0], // 75%
];

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SquareChannel {
    enabled: bool,

    nr10: u8,
    nrx1: u8,
    nrx2: u8,
    nrx3: u8,
    nrx4: u8,

    frequency: u16,
    frequency_timer: u16,
    duty_index: usize,

    envelope: Envelope,
    length: Length,
    length_mode: LengthMode,
    sweep: Sweep,

    last_duty: u8,
}

impl SquareChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            nr10: 0,
            nrx1: 0,
            nrx2: 0,
            nrx3: 0,
            nrx4: 0,
            frequency: 0,
            frequency_timer: 0,
            duty_index: 0,
            envelope: Envelope::default(),
            length: Length::new(64),
            length_mode: LengthMode::Infinite,
            sweep: Sweep::new(0, 0),
            last_duty: 0,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn read_u8<T: SquareChannelIO>(&self, addr: u16) -> u8 {
        T::read_u8(self, addr)
    }

    pub fn write_u8<T: SquareChannelIO>(&mut self, addr: u16, val: u8) {
        T::write_u8(self, addr, val);
    }

    fn get_duty(&self) -> usize {
        ((self.nrx1 & 0b1100_0000) >> 6) as usize
    }

    fn get_amplitude(&self) -> u8 {
        DUTY_LUT[self.get_duty()][self.duty_index]
    }

    fn tick_freq(&mut self) {
        match self.frequency_timer {
            0 => {
                self.frequency_timer = (2048 - self.frequency) * 4;
                self.duty_index += 1;

                if self.duty_index == 8 {
                    self.duty_index = 0;
                }
            }
            _ => self.frequency_timer -= 1,
        }
    }

    pub fn tick(&mut self, stepped_components: &SteppedComponents) {
        if !self.enabled {
            return;
        }

        // according to frosty, length is ticked even if channel is "disabled"?
        if stepped_components.length_crtl && matches!(self.length_mode, LengthMode::Timed) {
            self.enabled = !self.length.tick();
        }

        if stepped_components.vol_envelope {
            self.envelope.tick();
        }

        if self.sweep.is_enabled() && stepped_components.sweep {
            self.frequency = match self.sweep.tick(self.frequency) {
                Some(freq) => freq,
                None => {
                    self.enabled = false;
                    return;
                }
            }
        }

        self.tick_freq();
    }

    pub fn sample(&self) -> Sample {
        let dac_input = (self.get_amplitude() * self.envelope.current_vol()) as Sample;
        (dac_input / 7.5) - 1.0
    }
}

pub trait SquareChannelIO {
    fn read_u8(channel: &SquareChannel, addr: u16) -> u8;
    fn write_u8(channel: &mut SquareChannel, addr: u16, val: u8);
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Channel1IO;

impl SquareChannelIO for Channel1IO {
    fn read_u8(channel: &SquareChannel, addr: u16) -> u8 {
        match addr {
            0xFF10 => channel.nr10,
            0xFF11 => channel.nrx1 | 0b0011_1111,
            0xFF12 => channel.nrx2,
            0xFF13 => 0b1111_1111,
            0xFF14 => channel.nrx4 | 0b1011_1111,

            _ => unreachable!(
                "Channel 1 doesn't handle reading from address: {:#06X}",
                addr
            ),
        }
    }

    fn write_u8(channel: &mut SquareChannel, addr: u16, val: u8) {
        match addr {
            0xFF10 => channel.nr10 = val,
            0xFF11 => channel.nrx1 = val,
            0xFF12 => channel.nrx2 = val,
            0xFF13 => {
                channel.nrx3 = val;
                channel.frequency = (channel.frequency & 0b1111_1111_0000_0000) | (val as u16);
            }
            0xFF14 => {
                channel.nrx4 = val;
                let freq_high_3_bits = val & 0b0000_0111;
                channel.frequency =
                    (channel.frequency & 0b0000_0000_1111_1111) | ((freq_high_3_bits as u16) << 8);

                let length_enabled = val & 0b0100_0000 != 0;
                channel.length_mode = (length_enabled as u8).into();
                if length_enabled {
                    channel.length.init(channel.nrx1 & 0b0011_1111)
                }

                let channel_triggered = val & 0b1000_0000 != 0;
                if channel_triggered {
                    channel.enabled = true;
                    channel.envelope.init(channel.nrx2);
                    channel.sweep = Sweep::new(channel.nr10, channel.frequency);
                    channel.frequency_timer = (2048 - channel.frequency) * 4;
                }
            }

            _ => unreachable!("Channel 1 doesn't handle writing to address: {:#06X}", addr),
        };
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Channel2IO;

impl SquareChannelIO for Channel2IO {
    fn read_u8(channel: &SquareChannel, addr: u16) -> u8 {
        match addr {
            0xFF16 => channel.nrx1 | 0b0011_1111,
            0xFF17 => channel.nrx2,
            0xFF18 => 0b1111_1111,
            0xFF19 => channel.nrx4 | 0b1011_1111,

            _ => unreachable!(
                "Channel 2 doesn't handle reading from address: {:#06X}",
                addr
            ),
        }
    }

    fn write_u8(channel: &mut SquareChannel, addr: u16, val: u8) {
        match addr {
            0xFF16 => channel.nrx1 = val,
            0xFF17 => channel.nrx2 = val,
            0xFF18 => {
                channel.nrx3 = val;
                channel.frequency = (channel.frequency & 0b1111_1111_0000_0000) | (val as u16);
            }
            0xFF19 => {
                channel.nrx4 = val;
                let freq_high_3_bits = val & 0b0000_0111;
                channel.frequency =
                    (channel.frequency & 0b0000_0000_1111_1111) | ((freq_high_3_bits as u16) << 8);

                let length_enabled = val & 0b0100_0000 != 0;
                channel.length_mode = (length_enabled as u8).into();
                if length_enabled {
                    channel.length.init(channel.nrx1 & 0b0011_1111)
                }

                let channel_triggered = val & 0b1000_0000 != 0;
                if channel_triggered {
                    channel.enabled = true;
                    channel.envelope.init(channel.nrx2);
                    channel.frequency_timer = (2048 - channel.frequency) * 4;
                }
            }

            _ => unreachable!("Channel 2 doesn't handle writing to address: {:#06X}", addr),
        };
    }
}
