use serde::{Deserialize, Serialize};

use super::CartridgeInterface;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rom {
    #[serde(skip)]
    data: Vec<[u8; 0x4000]>,
}

impl Rom {
    pub fn new(rom: Vec<u8>) -> Self {
        assert_eq!(rom.len(), 0x8000);

        let data: Vec<[u8; 0x4000]> = rom
            .chunks_exact(0x4000)
            .map(|chunk| {
                let mut arr = [0; 0x4000];
                arr.copy_from_slice(chunk);
                arr
            })
            .collect();

        Self { data }
    }
}

impl CartridgeInterface for Rom {
    fn read_rom(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.data[0][addr as usize],
            0x4000..=0x7FFF => self.data[1][(addr - 0x4000) as usize],
            _ => panic!("Invalid address when reading from ROM cart"),
        }
    }

    fn write_rom(&mut self, _addr: u16, _value: u8) {
        // NOP
    }

    fn read_ram(&self, _addr: u16) -> u8 {
        0
    }

    fn write_ram(&mut self, _addr: u16, _value: u8) {
        // NOP
    }

    fn has_ram(&self) -> bool {
        false
    }

    fn ram_banks(&self) -> &Vec<[u8; 0x2000]> {
        unimplemented!("ROM has no RAM.");
    }

    fn load_rom(&mut self, rom: Vec<[u8; 0x4000]>) {
        self.data = rom;
    }

    fn take_rom(self) -> Vec<[u8; 0x4000]> {
        self.data
    }
}

#[cfg(test)]
use super::Cartridge;

#[cfg(test)]

pub fn create_test_rom() -> Cartridge {
    let rom = Rom {
        data: vec![[0; 0x4000], [0; 0x4000]],
    };

    Cartridge::Rom(rom)
}
