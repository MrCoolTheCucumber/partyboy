use crate::cpu::speed_controller::CpuSpeedMode;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteppedComponents {
    pub length_crtl: bool,
    pub vol_envelope: bool,
    pub sweep: bool,
}

impl SteppedComponents {
    pub fn none() -> Self {
        Self {
            length_crtl: false,
            vol_envelope: false,
            sweep: false,
        }
    }
}

impl From<u32> for SteppedComponents {
    fn from(val: u32) -> Self {
        Self {
            length_crtl: val % 2 != 0,
            vol_envelope: val == 7,
            sweep: val == 2 || val == 6,
        }
    }
}

fn is_falling_edge(prev: bool, current: bool) -> bool {
    prev && !current
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
        if self.cycle == 8 {
            self.cycle = 0;
        }
    }

    pub fn tick(&mut self, div: u8, speed: CpuSpeedMode) -> SteppedComponents {
        // From https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html
        // >> The frame sequencer clocks are derived from the DIV timer. In Normal Speed Mode,
        // >> falling edges of bit 5 step the FS while in CGB Double Speed Mode, bit 6 is used
        // >> instead. Here bits 5 and 6 refer to the bits of the upper byte of DIV (internally
        // >> DIV is 16 bit but only the upper 8 bits are mapped to memory).

        let is_b5_hi = div & (1 << 4) != 0;
        let is_b6_hi = div & (1 << 5) != 0;

        let b5_falling_edge = is_falling_edge(self.last_bit_5_hi, is_b5_hi);
        let b6_falling_edge = is_falling_edge(self.last_bit_6_hi, is_b6_hi);

        self.last_bit_5_hi = is_b5_hi;
        self.last_bit_6_hi = is_b6_hi;

        match speed {
            CpuSpeedMode::Single if b5_falling_edge => {
                let stepped = self.cycle.into();
                self.step_cycle();
                stepped
            }
            CpuSpeedMode::Double if b6_falling_edge => {
                let stepped = self.cycle.into();
                self.step_cycle();
                stepped
            }
            _ => SteppedComponents::none(),
        }
    }
}
