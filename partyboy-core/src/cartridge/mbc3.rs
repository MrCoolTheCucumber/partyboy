#[cfg(feature = "serde")]
use {
    super::serialize::{ram_bank_deserialize, ram_bank_serialize},
    serde::{Deserialize, Serialize},
};

use super::{init_rom_and_ram, CartridgeInterface};

// TODO: RTC impl is broken?

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mbc3 {
    is_ram_rtc_enabled: bool,
    current_rom_bank: usize,
    current_ram_bank: usize,
    rom_bank_mask: u8,

    #[serde(skip)]
    rom_banks: Vec<[u8; 0x4000]>,

    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "ram_bank_serialize",
            deserialize_with = "ram_bank_deserialize"
        )
    )]
    ram_banks: Vec<[u8; 0x2000]>,

    rtc_regs: [u8; 5],
    rtc_banked: bool,

    prev_latch_val: u8,
}

impl Mbc3 {
    pub fn new(
        rom: Vec<u8>,
        ram: Option<Vec<u8>>,
        num_rom_banks: usize,
        num_ram_banks: usize,
    ) -> Self {
        let rom_bank_mask = match num_rom_banks - 1 {
            0..=1 => 0b0000_0001,
            2..=3 => 0b0000_0011,
            4..=7 => 0b0000_0111,
            8..=15 => 0b0000_1111,
            16..=31 => 0b0001_1111,
            32..=63 => 0b0011_1111,
            64..=127 => 0b0111_1111,
            _ => 0b1111_1111,
        };

        let (rom_banks, ram_banks) = init_rom_and_ram(rom, ram, num_rom_banks, num_ram_banks);

        Self {
            is_ram_rtc_enabled: false,
            current_rom_bank: 1,
            current_ram_bank: 0,
            rom_bank_mask,

            rtc_regs: [0; 5],
            rtc_banked: false,

            rom_banks,
            ram_banks,
            prev_latch_val: 204, // random val
        }
    }
}

impl CartridgeInterface for Mbc3 {
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
                    self.is_ram_rtc_enabled = true;
                } else if value == 0x0 {
                    self.is_ram_rtc_enabled = false;
                }
            }

            0x2000..=0x3FFF => {
                self.current_rom_bank = value as usize;
                if self.current_rom_bank == 0 {
                    self.current_rom_bank = 1
                }

                self.current_rom_bank &= self.rom_bank_mask as usize;
            }

            0x4000..=0x5FFF => {
                if value <= 0x03 {
                    self.current_ram_bank = value as usize;
                    self.rtc_banked = false;
                } else if (0x08..=0x0C).contains(&value) {
                    self.rtc_banked = true;
                }
            }

            0x6000..=0x7FFF => {
                if self.prev_latch_val == 0x00 && value == 0x01 {
                    let now = now_secs();

                    self.rtc_regs[0] = (now % 60) as u8;
                    self.rtc_regs[1] = ((now / 60) % 60) as u8;
                    self.rtc_regs[2] = (((now / 60) / 60) % 24) as u8;
                }

                self.prev_latch_val = value;
            }

            _ => panic!(),
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        if !self.is_ram_rtc_enabled {
            return 0xFF;
        }

        if self.rtc_banked {
            // return self.rtc_regs[(addr - 0x08) as usize];
        }

        self.ram_banks[self.current_ram_bank][addr as usize]
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        if !self.is_ram_rtc_enabled {
            return;
        }

        // what to do if rtc is banked?

        self.ram_banks[self.current_ram_bank][addr as usize] = value;
    }

    fn has_ram(&self) -> bool {
        !self.ram_banks.is_empty()
    }

    fn ram_banks(&self) -> &Vec<[u8; 0x2000]> {
        &self.ram_banks
    }

    fn load_rom(&mut self, rom: Vec<[u8; 0x4000]>) {
        self.rom_banks = rom;
    }

    fn take_rom(self) -> Vec<[u8; 0x4000]> {
        self.rom_banks
    }
}

#[cfg(not(feature = "web"))]
fn now_secs() -> u64 {
    use std::time::UNIX_EPOCH;
    UNIX_EPOCH.elapsed().unwrap().as_secs()
}

#[cfg(feature = "web")]
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(inline_js = r#"
export function performance_now() {
    return performance.now();
}
"#)]
#[cfg(feature = "web")]
extern "C" {
    fn performance_now() -> f64;
}

#[cfg(feature = "web")]
fn now_secs() -> u64 {
    performance_now() as u64
}
