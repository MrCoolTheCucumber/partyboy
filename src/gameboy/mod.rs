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
        if self.cpu.stopped() {
            return;
        }

        Interrupts::tick(&mut self.bus.interrupts, &mut self.cpu);
        self.cpu.tick(&mut self.bus, &mut self.instruction_cache);
        self.bus.ppu.tick(&mut self.bus.interrupts);

        self.bus.timer.tick(&mut self.bus.interrupts);
    }

    pub fn get_frame_buffer(&self) -> &[u8] {
        self.bus.ppu.get_frame_buffer()
    }

    pub fn consume_draw_flag(&mut self) -> bool {
        self.bus.ppu.consume_draw_flag()
    }
}
