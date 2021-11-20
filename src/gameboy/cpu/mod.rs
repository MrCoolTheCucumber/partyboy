pub mod instructions;
pub mod register;

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
        }
    }

    fn fetch(&mut self, bus: &mut Bus) -> u8 {
        let op = bus.read_u8(self.pc);
        self.pc += 1;
        op
    }

    pub fn tick(&mut self, bus: &mut Bus, instruction_cache: &mut InstructionCache) {
        if self.instruction_opcode.is_none() {
            self.is_fetching = true;

            if bus.bios_enabled && self.pc >= 0x100 {
                bus.bios_enabled = false;
            }

            self.instruction_opcode = match self.fetch(bus) {
                0xCB => todo!(),
                opcode => Some(InstructionOpcode::Unprefixed(opcode)),
            };

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
