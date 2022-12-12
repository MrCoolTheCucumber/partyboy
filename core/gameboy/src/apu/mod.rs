mod channel_2;
mod envelope;
mod frame_sequencer;

struct Apu {
    powered_on: bool,

    nr50_raw: u8,
    left_vol: u8,
    right_vol: u8,

    /// Channel panning/mixing
    nr51_raw: u8,
}

impl Apu {
    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0xFF24 => self.nr50_raw,
            0xFF25 => self.nr51_raw,
            0xFF26 => todo!(),
            _ => unreachable!("Apu doesn't handle reading from address: {:#06X}", addr),
        }
    }
    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF24 => {
                self.nr50_raw = val;
                self.right_vol = val & 0b0000_0111;
                self.left_vol = (val & 0b0111_0000) >> 4;
            }
            0xFF25 => self.nr51_raw = val,
            0xFF26 => self.powered_on = (val & 0b1000_0000) != 0,
            _ => unreachable!("Apu doesn't handle writing to address: {:#06X}", addr),
        };
    }
}
