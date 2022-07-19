use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use super::{get_save_file_path_from_rom_path, try_read_save_file, Cartridge, RamIter};

pub struct Mbc5 {
    is_ram_enabled: bool,

    rom_bank_mask: u8,
    current_rom_bank: usize,
    current_ram_bank: usize,

    rom_banks: Vec<[u8; 0x4000]>,
    ram_banks: Vec<[u8; 0x2000]>,

    save_file_path: PathBuf,
}

impl Mbc5 {
    pub fn new(
        mut file: File,
        path: &Path,
        rom_bank_0: [u8; 0x4000],
        num_rom_banks: u16,
        num_ram_banks: u16,
    ) -> Self {
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
            8..=15 => 0b0000_1111,
            16..=31 => 0b0001_1111,
            32..=63 => 0b0011_1111,
            64..=127 => 0b0111_1111,
            128..=u16::MAX => 0b1111_1111,
        };

        let mut ram_banks = Vec::new();

        let save_file_path = get_save_file_path_from_rom_path(path);
        try_read_save_file(&save_file_path, num_ram_banks, &mut ram_banks);

        Self {
            is_ram_enabled: false,

            rom_bank_mask,
            current_rom_bank: 1,
            current_ram_bank: 0,

            rom_banks,
            ram_banks,

            save_file_path,
        }
    }
}

impl Cartridge for Mbc5 {
    fn read_rom(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom_banks[0][addr as usize],
            0x4000..=0x7FFF => self.rom_banks[self.current_rom_bank][(addr - 0x4000) as usize],
            _ => panic!(),
        }
    }

    fn write_rom(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let value = value & 0x0F;

                if value == 0x0A {
                    self.is_ram_enabled = true;
                } else if value == 0x0 {
                    self.is_ram_enabled = false;
                }
            }

            0x2000..=0x2FFF => {
                self.current_rom_bank =
                    (self.current_rom_bank & 0b1_0000_0000) | (value & self.rom_bank_mask) as usize;
            }

            0x3000..=0x3FFF => {
                self.current_rom_bank =
                    (self.current_rom_bank & 0b0_1111_1111) + (((value & 1) as usize) << 8);
            }

            0x4000..=0x5FFF => {
                if value <= 0x0F {
                    self.current_ram_bank = value as usize;
                }
            }

            0x6000..=0x7FFF => {}

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

    fn has_ram(&self) -> bool {
        !self.ram_banks.is_empty()
    }

    fn iter_ram(&self) -> RamIter {
        let iter = self
            .ram_banks
            .iter()
            .flat_map(|slice| slice.iter())
            .copied()
            .collect::<Vec<u8>>();
        iter.into()
    }

    fn save_file_path(&self) -> Option<&PathBuf> {
        Some(&self.save_file_path)
    }
}
