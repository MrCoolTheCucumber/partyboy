#![cfg(test)]

use super::{InstructionCache, InstructionOpcode};
use crate::{bus::Bus, cpu::Cpu};
use paste::paste;
use seq_macro::seq;

const CYCLE_TABLE: [u64; 0x100] = [
    4, 12, 8, 8, 4, 4, 8, 4, 20, 8, 8, 8, 4, 4, 8, 4, 4, 12, 8, 8, 4, 4, 8, 4, 12, 8, 8, 8, 4, 4,
    8, 4, 12, 12, 8, 8, 4, 4, 8, 4, 8, 8, 8, 8, 4, 4, 8, 4, 12, 12, 8, 8, 12, 12, 12, 4, 8, 8, 8,
    8, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4,
    4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 8, 8, 8, 8, 8, 8, 4, 8, 4, 4, 4,
    4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4,
    4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4,
    4, 4, 4, 8, 4, 20, 12, 16, 16, 24, 16, 8, 16, 8, 16, 12, 4, 12, 24, 8, 16, 20, 12, 16, 0, 24,
    16, 8, 16, 8, 16, 12, 0, 12, 0, 8, 16, 12, 12, 8, 0, 0, 16, 8, 16, 16, 4, 16, 0, 0, 0, 8, 16,
    12, 12, 8, 4, 0, 16, 8, 16, 12, 8, 16, 4, 0, 0, 8, 16,
];

const CB_CYCLE_TABLE: [u64; 0x100] = [
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8,
    16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8,
    8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, 8, 8,
    8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8,
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8,
    16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8,
    8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8,
    8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
];

fn init_default_state() -> (Cpu, Bus, InstructionCache) {
    let mut cpu = Cpu::new();
    cpu.is_fetching = true;
    let cartridge = Box::new(crate::cartridge::rom::create_test_rom());
    let bus = Bus::new(cartridge);
    let instruction_cache = InstructionCache::new();

    (cpu, bus, instruction_cache)
}

macro_rules! test_opcode_timing {
    ($op:expr) => {
        paste! {
            #[test]
            fn [<opcode_timing_ $op>]() {
                let expected_cycles = CYCLE_TABLE[$op];
                if expected_cycles == 0 || $op == 0xCB {
                    return;
                }

                let (mut cpu, mut bus, mut instruction_cache) = init_default_state();
                let opcode = InstructionOpcode::Unprefixed($op);

                let mut cycles: u64 = 0;
                cpu.instruction_opcode = Some(opcode);

                while cpu.instruction_opcode.is_some() {
                    cpu.tick(&mut bus, &mut instruction_cache);
                    cycles += 1;
                }

                assert_eq!(
                    cycles,
                    expected_cycles,
                    "Opcode {:#04X} failed timing test. Expected: {}, Result: {}",
                    $op,
                    expected_cycles,
                    cycles
                );
            }
        }
    };

    (CB $op:expr) => {
        paste! {
            #[test]
            fn [<cb_opcode_timing_ $op>]() {
                let expected_cycles = CB_CYCLE_TABLE[$op];

                let (mut cpu, mut bus, mut instruction_cache) = init_default_state();
                let opcode = InstructionOpcode::Prefixed($op);

                let mut cycles: u64 = 0;
                cpu.instruction_opcode = Some(opcode);

                while cpu.instruction_opcode.is_some() {
                    cpu.tick(&mut bus, &mut instruction_cache);
                    cycles += 1;
                }

                assert_eq!(
                    cycles,
                    expected_cycles,
                    "Opcode {:#06X} failed timing test. Expected: {}, Result: {}",
                    0xCB00 + $op,
                    expected_cycles,
                    cycles
                );
            }
        }
    };
}

macro_rules! define_opcode_timing_tests {
    () => {
        seq!(N in 0..=255 {
            test_opcode_timing!(N);
        });
    };

    (CB) => {
        seq!(N in 0..=255 {
            test_opcode_timing!(CB N);
        });
    };
}

define_opcode_timing_tests!();

define_opcode_timing_tests!(CB);
