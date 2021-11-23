pub mod instructions;
pub mod register;

use std::fmt::Debug;

use crate::gameboy::cpu::instructions::InstructionStep;

use self::{
    instructions::{InstructionCache, InstructionOpcode, InstructionState},
    register::Register,
};
use super::bus::Bus;

pub struct Cpu {
    af: Register,
    bc: Register,
    de: Register,
    hl: Register,

    pc: u16,
    sp: u16,

    // unnoficial temp values used to help store state
    // between "instruction steps"
    operand8: u8,
    operand16: u16,
    temp8: u8,
    temp16: u16,

    cycle: u64,

    is_fetching: bool,
    instruction_opcode: Option<InstructionOpcode>,

    stopped: bool,
    halted: bool,
    halted_waiting_for_interrupt_pending: bool,
    halt_bug_triggered: bool,
    ei_delay: bool,
    ei_delay_cycles: u8,
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cpu")
            .field("af", &self.af)
            .field("bc", &self.bc)
            .field("de", &self.de)
            .field("hl", &self.hl)
            .field("pc", &self.pc)
            .field("sp", &self.sp)
            .field("operand8", &self.operand8)
            .field("operand16", &self.operand16)
            .field("temp8", &self.temp8)
            .field("temp16", &self.temp16)
            .field("cycle", &self.cycle)
            .field("is_fetching", &self.is_fetching)
            .field("instruction_opcode", &self.instruction_opcode)
            .finish()
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            af: Register::new(0x01, 0xB0),
            bc: Register::new(0x00, 0x13),
            de: Register::new(0x00, 0xD8),
            hl: Register::new(0x01, 0x4D),

            pc: 0x100,
            sp: 0xFFFE,

            operand8: 0x0,
            operand16: 0x0,
            temp8: 0x0,
            temp16: 0x0,

            cycle: 0,

            is_fetching: false,
            instruction_opcode: None,

            stopped: false,
            halted: false,
            halted_waiting_for_interrupt_pending: false,
            halt_bug_triggered: false,
            ei_delay: false,
            ei_delay_cycles: 0,
        }
    }

    pub fn initiate_interrupt_service_routin(&mut self) {
        // TODO: this should only run in debug mode
        // make it a debug_assert?
        if self.instruction_opcode.is_some() {
            panic!("ISR was fired whilst running an instruction");
        }

        self.instruction_opcode = Some(InstructionOpcode::InterruptServiceRoutine);
        self.is_fetching = false;
        self.halted = false; // TODO: un-halting should take extra 4t (TCAGBD 4.9?)
    }

    pub fn stopped(&self) -> bool {
        self.stopped
    }

    pub fn is_fetching(&self) -> bool {
        self.is_fetching
    }

    pub fn is_processing_instruction(&self) -> bool {
        self.instruction_opcode.is_some()
    }

    fn fetch(&mut self, bus: &mut Bus) -> u8 {
        let op = bus.read_u8(self.pc);
        self.pc += 1;
        op
    }

    pub fn tick(&mut self, bus: &mut Bus, instruction_cache: &mut InstructionCache) {
        if self.ei_delay {
            self.ei_delay_cycles -= 1;

            if self.ei_delay_cycles == 0 {
                bus.interrupts.enable_master();
                self.ei_delay = false;
            }
        }

        if self.halted {
            if self.halted_waiting_for_interrupt_pending {
                if !bus.interrupts.halt_interrupt_pending {
                    return;
                }

                bus.interrupts.halt_interrupt_pending = false;
                bus.interrupts.waiting_for_halt_if = false;
                self.halted_waiting_for_interrupt_pending = false;
                self.halted = false;
            } else {
                return;
            }
        }

        if self.instruction_opcode.is_none() {
            self.is_fetching = true;

            if bus.bios_enabled && self.pc >= 0x100 {
                bus.bios_enabled = false;
            }

            self.instruction_opcode = match self.fetch(bus) {
                0xCB => Some(InstructionOpcode::Prefixed(self.fetch(bus))),
                opcode => Some(InstructionOpcode::Unprefixed(opcode)),
            };

            match self.instruction_opcode.unwrap() {
                InstructionOpcode::InterruptServiceRoutine => log::debug!("Fetched the isr??"),
                InstructionOpcode::Unprefixed(opcode) => log::debug!("Fetched {:#04X}", opcode),
                InstructionOpcode::Prefixed(opcode) => {
                    log::debug!("Fetched {:#06X}", 0xCB00 + (opcode as u16))
                }
            }

            self.cycle += 1;
            return;
        }

        self.cycle += 1;
        if self.cycle < 4 {
            return;
        }

        self.cycle = 0;

        let opcode = self.instruction_opcode.unwrap();
        match instruction_cache.get(opcode) {
            InstructionStep::Standard(_) => {
                if self.is_fetching {
                    self.is_fetching = false;
                    return;
                }

                self.exec(opcode, bus, instruction_cache);
            }
            InstructionStep::Instant(_) => {
                self.is_fetching = false;
                self.exec(opcode, bus, instruction_cache);
            }
        }
    }

    fn exec(
        &mut self,
        opcode: InstructionOpcode,
        bus: &mut Bus,
        instruction_cache: &mut InstructionCache,
    ) {
        match instruction_cache.exec(opcode, self, bus) {
            InstructionState::InProgress => {}
            InstructionState::ExecNextInstantly => {
                self.exec(opcode, bus, instruction_cache);
            }
            InstructionState::Branch(continue_exec) => {
                if !continue_exec {
                    instruction_cache.reset(opcode);
                    self.instruction_opcode = None;
                }
            }
            InstructionState::Finished => {
                instruction_cache.reset(opcode);
                self.instruction_opcode = None
            }
        }
    }
}
