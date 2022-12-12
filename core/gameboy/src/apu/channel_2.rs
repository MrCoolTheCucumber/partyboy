const DutyLUT: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [0, 0, 0, 0, 0, 0, 1, 1], // 25%
    [0, 0, 0, 0, 1, 1, 1, 1], // 50%
    [1, 1, 1, 1, 1, 1, 0, 0], // 75%
];

struct Channel2 {
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,

    frequency_timer: u16,
    duty_index: usize,
}

impl Channel2 {
    pub fn new() -> Self {
        Self {
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            frequency_timer: 0,
            duty_index: 0,
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0xFF16 => self.nr21, // TODO: bits 0-5 are not readable, what to return?
            0xFF17 => self.nr22,
            0xFF18 => self.nr23,
            0xFF19 => self.nr24,

            _ => unreachable!(
                "Channel 2 doesn't handle reading from address: {:#06X}",
                addr
            ),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF16 => self.nr21 = val,
            0xFF17 => self.nr22 = val,
            0xFF18 => self.nr23 = val,
            0xFF19 => self.nr24 = val,

            _ => unreachable!("Channel 2 doesn't handle writing to address: {:#06X}", addr),
        };
    }

    fn get_duty(&self) -> usize {
        ((self.nr21 & 0b1100_0000) >> 6) as usize
    }

    fn get_frequency(&self) -> u16 {
        let lo = self.nr23;
        let hi = self.nr24 & 0b0000_0111;
        ((hi as u16) << 8) | (lo as u16)
    }

    fn get_amplitude(&self) -> u8 {
        DutyLUT[self.get_duty()][self.duty_index]
    }

    fn tick_freq(&mut self) {
        // TODO: return if disabled?

        match self.frequency_timer {
            0 => {
                self.frequency_timer = (2048 - self.get_frequency()) * 4;
                self.duty_index += 1;
                self.duty_index &= 0b111;
            }
            _ => self.frequency_timer -= 1,
        }
    }
}
