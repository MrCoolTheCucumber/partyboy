#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Length {
    enabled: bool,
    base_length: u16,
    length_timer: u16,
}

impl Length {
    pub fn new(base_length: u16) -> Self {
        Self {
            enabled: false,
            base_length,
            length_timer: 0,
        }
    }

    pub fn init(&mut self, initial_length_timer: u8) {
        self.length_timer = self.base_length.wrapping_sub(initial_length_timer as u16);
        self.enabled = true;
    }

    /// Returns true if the length reaches 0
    pub fn tick(&mut self) -> bool {
        if self.enabled {
            self.length_timer -= 1;
            self.enabled = self.length_timer != 0;
            return self.length_timer == 0;
        }

        false
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LengthMode {
    Infinite = 0,
    Timed = 1,
}

impl From<u8> for LengthMode {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Infinite,
            1 => Self::Timed,
            _ => unreachable!("Invalid lengthmode value"),
        }
    }
}
