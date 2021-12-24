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
use dma::{hdma::Hdma, oam::OamDma};
use ppu::rgb::Rgb;

use crate::dma::hdma::DmaType;

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

        fn tick_cpu_related(gb: &mut GameBoy) {
            Interrupts::tick(&mut gb.bus.interrupts, &mut gb.cpu);
            gb.cpu.tick(&mut gb.bus, &mut gb.instruction_cache);
        }

        // If HDMA/GDMA is currently copying data, then cpu execution is paused
        if let Some(state) = self.bus.ppu.hdma.current_dma {
            match state {
                DmaType::Hdma => {
                    if !self.bus.ppu.hdma.hdma_currently_copying {
                        tick_cpu_related(self);
                    }
                }
                DmaType::Gdma => Hdma::tick_gdma(&mut self.bus),
            }
        } else {
            tick_cpu_related(self)
        }

        self.bus.tick_ppu();
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
