pub mod builder;
mod bus;
mod cartridge;
mod cpu;
mod dma;
pub mod input;
mod interrupts;
mod ppu;
mod timer;

use builder::SerialWriteHandler;
use dma::oam::OamDma;
use ppu::rgb::Rgb;

use self::{
    builder::GameBoyBuilder,
    bus::Bus,
    cpu::{instructions::InstructionCache, Cpu},
    input::Keycode,
    interrupts::Interrupts,
};

pub struct GameBoy {
    instruction_cache: InstructionCache,
    cpu: Cpu,
    bus: Bus,
}

impl GameBoy {
    fn new(rom_path: &str, serial_write_handler: SerialWriteHandler) -> Self {
        let cartridge = cartridge::create(rom_path);

        Self {
            instruction_cache: InstructionCache::new(),
            cpu: Cpu::new(),
            bus: Bus::new(cartridge, serial_write_handler),
        }
    }

    pub fn builder() -> GameBoyBuilder {
        GameBoyBuilder::new()
    }

    pub fn tick(&mut self) {
        if self.cpu.stopped() {
            return;
        }

        Interrupts::tick(&mut self.bus.interrupts, &mut self.cpu);
        self.cpu.tick(&mut self.bus, &mut self.instruction_cache);
        self.bus.ppu.tick(&mut self.bus.interrupts);
        OamDma::dma_tick(&mut self.bus);

        self.bus.timer.tick(&mut self.bus.interrupts);
    }

    pub fn get_frame_buffer(&self) -> &[Rgb] {
        self.bus.ppu.get_frame_buffer()
    }

    pub fn consume_draw_flag(&mut self) -> bool {
        self.bus.ppu.consume_draw_flag()
    }

    pub fn key_down(&mut self, key: Keycode) {
        if self.bus.input.key_down(key) {
            // self.cpu.stopped = false;
            self.bus
                .interrupts
                .request_interupt(interrupts::InterruptFlag::Joypad)
        }
    }

    pub fn key_up(&mut self, key: Keycode) {
        self.bus.input.key_up(key);
    }
}
