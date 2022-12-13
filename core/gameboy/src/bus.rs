#![allow(clippy::match_overlapping_arm)]

use std::fmt::Display;

use super::{cartridge::Cartridge, input::Input, interrupts::Interrupts, ppu::Ppu, timer::Timer};
use crate::{
    apu::Apu, builder::SerialWriteHandler, common::D2Array,
    cpu::speed_controller::CpuSpeedController, dma::oam::OamDma,
};

#[cfg(feature = "serde")]
use {
    serde::{Deserialize, Serialize},
    serde_big_array::BigArray,
};

#[derive(Clone, Copy, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CgbCompatibility {
    None,
    #[default]
    CgbOnly,
    CgbAndDmg,
}

impl CgbCompatibility {
    pub fn is_cgb_mode(&self) -> bool {
        matches!(self, &CgbCompatibility::CgbOnly)
    }
}

impl Display for CgbCompatibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CgbCompatibility::None => write!(f, "Dmg"),
            CgbCompatibility::CgbOnly => write!(f, "Cgb only"),
            CgbCompatibility::CgbAndDmg => write!(f, "CgbCompat"),
        }
    }
}

impl From<u8> for CgbCompatibility {
    fn from(val: u8) -> Self {
        match val {
            0x80 => CgbCompatibility::CgbAndDmg,
            0xC0 => CgbCompatibility::CgbOnly,
            _ => CgbCompatibility::None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct Bus {
    #[cfg_attr(
        feature = "serde",
        serde(skip, default = "Bus::get_handle_blargg_output")
    )]
    serial_write_handler: SerialWriteHandler,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub cartridge: Option<Box<dyn Cartridge>>,
    pub ppu: Ppu,

    pub working_ram: D2Array<u8, 0x1000, 8>,
    working_ram_bank: usize,

    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    pub io: [u8; 0x100],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    pub zero_page: [u8; 0x80],

    pub oam_dma: OamDma,

    pub bios_enabled: bool,
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    pub bios: [u8; 0x900],
    pub console_compatibility_mode: CgbCompatibility,

    pub interrupts: Interrupts,
    pub timer: Timer,
    pub input: Input,
    pub cpu_speed_controller: CpuSpeedController,
    pub apu: Apu,
}

impl Bus {
    pub fn new(
        cartridge: Option<Box<dyn Cartridge>>,
        serial_write_handler: SerialWriteHandler,
        bios: [u8; 2304],
    ) -> Self {
        Self {
            serial_write_handler,

            cartridge,
            ppu: Ppu::new(),

            working_ram: [[0; 0x1000]; 8].into(),
            working_ram_bank: 1,

            io: [0; 0x100],
            zero_page: [0; 0x80],

            oam_dma: OamDma::default(),

            bios_enabled: true,
            bios,
            console_compatibility_mode: CgbCompatibility::CgbOnly,

            interrupts: Interrupts::new(),
            timer: Timer::new(),
            input: Input::new(),
            cpu_speed_controller: CpuSpeedController::new(CgbCompatibility::CgbOnly),
            apu: Apu::new(),
        }
    }

    pub fn set_serial_write_handler(&mut self, handler: SerialWriteHandler) {
        self.serial_write_handler = handler;
    }

    pub(crate) fn get_handle_blargg_output() -> SerialWriteHandler {
        let mut blargg_output_buffer: Vec<char> = Vec::new();

        let handler: SerialWriteHandler = Box::new(move |val| {
            let c = val as char;
            if c == "\n".chars().next().unwrap() {
                let string = String::from_iter(blargg_output_buffer.iter());
                log::info!("{}", string);
                blargg_output_buffer.clear();
            } else {
                blargg_output_buffer.push(c);
            }
        });

        handler
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x00FF if self.bios_enabled => self.bios[addr as usize],
            0x0200..=0x08FF if self.bios_enabled => self.bios[addr as usize],

            0x0000..=0x7FFF => self
                .cartridge
                .as_ref()
                .map(|cart| cart.read_rom(addr))
                .unwrap_or(0xFF),
            0x8000..=0x9FFF => self.ppu.read_vram(addr - 0x8000),
            0xA000..=0xBFFF => self
                .cartridge
                .as_ref()
                .map(|cart| cart.read_ram(addr - 0xA000))
                .unwrap_or(0xFF),

            0xC000..=0xCFFF => self.working_ram[0][(addr - 0xC000) as usize],
            0xD000..=0xDFFF => self.working_ram[self.working_ram_bank][(addr - 0xD000) as usize],

            0xE000..=0xEFFF => self.working_ram[0][(addr - 0xE000) as usize],
            0xF000..=0xFDFF => self.working_ram[self.working_ram_bank][(addr - 0xF000) as usize],

            0xFE00..=0xFEFF => {
                if addr < 0xFEA0 {
                    if self.oam_dma.is_active() {
                        return 0xFF;
                    }

                    return self.ppu.sprite_table[(addr - 0xFE00) as usize];
                }

                0xFF
            }

            // 0xFF00 and above
            0xFF00 => self.input.read_joyp(),
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF0F => 0b1110_0000 | (self.interrupts.flags & 0b0001_1111),
            0xFFFF => self.interrupts.enable,

            0xFF46 => self.oam_dma.read_u8(),
            0xFF51..=0xFF55 => self.ppu.hdma.read_u8(addr),

            0xFF40..=0xFF4B => self.ppu.read_u8(addr),
            0xFF4D => self.cpu_speed_controller.read_key1(),
            0xFF4F => self.ppu.read_u8(addr),
            0xFF68..=0xFF6B => self.ppu.read_u8(addr),

            0xFF70 => self.working_ram_bank as u8,

            0xFF10..=0xFF14 => self.apu.read_u8(addr),
            0xFF16..=0xFF19 => self.apu.read_u8(addr),
            0xFF24..=0xFF26 => self.apu.read_u8(addr),

            0xFF00..=0xFF7F => self.io[(addr - 0xFF00) as usize],
            0xFF80..=0xFFFE => self.zero_page[(addr - 0xFF80) as usize],
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x7FFF => {
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.write_rom(addr, val)
                }
            }
            0x8000..=0x9FFF => self.ppu.write_vram(addr - 0x8000, val),
            0xA000..=0xBFFF => {
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.write_ram(addr - 0xA000, val)
                }
            }

            0xC000..=0xCFFF => self.working_ram[0][(addr - 0xC000) as usize] = val,
            0xD000..=0xDFFF => {
                self.working_ram[self.working_ram_bank][(addr - 0xD000) as usize] = val
            }

            0xE000..=0xEFFF => self.working_ram[0][(addr - 0xE000) as usize] = val,
            0xF000..=0xFDFF => {
                self.working_ram[self.working_ram_bank][(addr - 0xF000) as usize] = val
            }

            0xFE00..=0xFEFF => {
                if self.oam_dma.is_active() {
                    return;
                }

                // TODO: redundant if? just add more match branches
                if addr < 0xFEA0 {
                    self.ppu.sprite_table[(addr - 0xFE00) as usize] = val;
                }
            }

            0xFF10..=0xFF14 => self.apu.write_u8(addr, val),
            0xFF16..=0xFF19 => self.apu.write_u8(addr, val),
            0xFF24..=0xFF26 => self.apu.write_u8(addr, val),

            // 0xFF00 and above
            0xFF00 => self.input.set_column_line(val),
            0xFF01 => (self.serial_write_handler)(val),
            0xFF03..=0xFF07 => self.timer.write(addr, val),
            0xFF0F => self.interrupts.flags = val,
            0xFF50 => {
                log::info!("Disabling BIOS");
                self.bios_enabled = false; // accept any val for now
            }
            0xFFFF => self.interrupts.enable = val,

            0xFF46 => self.oam_dma.write_u8(val),
            0xFF51..=0xFF55 => self.ppu.hdma.write_u8(addr, val),

            0xFF40..=0xFF4B => self.ppu.write_u8(addr, val, &mut self.interrupts),
            0xFF4C => {
                let val = CgbCompatibility::from(val);
                let val = match val {
                    CgbCompatibility::None => val,
                    _ => CgbCompatibility::CgbOnly,
                };

                self.console_compatibility_mode = val;
                self.ppu
                    .set_console_compatibility(self.console_compatibility_mode);
                self.cpu_speed_controller
                    .set_console_compatibility(self.console_compatibility_mode);
                log::info!(
                    "Setting compatibility mode: {}",
                    self.console_compatibility_mode
                );
            }
            0xFF4D => {
                // Key1 (speed switching)
                let prepare = (val & 0b0000_0001) == 1;
                self.cpu_speed_controller.set_prepare_speed_switch(prepare);
            }
            0xFF4F => self.ppu.write_u8(addr, val, &mut self.interrupts),
            0xFF68..=0xFF6C => self.ppu.write_u8(addr, val, &mut self.interrupts),

            0xFF70 => {
                let mut bank = (val & 0b0000_0111) as usize;
                if bank == 0 {
                    bank = 1;
                }
                self.working_ram_bank = bank;
            }

            0xFF00..=0xFF7F => self.io[(addr - 0xFF00) as usize] = val,
            0xFF80..=0xFFFE => self.zero_page[(addr - 0xFF80) as usize] = val,
        }
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        self.read_u8(addr) as u16 + ((self.read_u8(addr + 1) as u16) << 8)
    }

    pub fn write_u16(&mut self, addr: u16, val: u16) {
        let lower_val: u8 = (val & 0x00FF) as u8;
        let higher_val: u8 = ((val & 0xFF00) >> 8) as u8;

        self.write_u8(addr, lower_val);
        self.write_u8(addr + 1, higher_val);
    }

    pub fn tick_ppu(&mut self) {
        self.ppu.tick(&mut self.interrupts);
    }

    pub fn hdma_copy_word(&mut self) -> bool {
        self.ppu.hdma.tick_hdma(
            // unwrap: if we have no cart by the time we need to run hdma
            // then something is horribly wrong
            self.cartridge.as_ref().unwrap().as_ref(),
            &self.working_ram,
            self.working_ram_bank,
            &mut self.ppu.gpu_vram,
            (self.ppu.gpu_vram_bank & 1) as usize,
        )
    }
}
