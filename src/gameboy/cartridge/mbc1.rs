use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use super::{get_save_file_path_from_rom_path, try_read_save_file, Cartridge};

pub struct MBC1 {
    is_ram_enabled: bool,
    current_rom_bank: usize,
    current_ram_bank: usize,
    mode: u8, // 0 = ROM 1 = RAM

    rom_banks: Vec<[u8; 0x4000]>,
    ram_banks: Vec<[u8; 0x2000]>,

    save_file_path: PathBuf,
}

impl MBC1 {
    pub fn new(
        mut file: File,
        path: &Path,
        rom_bank_0: [u8; 0x4000],
        cartridge_type_code: u8,
        num_rom_banks: u16,
        num_ram_banks: u16,
    ) -> Self {
        let mut rom_banks = Vec::new();
        rom_banks.push(rom_bank_0);

        for _ in 0..num_rom_banks - 1 {
            let mut bank = [0; 0x4000];
            file.read_exact(&mut bank).ok();
            rom_banks.push(bank);
        }

        let mut ram_banks = Vec::new();
        let save_file_path = get_save_file_path_from_rom_path(path);

        // try to open save file
        let sav_file = File::open(&save_file_path);
        try_read_save_file(sav_file, num_ram_banks, &mut ram_banks);

        Self {
            is_ram_enabled: false,
            current_rom_bank: 1,
            current_ram_bank: 0,
            mode: 0,

            rom_banks,
            ram_banks,

            save_file_path,
        }
    }
}

impl Drop for MBC1 {
    fn drop(&mut self) {
        // create save file
        let mut sav_file = File::create(&self.save_file_path).unwrap();
        for bank in &self.ram_banks {
            sav_file.write_all(bank).unwrap();
        }
        println!("Save file written!");
    }
}

impl Cartridge for MBC1 {
    fn read_rom(&self, addr: u16) -> u8 {
        match addr & 0xF000 {
            0x0000 | 0x1000 | 0x2000 | 0x3000 => self.rom_banks[0][addr as usize],

            0x4000 | 0x5000 | 0x6000 | 0x7000 => {
                self.rom_banks[self.current_rom_bank][(addr - 0x4000) as usize]
            }

            _ => panic!(),
        }
    }

    fn write_rom(&mut self, addr: u16, value: u8) {
        match addr & 0xF000 {
            0x0000 | 0x1000 => {
                self.is_ram_enabled = (value & 0x0F) == 0x0A;
            }

            // TODO: The below match arms are wrong in some way
            0x2000 | 0x3000 => {
                self.current_rom_bank = (self.current_rom_bank & 0b0110_0000) + value as usize;
                if self.current_rom_bank == 0 {
                    self.current_rom_bank = 1
                }
            }

            0x4000 | 0x5000 => {
                if self.mode == 0 {
                    self.current_rom_bank = (self.current_rom_bank & 0x0b0001_1111)
                        + ((value & 0b0000_0011) << 5) as usize;
                } else {
                    panic!("mode 1 unimpl");
                }
            }

            0x6000 | 0x7000 => {
                self.mode = value & 1;
            }

            _ => panic!(),
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        if !self.is_ram_enabled {
            return 0xFF;
        }

        self.ram_banks[self.current_ram_bank][addr as usize]
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        if !self.is_ram_enabled {
            return;
        }

        self.ram_banks[self.current_ram_bank][addr as usize] = value;
    }
}
