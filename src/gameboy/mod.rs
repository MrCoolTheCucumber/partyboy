mod bus;
mod cartridge;
pub mod cpu;
mod interrupts;
mod ppu;
mod timer;

use self::{
    bus::Bus,
    cpu::{instructions::InstructionCache, Cpu},
    interrupts::Interrupts,
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
        Interrupts::tick(&mut self.bus.interrupts, &mut self.cpu);
        self.cpu.tick(&mut self.bus, &mut self.instruction_cache);
        self.bus.timer.tick(&mut self.bus.interrupts);
    }
}
