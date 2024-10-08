use std::fmt::Display;

use super::cpu::Cpu;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct Interrupts {
    pub master: u8,
    pub enable: u8,
    pub flags: u8,

    pub waiting_for_halt_if: bool,
    pub halt_interrupt_pending: bool,
}

#[derive(Clone, Copy)]
pub enum InterruptFlag {
    VBlank = 0b00000001,
    Stat = 0b00000010,
    Timer = 0b00000100,
    Serial = 0b00001000,
    Joypad = 0b00010000,
}

impl InterruptFlag {
    pub fn vector(&self) -> u16 {
        match self {
            InterruptFlag::VBlank => 0x40,
            InterruptFlag::Stat => 0x48,
            InterruptFlag::Timer => 0x50,
            InterruptFlag::Serial => 0x58,
            InterruptFlag::Joypad => 0x60,
        }
    }
}

impl Display for InterruptFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterruptFlag::VBlank => write!(f, "VBlank"),
            InterruptFlag::Stat => write!(f, "Stat"),
            InterruptFlag::Timer => write!(f, "Timer"),
            InterruptFlag::Serial => write!(f, "Serial"),
            InterruptFlag::Joypad => write!(f, "Joypad"),
        }
    }
}

impl Interrupts {
    pub fn new() -> Self {
        Self {
            master: 0,
            enable: 0,
            flags: 0,

            waiting_for_halt_if: false,
            halt_interrupt_pending: false,
        }
    }

    pub fn enable_master(&mut self) {
        self.master = 1;
    }

    pub fn disable_master(&mut self) {
        self.master = 0;
    }

    pub fn is_master_enabled(&self) -> bool {
        self.master > 0
    }

    pub fn get_interrupt_state(&self) -> Option<InterruptFlag> {
        self.get_interupt_state_latched(self.enable, self.flags)
    }

    pub fn get_interupt_state_latched(
        &self,
        interupt_enable_flags: u8,
        interupt_req_flags: u8,
    ) -> Option<InterruptFlag> {
        if (interupt_enable_flags > 0) && interupt_req_flags > 0 {
            let interupt: u8 = interupt_enable_flags & interupt_req_flags & 0x1F;

            if interupt & InterruptFlag::VBlank as u8 > 0 {
                return Some(InterruptFlag::VBlank);
            }

            if interupt & InterruptFlag::Stat as u8 > 0 {
                return Some(InterruptFlag::Stat);
            }

            if interupt & InterruptFlag::Timer as u8 > 0 {
                return Some(InterruptFlag::Timer);
            }

            if interupt & InterruptFlag::Serial as u8 > 0 {
                return Some(InterruptFlag::Serial);
            }

            if interupt & InterruptFlag::Joypad as u8 > 0 {
                return Some(InterruptFlag::Joypad);
            }
        }

        None
    }

    pub fn clear_interupt(&mut self, flag: InterruptFlag) {
        self.flags &= !(flag as u8);
    }

    pub fn request_interupt(&mut self, flag: InterruptFlag) {
        self.flags |= flag as u8;

        if self.waiting_for_halt_if {
            self.halt_interrupt_pending = true;
        }
    }

    pub fn tick(interrupt: &mut Interrupts, cpu: &mut Cpu) {
        if interrupt.is_master_enabled()
            && (!cpu.is_processing_instruction() || cpu.is_fetching())
            && interrupt.get_interrupt_state().is_some()
        {
            cpu.initiate_interrupt_service_routin();
        }
    }
}
