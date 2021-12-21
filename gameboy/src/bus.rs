#![allow(clippy::match_overlapping_arm)]

use std::fmt::Display;

use super::{cartridge::Cartridge, input::Input, interrupts::Interrupts, ppu::Ppu, timer::Timer};
use crate::{builder::SerialWriteHandler, dma::Dma};

include!(concat!(env!("OUT_DIR"), "/boot_rom.rs"));

#[derive(Clone, Copy)]
pub enum CgbCompatibility {
    None,
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

pub(crate) struct Bus {
    serial_write_handler: SerialWriteHandler,

    pub cartridge: Box<dyn Cartridge>,
    pub ppu: Ppu,

    pub working_ram: [[u8; 0x1000]; 8],
    working_ram_bank: usize,

    pub io: [u8; 0x100],
    pub zero_page: [u8; 0x80],
    pub dma: Dma,

    pub bios_enabled: bool,
    pub bios: [u8; 0x900],
    pub console_compatibility_mode: CgbCompatibility,

    pub interrupts: Interrupts,
    pub timer: Timer,
    pub input: Input,
}

impl Bus {
    pub fn new(cartridge: Box<dyn Cartridge>, serial_write_handler: SerialWriteHandler) -> Self {
        Self {
            serial_write_handler,

            cartridge,
            ppu: Ppu::new(),

            working_ram: [[0; 0x1000]; 8],
            working_ram_bank: 1,

            io: [0; 0x100],
            zero_page: [0; 0x80],
            dma: Dma::default(),

            bios_enabled: true,
            bios: BOOT_ROM,
            console_compatibility_mode: CgbCompatibility::CgbOnly,

            interrupts: Interrupts::new(),
            timer: Timer::new(),
            input: Input::new(),
        }
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
            0x0000..=0x00FF if self.bios_enabled => return self.bios[addr as usize],
            0x0200..=0x08FF if self.bios_enabled => return self.bios[addr as usize],

            0x0000..=0x7FFF => self.cartridge.read_rom(addr),
            0x8000..=0x9FFF => self.ppu.read_vram(addr - 0x8000),
            0xA000..=0xBFFF => self.cartridge.read_ram(addr - 0xA000),

            0xC000..=0xCFFF => self.working_ram[0][(addr - 0xC000) as usize],
            0xD000..=0xDFFF => self.working_ram[self.working_ram_bank][(addr - 0xD000) as usize],

            0xE000..=0xEFFF => self.working_ram[0][(addr - 0xE000) as usize],
            0xF000..=0xFDFF => self.working_ram[self.working_ram_bank][(addr - 0xF000) as usize],

            0xFE00..=0xFEFF => self.ppu.sprite_table[(addr - 0xFE00) as usize],

            // 0xFF00 and above
            0xFF00 => self.input.read_joyp(),
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF0F => 0b1110_0000 | (self.interrupts.flags & 0b0001_1111),
            0xFFFF => self.interrupts.enable,

            0xFF51..=0xFF55 => self.dma.read_u8(addr),
            0xFF40..=0xFF4B => self.ppu.read_u8(addr),
            0xFF4F => self.ppu.read_u8(addr),
            0xFF68..=0xFF6B => self.ppu.read_u8(addr),

            0xFF70 => self.working_ram_bank as u8,

            0xFF00..=0xFF7F => self.io[(addr - 0xFF00) as usize],
            0xFF80..=0xFFFE => self.zero_page[(addr - 0xFF80) as usize],
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x7FFF => {
                self.cartridge.write_rom(addr, val);
            }
            0x8000..=0x9FFF => self.ppu.write_vram(addr - 0x8000, val),
            0xA000..=0xBFFF => self.cartridge.write_ram(addr - 0xA000, val),

            0xC000..=0xCFFF => self.working_ram[0][(addr - 0xC000) as usize] = val,
            0xD000..=0xDFFF => {
                self.working_ram[self.working_ram_bank][(addr - 0xD000) as usize] = val
            }

            0xE000..=0xEFFF => self.working_ram[0][(addr - 0xE000) as usize] = val,
            0xF000..=0xFDFF => {
                self.working_ram[self.working_ram_bank][(addr - 0xF000) as usize] = val
            }

            0xFE00..=0xFEFF => {
                // TODO: redundant if? just add more match branches
                if addr < 0xFEA0 {
                    self.ppu.sprite_table[(addr - 0xFE00) as usize] = val;
                }
            }

            // 0xFF00 and above
            0xFF4C => {
                self.console_compatibility_mode = CgbCompatibility::from(val);
                self.ppu
                    .set_console_compatibility(self.console_compatibility_mode);
                log::info!(
                    "Setting compatibility mode: {}",
                    self.console_compatibility_mode
                );
            }

            0xFF00 => self.input.set_column_line(val),
            0xFF01 => (self.serial_write_handler)(val),
            0xFF03..=0xFF07 => self.timer.write(addr, val),
            0xFF0F => self.interrupts.flags = val,
            0xFF50 => {
                log::info!("Disabling BIOS");
                self.bios_enabled = false; // accept any val for now
            }
            0xFFFF => self.interrupts.enable = val,

            0xFF46 => {
                let source_addr: u16 = (val as u16) << 8;

                for i in 0..160 {
                    let src_val = self.read_u8(source_addr + i);
                    self.write_u8(0xFE00 + i, src_val);
                }

                // write the inderlying value
                self.ppu.write_u8(addr, val);
            }

            0xFF51..=0xFF54 => self.dma.write_u8(addr, val),
            0xFF55 => {
                let bytes_to_transfer: u16 = ((val & 0b0111_1111) + 1) as u16 * 0x10;

                // TOOD: for now, just ignore mode and instantly copy?
                let _is_gdma = (val & 0b1000_0000) == 0;

                let mut src_addr: u16 =
                    ((self.dma.src_hi as u16) << 8) | ((self.dma.src_lo) as u16);
                let mut dest_addr: u16 =
                    (((self.dma.dest_hi) as u16) << 8) | ((self.dma.dest_lo) as u16);

                // Apply "masks"
                src_addr &= 0b0111_1111_1111_0000;
                dest_addr &= 0b0001_1111_1111_0000;
                dest_addr |= 0b1000_0000_0000_0000;

                for i in 0..bytes_to_transfer {
                    let transfer_val = self.read_u8(src_addr + i);
                    self.write_u8(dest_addr + i, transfer_val)
                }
            }

            0xFF40..=0xFF4B => self.ppu.write_u8(addr, val),
            0xFF4F => self.ppu.write_u8(addr, val),
            0xFF68..=0xFF6C => self.ppu.write_u8(addr, val),

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
}
