mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
pub mod rom;

use crate::cartridge::{mbc1::Mbc1, mbc2::Mbc2, mbc3::Mbc3, mbc5::Mbc5, rom::Rom};

pub struct RamIter {
    #[allow(dead_code)]
    pub(self) ram: Vec<u8>,
}

impl RamIter {
    pub fn empty() -> Self {
        Self { ram: Vec::new() }
    }
}

impl From<Vec<u8>> for RamIter {
    fn from(ram: Vec<u8>) -> Self {
        Self { ram }
    }
}

impl RamIter {
    #[allow(dead_code)]
    pub fn as_slice(&self) -> &[u8] {
        self.ram.as_slice()
    }
}

pub trait Cartridge {
    fn read_rom(&self, addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, value: u8);

    fn read_ram(&self, addr: u16) -> u8;
    fn write_ram(&mut self, addr: u16, value: u8);

    // we cant return `impl trait` in rust yet so lets do this
    fn iter_ram(&self) -> RamIter;
    fn has_ram(&self) -> bool;

    // fn create_save_file(&self) {
    //     let ram_iter = self.iter_ram();
    //     let save_file_path = self.save_file_path();

    //     if let Some(save_file_path) = save_file_path {
    //         if self.has_ram() {
    //             let mut sav_file = File::create(save_file_path).unwrap();
    //             // TODO: handle result
    //             let _ = sav_file.write_all(ram_iter.as_slice());
    //             log::info!("Save file written!");
    //         }
    //     }
    // }
}

// TODO: this should probably return a result, and be an error
// if e.g. the vec is too small or something
pub fn create(rom: Vec<u8>, ram: Option<Vec<u8>>) -> Box<dyn Cartridge> {
    // parse cart header
    // CGB flag
    log::info!("CGB compat mode: {:#04X}", rom[0x143]);
    if rom[0x143] == 0xC0 {
        log::warn!("This rom is only supported for game boy color");
    }

    let cartridge_type_code = rom[0x147];
    let rom_size_code = rom[0x148];
    let ram_size_code = rom[0x149];

    // This includes rom bank 0
    let num_rom_banks: usize = match rom_size_code {
        0x00 => 2,   // 32KB
        0x01 => 4,   // 64KB
        0x02 => 8,   // 128KB
        0x03 => 16,  // 256KB
        0x04 => 32,  // 512KB
        0x05 => 64,  // 1MB
        0x06 => 128, // 2MB
        0x07 => 256, // 4MB
        0x08 => 512, // 8MB
        0x09 => 1024,

        // pandocs says there are some other special codes
        // but is not sure if they are legit
        // lets define them anyway
        0x52 => 72, // 1.1MB
        0x53 => 80, // 1.2MB
        0x54 => 96, // 1.5MB

        _ => panic!(
            "Cartridge has invalid ROM size code? Code: {:#04X}",
            rom_size_code
        ),
    };

    log::debug!("rom size code: {}, banks: {}", rom_size_code, num_rom_banks);

    let num_ram_banks: usize = match ram_size_code {
        0x00 => 0,
        0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        0x05 => 8,

        _ => panic!(
            "Cartridge has invalid RAM size code? Code: {:#04X}",
            ram_size_code
        ),
    };

    log::debug!("ram size code: {}, banks: {}", ram_size_code, num_ram_banks);

    match cartridge_type_code {
        0x00 => Box::new(Rom::new(rom)),

        0x01 | 0x02 | 0x03 => {
            log::info!("MBC1 cart detected!");
            Box::new(Mbc1::new(rom, ram, num_rom_banks, num_ram_banks))
        }

        0x05 | 0x06 => {
            log::info!("MBC2 cart detected!");
            Box::new(Mbc2::new(rom, ram, num_rom_banks, num_ram_banks))
        }

        0x0F..=0x13 => {
            log::info!("MBC3 cart detected!");
            Box::new(Mbc3::new(rom, ram, num_rom_banks, num_ram_banks))
        }

        0x19..=0x1E => {
            log::info!("MBC5 cart detected!");
            Box::new(Mbc5::new(rom, ram, num_rom_banks, num_ram_banks))
        }

        _ => unimplemented!(
            "Unable to handle cartridge type: {:#04X}",
            cartridge_type_code
        ),
    }
}

fn init_rom_and_ram(
    rom: Vec<u8>,
    ram: Option<Vec<u8>>,
    num_rom_banks: usize,
    num_ram_banks: usize,
) -> (Vec<[u8; 0x4000]>, Vec<[u8; 0x2000]>) {
    assert_eq!(rom.len(), num_rom_banks * 0x4000);
    let rom_banks: Vec<[u8; 0x4000]> = rom
        .chunks_exact(0x4000)
        .map(|chunk| {
            let mut arr = [0; 0x4000];
            arr.copy_from_slice(chunk);
            arr
        })
        .collect();

    let ram_banks: Vec<[u8; 0x2000]> = match ram {
        Some(ram) => {
            assert_eq!(ram.len(), num_ram_banks * 0x2000);
            ram.chunks_exact(0x2000)
                .map(|chunk| {
                    let mut arr = [0; 0x2000];
                    arr.copy_from_slice(chunk);
                    arr
                })
                .collect()
        }
        None => (0..num_ram_banks).map(|_| [0u8; 0x2000]).collect(),
    };

    assert_eq!(num_rom_banks, rom_banks.len());
    assert_eq!(num_ram_banks, ram_banks.len());

    (rom_banks, ram_banks)
}
