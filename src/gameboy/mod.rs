mod bus;
mod cartridge;
pub mod cpu;
mod ppu;

use self::{
    bus::Bus,
    cpu::{instructions::InstructionCache, Cpu},
};

pub struct GameBoy {
    instruction_cache: InstructionCache,
    cpu: Cpu,
    bus: Bus,
}

impl GameBoy {
    pub fn new(rom_path: &str) -> Self {
        let cartridge = cartridge::create(rom_path);

        Self {
            instruction_cache: InstructionCache::new(),
            cpu: Cpu::new(),
            bus: Bus::new(cartridge),
        }
    }

    pub fn tick(&mut self) {
        self.cpu.tick(&mut self.bus, &mut self.instruction_cache);
    }
}
