#[cfg(feature = "serde")]
use {
    super::serialize::{ram_bank_deserialize, ram_bank_serialize},
    serde::{Deserialize, Serialize},
};

use super::{init_rom_and_ram, CartridgeInterface};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mbc2 {
    is_ram_enabled: bool,

    current_rom_bank: usize,
    rom_bank_mask: u8,

    #[serde(skip)]
    rom_banks: Vec<[u8; 0x4000]>,

    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "ram_bank_serialize",
            deserialize_with = "ram_bank_deserialize"
        )
    )]
    ram_banks: Vec<[u8; 0x2000]>,
}

impl Mbc2 {
    pub fn new(
        rom: Vec<u8>,
        ram: Option<Vec<u8>>,
        num_rom_banks: usize,
        num_ram_banks: usize,
    ) -> Self {
        let rom_bank_mask = match num_rom_banks - 1 {
            0..=1 => 0b0000_0001,
            2..=3 => 0b0000_0011,
            4..=7 => 0b0000_0111,
            _ => 0b0000_1111,
        };

        let (rom_banks, mut ram_banks) = init_rom_and_ram(rom, ram, num_rom_banks, num_ram_banks);

        // MBC2 always has at least 1 bank:
        // https://gbdev.io/pandocs/The_Cartridge_Header.html#0149--ram-size
        if num_ram_banks == 0 {
            ram_banks.push([0; 0x2000]);
        }

        Self {
            is_ram_enabled: false,
            current_rom_bank: 1,
            rom_banks,
            ram_banks,
            rom_bank_mask,
        }
    }
}

impl CartridgeInterface for Mbc2 {
    fn read_rom(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom_banks[0][addr as usize],
            0x4000..=0x7FFF => self.rom_banks[self.current_rom_bank][(addr - 0x4000) as usize],

            _ => panic!(
                "Tried to read from cartridge rom with invalid addr: {:#06X}",
                addr
            ),
        }
    }

    fn write_rom(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x3FFF => {
                let is_bit_8_set = addr & 0b0000_0001_0000_0000 != 0;
                if is_bit_8_set {
                    // select rom bank
                    let mut value = value & 0b0000_1111;
                    if value == 0 {
                        value = 1;
                    }

                    self.current_rom_bank = (value & self.rom_bank_mask) as usize;
                } else {
                    // enable/disable ram
                    self.is_ram_enabled = (value & 0x0F) == 0x0A
                }
            }
            0x4000..=0x7FFF => {}

            _ => panic!(
                "Tried to write to cartridge rom with invalid addr: {:#06X}, val: {:#04X}",
                addr, value
            ),
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        if !self.is_ram_enabled {
            return 0xFF;
        }

        let addr = addr & 0b0000_0001_1111_1111;
        self.ram_banks[0][addr as usize] | 0b1111_0000
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        if !self.is_ram_enabled {
            return;
        }

        let addr = addr & 0b0000_0001_1111_1111;
        self.ram_banks[0][addr as usize] = value;
    }

    fn has_ram(&self) -> bool {
        !self.ram_banks.is_empty()
    }

    fn ram_banks(&self) -> &Vec<[u8; 0x2000]> {
        &self.ram_banks
    }

    fn load_rom(&mut self, rom: Vec<[u8; 0x4000]>) {
        self.rom_banks = rom;
    }

    fn take_rom(self) -> Vec<[u8; 0x4000]> {
        self.rom_banks
    }
}
