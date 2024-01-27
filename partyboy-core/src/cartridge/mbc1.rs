#[cfg(feature = "serde")]
use {
    super::serialize::{
        ram_bank_deserialize, ram_bank_serialize, rom_bank_deserialize, rom_bank_serialize,
    },
    serde::{Deserialize, Serialize},
};

use super::{init_rom_and_ram, CartridgeInterface};

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum BankingMode {
    Mode0 = 0,
    Mode1 = 1,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mbc1 {
    is_ram_enabled: bool,

    rom_lo_reg: u8,
    rom_hi_reg: u8,

    current_zero_bank: usize,
    current_rom_bank: usize,
    current_ram_bank: usize,

    mode: BankingMode,

    rom_bank_mask_lo: u8,
    rom_bank_mask_hi: u8,

    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "rom_bank_serialize",
            deserialize_with = "rom_bank_deserialize"
        )
    )]
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

impl Mbc1 {
    pub fn new(
        rom: Vec<u8>,
        ram: Option<Vec<u8>>,
        num_rom_banks: usize,
        num_ram_banks: usize,
    ) -> Self {
        let rom_bank_mask_lo = match num_rom_banks - 1 {
            0..=1 => 0b0000_0001,
            2..=3 => 0b0000_0011,
            4..=7 => 0b0000_0111,
            8..=15 => 0b0000_1111,
            _ => 0b0001_1111,
        };

        let rom_bank_mask_hi = match num_rom_banks - 1 {
            0x00..=0x1F => 0b0000_0000,
            0x20..=0x3F => 0b0010_0000,
            _ => 0b0110_0000,
        };

        let (rom_banks, ram_banks) = init_rom_and_ram(rom, ram, num_rom_banks, num_ram_banks);

        Self {
            is_ram_enabled: false,

            rom_lo_reg: 0,
            rom_hi_reg: 0,

            current_zero_bank: 0,
            current_rom_bank: 1,
            current_ram_bank: 0,

            mode: BankingMode::Mode0,

            rom_bank_mask_lo,
            rom_bank_mask_hi,

            rom_banks,
            ram_banks,
        }
    }

    fn get_mapped_0_bank(&self) -> usize {
        match self.mode {
            BankingMode::Mode0 => 0,
            BankingMode::Mode1 => ((self.rom_hi_reg << 5) & self.rom_bank_mask_hi) as usize,
        }
    }

    fn get_mapped_ram_bank(&self) -> usize {
        match self.mode {
            BankingMode::Mode0 => 0,
            BankingMode::Mode1 => self.current_ram_bank,
        }
    }
}

impl CartridgeInterface for Mbc1 {
    fn read_rom(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom_banks[self.get_mapped_0_bank()][addr as usize],
            0x4000..=0x7FFF => self.rom_banks[self.current_rom_bank][(addr - 0x4000) as usize],

            _ => panic!(
                "Tried to read from cartridge rom with invalid addr: {:#06X}",
                addr
            ),
        }
    }

    fn write_rom(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.is_ram_enabled = (value & 0x0F) == 0x0A,

            0x2000..=0x3FFF => {
                self.rom_lo_reg = value & 0b0001_1111; // store the raw value for mode 1 stuff

                // we only force the bank to 1 if the whole 5 bits are 0?
                // and then mask to the appropriate range after?
                // this behaviour is required to pass some of the mooneye mbc1 test
                let value = if value & 0b0001_1111 == 0 { 1 } else { value };
                let value = (value & self.rom_bank_mask_lo) as usize;

                self.current_rom_bank = (self.current_rom_bank & 0b0110_0000) | value;
            }

            0x4000..=0x5FFF => match self.mode {
                BankingMode::Mode0 => {
                    self.rom_hi_reg = value & 0b0000_0011;

                    let higher_bits = ((value << 5) & self.rom_bank_mask_hi) as usize;
                    self.current_rom_bank = (self.current_rom_bank & 0b0001_1111) | higher_bits;
                }
                BankingMode::Mode1 => {
                    if self.ram_banks.len() == 4 {
                        self.current_ram_bank = (value & 0b0000_0011) as usize;
                        return;
                    }

                    self.rom_hi_reg = value & 0b0000_0011;

                    let higher_bits = ((value << 5) & self.rom_bank_mask_hi) as usize;
                    let selected_rom_bank = (self.current_rom_bank & 0b0001_1111) | higher_bits;

                    match selected_rom_bank {
                        0x00 | 0x20 | 0x40 | 0x60 => {
                            self.current_zero_bank = selected_rom_bank;
                        }
                        _ => self.current_rom_bank = selected_rom_bank,
                    }
                }
            },

            0x6000..=0x7FFF => {
                self.mode = if value & 1 == 0 {
                    BankingMode::Mode0
                } else {
                    BankingMode::Mode1
                };
            }

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

        self.ram_banks[self.get_mapped_ram_bank()][addr as usize]
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        if !self.is_ram_enabled {
            return;
        }

        let mapped_ram_bank = self.get_mapped_ram_bank();
        self.ram_banks[mapped_ram_bank][addr as usize] = value;
    }

    fn has_ram(&self) -> bool {
        !self.ram_banks.is_empty()
    }

    fn ram_banks(&self) -> &Vec<[u8; 0x2000]> {
        &self.ram_banks
    }
}
