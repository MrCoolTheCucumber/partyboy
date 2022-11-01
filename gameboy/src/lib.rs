pub mod builder;
mod bus;
mod cartridge;
mod cpu;
mod dma;
pub mod input;
mod interrupts;
pub mod ppu;
mod timer;

#[cfg(feature = "debug_info")]
pub mod debug;

use builder::SerialWriteHandler;
use dma::{
    hdma::{Hdma, HdmaController},
    oam::OamDma,
};

#[cfg(feature = "web")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(not(feature = "web"))]
use self::builder::GameBoyBuilder;

use self::{
    bus::Bus,
    cpu::{instructions::InstructionCache, Cpu},
    dma::hdma::DmaType,
    input::Keycode,
    interrupts::Interrupts,
    ppu::rgb::Rgb,
};

#[cfg_attr(feature = "web", wasm_bindgen)]
pub struct GameBoy {
    instruction_cache: InstructionCache,
    cpu: Cpu,
    bus: Bus,
    hdma_controller: HdmaController,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl GameBoy {
    #[cfg(not(feature = "web"))]
    fn new(rom: Vec<u8>, ram: Option<Vec<u8>>, serial_write_handler: SerialWriteHandler) -> Self {
        let cartridge = cartridge::create(rom, ram);

        Self {
            instruction_cache: InstructionCache::new(),
            cpu: Cpu::new(),
            bus: Bus::new(cartridge, serial_write_handler),
            hdma_controller: HdmaController::default(),
        }
    }

    #[cfg(not(feature = "web"))]
    pub fn builder() -> GameBoyBuilder {
        GameBoyBuilder::new()
    }

    #[cfg(feature = "web")]
    pub fn new(rom: Vec<u8>, ram: Option<Vec<u8>>) -> Self {
        console_error_panic_hook::set_once();
        let cartridge = cartridge::create(rom, ram);

        Self {
            instruction_cache: InstructionCache::new(),
            cpu: Cpu::new(),
            bus: Bus::new(cartridge, Box::new(Bus::get_handle_blargg_output())),
            hdma_controller: HdmaController::default(),
        }
    }

    fn tick_cpu_related(&mut self) {
        Interrupts::tick(&mut self.bus.interrupts, &mut self.cpu);
        self.cpu.tick(&mut self.bus, &mut self.instruction_cache);

        if self.bus.cpu_speed_controller.is_double_speed() {
            self.cpu.tick(&mut self.bus, &mut self.instruction_cache);
        }
    }

    pub fn tick(&mut self) {
        if self.cpu.stopped() {
            return;
        }

        // check the controller state first before handling!
        if !self.hdma_controller.currently_copying(&self.bus) {
            self.tick_cpu_related();
        }

        self.hdma_controller
            .handle_hdma(&mut self.bus, &mut self.cpu);

        // TODO: move into handle_hdma
        if matches!(self.bus.ppu.hdma.current_dma, Some(DmaType::Gdma)) {
            Hdma::tick_gdma(&mut self.bus);
        }

        self.bus.tick_ppu();

        OamDma::dma_tick(&mut self.bus);
        self.bus.timer.tick(&mut self.bus.interrupts);

        if self.bus.cpu_speed_controller.is_double_speed() {
            OamDma::dma_tick(&mut self.bus);
            self.bus.timer.tick(&mut self.bus.interrupts);
        }
    }

    #[cfg(not(feature = "web"))]
    pub fn get_frame_buffer(&self) -> &[Rgb] {
        self.bus.ppu.get_frame_buffer()
    }

    #[cfg(feature = "web")]
    pub fn get_frame_buffer(&self) -> Vec<u8> {
        self.bus
            .ppu
            .get_frame_buffer()
            .to_vec()
            .iter()
            .flat_map(|px| [px.r, px.g, px.b])
            .collect()
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

    #[cfg(feature = "debug_info")]
    pub(crate) fn bus(&self) -> &Bus {
        &self.bus
    }

    #[cfg(feature = "web")]
    pub fn tick_to_frame(&mut self) {
        while !self.consume_draw_flag() {
            self.tick();
        }
    }
}
