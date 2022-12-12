use crate::cpu::speed_controller::CpuSpeedMode;

pub struct SteppedComponents {
    length_crtl: bool,
    vol_envelope: bool,
    sweep: bool,
}

impl From<u32> for SteppedComponents {
    fn from(val: u32) -> Self {
        Self {
            length_crtl: val % 2 != 0,
            vol_envelope: val == 8,
            sweep: val == 3 || val == 7,
        }
    }
}

fn is_falling_edge(prev: bool, current: bool) -> bool {
    prev && !current
}

pub struct FrameSequencer {
    cycle: u32,
    last_bit_5_hi: bool,
    last_bit_6_hi: bool,
}

impl FrameSequencer {
    pub fn new() -> Self {
        FrameSequencer {
            cycle: 0,
            last_bit_5_hi: false,
            last_bit_6_hi: false,
        }
    }

    pub fn step_cycle(&mut self) {
        self.cycle += 1;
        if self.cycle == 9 {
            self.cycle = 1;
        }
    }

    pub fn tick(&mut self, div: u8, speed: CpuSpeedMode) -> Option<SteppedComponents> {
        let is_b5_hi = div & (1 << 5) != 0;
        let is_b6_hi = div & (1 << 6) != 0;

        let b5_falling_edge = is_falling_edge(self.last_bit_5_hi, is_b5_hi);
        let b6_falling_edge = is_falling_edge(self.last_bit_6_hi, is_b6_hi);

        self.last_bit_5_hi = is_b5_hi;
        self.last_bit_6_hi = is_b6_hi;

        match speed {
            CpuSpeedMode::Single if b5_falling_edge => {
                self.step_cycle();
                Some(self.cycle.into())
            }
            CpuSpeedMode::Double if b6_falling_edge => {
                self.step_cycle();
                Some(self.cycle.into())
            }
            _ => None,
        }
    }
}
