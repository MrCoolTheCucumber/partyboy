pub struct Hdma {
    pub src_hi: u8,
    pub src_lo: u8,
    pub dest_hi: u8,
    pub dest_lo: u8,
    pub hdma5: u8, // length/start/mode?
}

impl Default for Hdma {
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

impl Hdma {
    pub fn read_u8(&self, addr: u16) -> u8 {
        log::debug!("Reading a HDMA reg: {:#06X}", addr);
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

            0xFF55 => {
                self.hdma5 = val;
                let bytes_to_transfer: u16 = ((val & 0b0111_1111) + 1) as u16 * 0x10;

                // TOOD: for now, just ignore mode and instantly copy?
                let _is_gdma = (val & 0b1000_0000) == 0;

                let mut src_addr: u16 = ((self.src_hi as u16) << 8) | ((self.src_lo) as u16);
                let mut dest_addr: u16 = (((self.dest_hi) as u16) << 8) | ((self.dest_lo) as u16);

                // Apply "masks"
                src_addr &= 0b0111_1111_1111_0000;
                dest_addr &= 0b0001_1111_1111_0000;
                dest_addr |= 0b1000_0000_0000_0000;

                for i in 0..bytes_to_transfer {
                    let transfer_val = self.read_u8(src_addr + i);
                    self.write_u8(dest_addr + i, transfer_val)
                }

                log::debug!(
                    "Executing HDMA: Src: {:#06X}, Dest: {:#06X}",
                    src_addr,
                    dest_addr
                );
            }

            _ => panic!("HDMA doesnt handle writing to address: {:#06X}", addr),
        }
    }
}
