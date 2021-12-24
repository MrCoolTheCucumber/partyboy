use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use super::{get_save_file_path_from_rom_path, try_read_save_file, Cartridge};

pub struct Mbc2 {
    is_ram_enabled: bool,

    current_rom_bank: usize,
    rom_bank_mask: u8,

    rom_banks: Vec<[u8; 0x4000]>,
    ram_banks: Vec<[u8; 0x2000]>,

    save_file_path: PathBuf,
}

impl Mbc2 {
    pub fn new(mut file: File, path: &Path, rom_bank_0: [u8; 0x4000], num_rom_banks: u16) -> Self {
        let mut rom_banks = vec![rom_bank_0];

        for _ in 0..num_rom_banks - 1 {
            let mut bank = [0; 0x4000];
            file.read_exact(&mut bank).ok();
            rom_banks.push(bank);
        }

        let rom_bank_mask = match num_rom_banks - 1 {
            0..=1 => 0b0000_0001,
            2..=3 => 0b0000_0011,
            4..=7 => 0b0000_0111,
            8..=u16::MAX => 0b0000_1111,
        };

        let mut ram_banks = Vec::new();
        let save_file_path = get_save_file_path_from_rom_path(path);

        try_read_save_file(&save_file_path, 1, &mut ram_banks);

        Self {
            is_ram_enabled: false,
            current_rom_bank: 1,
            rom_banks,
            ram_banks,
            save_file_path,
            rom_bank_mask,
        }
    }
}

impl Cartridge for Mbc2 {
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
}

impl Drop for Mbc2 {
    fn drop(&mut self) {
        // create save file
        if !self.ram_banks.is_empty() {
            let mut sav_file = File::create(&self.save_file_path).unwrap();
            for bank in &self.ram_banks {
                sav_file.write_all(bank).unwrap();
            }
            log::info!("Save file written!");
        }
    }
}
