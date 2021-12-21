use crate::bus::Bus;

pub(crate) struct OamDma {
    io_reg: u8,

    transfer_index: u16,
    transfer_base_addr: u16,
    queue_counter: u8,
    queue_val: u16,
    active: bool,
    active_clock: u8,
}

impl Default for OamDma {
    fn default() -> Self {
        Self {
            io_reg: 0xFF,

            transfer_base_addr: 0,
            transfer_index: 0,
            queue_counter: 0,
            queue_val: 0,
            active: false,
            active_clock: 0,
        }
    }
}

impl OamDma {
    pub fn queue(&mut self, val: u8) {
        self.queue_val = val as u16;
        self.queue_counter = 5;
    }

    pub fn read_u8(&self) -> u8 {
        self.io_reg
    }

    pub fn write_u8(&mut self, val: u8) {
        self.io_reg = val;
        self.queue(val);
    }

    pub fn dma_tick(bus: &mut Bus) {
        // tick transfer, if active
        if bus.oam_dma.active {
            bus.oam_dma.active_clock += 1;
            if bus.oam_dma.active_clock == 4 {
                let src_val =
                    bus.read_u8(bus.oam_dma.transfer_base_addr + bus.oam_dma.transfer_index);
                bus.ppu.sprite_table[bus.oam_dma.transfer_index as usize] = src_val;
                bus.oam_dma.transfer_index += 1;

                if bus.oam_dma.transfer_index == 160 {
                    bus.oam_dma.active = false;
                }

                bus.oam_dma.active_clock = 0;
            }
        }

        // tick queue
        if bus.oam_dma.queue_counter > 0 {
            bus.oam_dma.queue_counter -= 1;

            if bus.oam_dma.queue_counter == 0 {
                bus.oam_dma.transfer_base_addr = bus.oam_dma.queue_val << 8;
                bus.oam_dma.transfer_index = 0;
                bus.oam_dma.active = true;
                bus.oam_dma.active_clock = 0;
            }
        }
    }
}
