use super::{Cartridge, RamIter};

pub struct Rom {
    data: Box<[u8]>,
}

impl Rom {
    pub fn new(rom: Vec<u8>) -> Self {
        assert_eq!(rom.len(), 0x8000);
        Self {
            data: rom.into_boxed_slice(),
        }
    }
}

impl Cartridge for Rom {
    fn read_rom(&self, addr: u16) -> u8 {
        match addr < 0x8000 {
            true => self.data[addr as usize],
            false => panic!("Invalid address when reading from ROM cart"),
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

    fn iter_ram(&self) -> RamIter {
        RamIter::empty()
    }
}

#[cfg(test)]
pub fn create_test_rom() -> Rom {
    Rom {
        data: Box::new([0; 0x8000]),
    }
}
