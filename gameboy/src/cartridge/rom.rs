use std::{fs::File, io::Read, path::PathBuf};

use super::{Cartridge, RamIter};

pub struct Rom {
    rom_bank_0: [u8; 0x4000],
    rom_bank_1: [u8; 0x4000],
}

impl Rom {
    pub fn new(mut file: File, rom_bank_0: [u8; 0x4000]) -> Self {
        let mut rom_bank_1: [u8; 0x4000] = [0; 0x4000];
        file.read_exact(&mut rom_bank_1).ok();

        Self {
            rom_bank_0,
            rom_bank_1,
        }
    }
}

impl Cartridge for Rom {
    fn read_rom(&self, addr: u16) -> u8 {
        match addr & 0xF000 {
            0x0000 | 0x1000 | 0x2000 | 0x3000 => self.rom_bank_0[addr as usize],
            0x4000 | 0x5000 | 0x6000 | 0x7000 => self.rom_bank_1[(addr - 0x4000) as usize],

            _ => panic!("Invalid address when reading from ROM cart"),
        }
    }

    fn write_rom(&mut self, _addr: u16, _value: u8) {
        // NOP
    }

    // This cart has no ram?

    fn read_ram(&self, _addr: u16) -> u8 {
        0
    }

    fn write_ram(&mut self, _addr: u16, _value: u8) {}

    fn has_ram(&self) -> bool {
        false
    }

    fn iter_ram(&self) -> RamIter {
        RamIter::empty()
    }

    fn save_file_path(&self) -> Option<&PathBuf> {
        None
    }
}

#[cfg(test)]
pub fn create_test_rom() -> Rom {
    Rom {
        rom_bank_0: [0; 0x4000],
        rom_bank_1: [0; 0x4000],
    }
}
