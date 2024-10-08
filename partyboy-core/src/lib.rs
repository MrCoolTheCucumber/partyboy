mod apu;
pub mod builder;
mod bus;
mod cartridge;
mod common;
mod cpu;
#[cfg(feature = "debug_info")]
pub mod debug;
mod dma;
pub mod input;
mod interrupts;
pub mod ppu;
mod timer;

use apu::Sample;
use cartridge::Cartridge;
#[cfg(not(feature = "web"))]
use ppu::rgb::Rgb;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

use self::builder::GameBoyBuilder;
use self::{
    builder::SerialWriteHandler,
    bus::Bus,
    cpu::{Cpu, InstructionCache},
    dma::{
        hdma::{DmaType, Hdma, HdmaController},
        oam::OamDma,
    },
    input::Keycode,
    interrupts::Interrupts,
};

/// Number of cycles per second the gameboy does in single speed mode.
/// When the emulator is in double speed mode, you don't need to double the speed
/// as the `tick` function will internally tick twice
pub const SPEED: u64 = 4_194_304;

#[cfg_attr(feature = "web", wasm_bindgen)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GameBoy {
    #[cfg_attr(feature = "serde", serde(skip))]
    instruction_cache: InstructionCache,
    cpu: Cpu,
    bus: Bus,
    hdma_controller: HdmaController,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl GameBoy {
    fn new(
        rom: Option<Vec<u8>>,
        ram: Option<Vec<u8>>,
        bios: [u8; 0x900],
        serial_write_handler: SerialWriteHandler,
    ) -> Self {
        let cartridge = rom.map(|rom| Cartridge::new(rom, ram));

        Self {
            instruction_cache: InstructionCache::new(),
            cpu: Cpu::new(),
            bus: Bus::new(cartridge, serial_write_handler, bios),
            hdma_controller: HdmaController::default(),
        }
    }

    pub fn builder() -> GameBoyBuilder {
        GameBoyBuilder::new()
    }

    fn tick_cpu_related(&mut self) {
        Interrupts::tick(&mut self.bus.interrupts, &mut self.cpu);
        self.cpu.tick(&mut self.bus, &mut self.instruction_cache);

        if self.bus.cpu_speed_controller.is_double_speed() {
            self.cpu.tick(&mut self.bus, &mut self.instruction_cache);
        }
    }

    #[cfg(not(feature = "web"))]
    pub fn tick(&mut self) -> Option<(Sample, Sample)> {
        if self.cpu.stopped() {
            return self.bus.apu.tick_sample_only();
        }

        #[cfg(feature = "gen_bios_snapshot")]
        if self.cpu.pc == 0x100 && !self.bus.bios_enabled {
            return None;
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

        self.bus.apu.tick(
            self.bus.timer.div(),
            self.bus.cpu_speed_controller.speed_mode(),
        )
    }

    // FIXME: Avoid having two different tick functions if possible
    #[cfg(feature = "web")]
    pub fn tick(&mut self) -> Option<Box<[f32]>> {
        if self.cpu.stopped() {
            return self
                .bus
                .apu
                .tick_sample_only()
                .map(|(l, r): (Sample, Sample)| vec![l, r].into_boxed_slice());
        }

        #[cfg(feature = "gen_bios_snapshot")]
        if self.cpu.pc == 0x100 && !self.bus.bios_enabled {
            return None;
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

        self.bus
            .apu
            .tick(
                self.bus.timer.div(),
                self.bus.cpu_speed_controller.speed_mode(),
            )
            .map(|(l, r): (Sample, Sample)| vec![l, r].into_boxed_slice())
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

    pub fn release_all_keys(&mut self) {
        self.bus.input.release_all_keys();
    }

    #[cfg(feature = "debug_info")]
    pub(crate) fn bus(&self) -> &Bus {
        &self.bus
    }

    pub fn load_snapshot(&mut self, mut snapshot: GameBoy) {
        if let (Some(old_cart), Some(new_cart)) =
            (self.bus.cartridge.take(), snapshot.bus.cartridge.as_mut())
        {
            new_cart.load_rom(old_cart.take_rom());
        }

        *self = snapshot;
    }

    pub fn try_read_cartridge_ram(&self) -> Option<Box<[u8]>> {
        self.bus
            .cartridge
            .as_ref()
            .map(|cart| cart.iter_ram().collect::<Vec<_>>().into_boxed_slice())
    }
}
