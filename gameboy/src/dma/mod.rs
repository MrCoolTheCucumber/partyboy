pub struct Dma {
    pub src_hi: u8,
    pub src_lo: u8,
    pub dest_hi: u8,
    pub dest_lo: u8,
    pub hdma5: u8, // length/start/mode?
}

impl Default for Dma {
    fn default() -> Self {
        Self {
            src_hi: 0x0,
            src_lo: 0x0,
            dest_hi: 0x0,
            dest_lo: 0x0,
            hdma5: 0x0,
        }
    }
}

impl Dma {
    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0xFF51 => self.src_hi,
            0xFF52 => self.src_lo,
            0xFF53 => self.dest_hi,
            0xFF54 => self.dest_lo,
            0xFF55 => self.hdma5 | 0b1000_000, // DMA always not active

            _ => panic!("HDMA doesnt handle reading from address: {:#06X}", addr),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF51 => self.src_hi = val,
            0xFF52 => self.src_lo = val,
            0xFF53 => self.dest_hi = val,
            0xFF54 => self.dest_lo = val,
            0xFF55 => self.hdma5 = val,

            _ => panic!("HDMA doesnt handle writing to address: {:#06X}", addr),
        }
    }
}
