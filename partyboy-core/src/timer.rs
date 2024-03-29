use crate::interrupts::{InterruptFlag, Interrupts};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,

    tac_freq_bits: [u8; 4],
    tima_overflown: bool,
    ticks_since_tima_overflown: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0x0008, // 5, 6, 7, 8 ok
            tima: 0,
            tma: 0,
            tac: 0,

            tac_freq_bits: [9, 3, 5, 7],
            tima_overflown: false,
            ticks_since_tima_overflown: 0,
        }
    }

    pub fn div(&self) -> u8 {
        (self.div >> 8) as u8
    }

    #[inline(always)]
    fn is_timer_enabled(&self) -> bool {
        self.tac & 0b0000_0100 != 0
    }

    pub fn tick(&mut self, interrupts: &mut Interrupts) {
        let prev_div = self.div;
        self.div += 1;

        if self.is_timer_enabled() && self.div_falling_edge_occured(prev_div, self.div) {
            self.incr_tima();
        }

        if self.tima_overflown {
            self.ticks_since_tima_overflown += 1;
        }

        if self.ticks_since_tima_overflown == 1 {
            interrupts.request_interupt(InterruptFlag::Timer)
        } else if self.ticks_since_tima_overflown == 5 {
            self.tima = self.tma
        } else if self.ticks_since_tima_overflown == 6 {
            self.tima = self.tma;
            self.tima_overflown = false;
            self.ticks_since_tima_overflown = 0;
        }
    }

    // "falling edge" = previously 1, now 0
    #[inline(always)]
    fn div_falling_edge_occured(&mut self, prev_div: u16, current_div: u16) -> bool {
        let bit_to_check = self.tac_freq_bits[(self.tac & 3) as usize];
        let prev_bit = (prev_div >> bit_to_check) & 1;
        let bit = (current_div >> bit_to_check) & 1;

        prev_bit == 1 && bit == 0
    }

    #[inline(always)]
    fn incr_tima(&mut self) {
        self.tima += 1;

        if self.tima == 0 {
            self.tima_overflown = true;
            self.ticks_since_tima_overflown = 0;
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF03 => self.div as u8,
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0b11111000,

            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF03 | 0xFF04 => {
                let prev_div = self.div;
                self.div = 0;

                let falling_edge_occured = self.div_falling_edge_occured(prev_div, self.div);
                if self.tac & (1 << 2) != 0 && falling_edge_occured {
                    self.incr_tima()
                }
            }
            0xFF05 => {
                if self.ticks_since_tima_overflown < 5 {
                    self.tima = val;
                    self.tima_overflown = false;
                    self.ticks_since_tima_overflown = 0;
                }
            }
            0xFF06 => self.tma = val,
            0xFF07 => {
                let old = self.tac & (1 << 2) != 0
                    && (self.div >> self.tac_freq_bits[(self.tac & 3) as usize]) & 1 == 1;

                self.tac = val;

                let new = self.tac & (1 << 2) != 0
                    && (self.div >> self.tac_freq_bits[(self.tac & 3) as usize]) & 1 == 1;

                if old && !new {
                    self.incr_tima();
                }
            }

            _ => unreachable!(),
        }
    }
}
