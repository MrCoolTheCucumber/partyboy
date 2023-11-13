#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq)]
enum SweepDirection {
    Increase,
    Decrease,
}

impl From<u8> for SweepDirection {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Increase,
            1 => Self::Decrease,
            _ => unreachable!("Invalid sweep direction value"),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq)]
pub struct Sweep {
    enabled: bool,
    shadow_freq: u16,
    timer: u8,
    period: u8,
    direction: SweepDirection,
    slope: u8,
}

impl Default for Sweep {
    fn default() -> Self {
        Self {
            enabled: false,
            direction: SweepDirection::Increase,
            shadow_freq: 0,
            timer: 0,
            period: 0,
            slope: 0,
        }
    }
}

impl Sweep {
    pub fn new(nr10: u8, freq: u16) -> Self {
        let period = (nr10 & 0b0111_0000) >> 4;
        let direction: SweepDirection = ((nr10 & 0b0000_1000) >> 3).into();
        let slope = nr10 & 0b0000_0111;

        Self {
            enabled: period != 0 || slope != 0,
            shadow_freq: freq,
            timer: if period != 0 { period } else { 8 },
            period,
            direction,
            slope,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn calc_freq(&mut self, freq: u16) -> Option<u16> {
        let mut new_freq = freq >> self.slope;

        match self.direction {
            SweepDirection::Increase => new_freq += freq,
            SweepDirection::Decrease => new_freq = freq - new_freq,
        }

        if new_freq > 2047 {
            self.enabled = false;
            return None;
        }

        Some(new_freq)
    }

    pub fn tick(&mut self, freq: u16) -> Option<u16> {
        if !self.enabled {
            return Some(freq);
        }

        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer != 0 {
            return Some(freq);
        }

        if self.period > 0 {
            self.timer = self.period;
        } else {
            self.timer = 8;
        }

        if self.enabled && self.period > 0 {
            let Some(new_freq) = self.calc_freq(freq) else {
                return None;
            };

            if new_freq <= 2047 && self.slope > 0 && self.calc_freq(new_freq).is_none() {
                return None;
            }

            return Some(new_freq);
        }

        Some(freq)
    }
}
