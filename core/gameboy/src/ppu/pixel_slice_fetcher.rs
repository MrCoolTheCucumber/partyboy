#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{BGMapFlags, FifoPixel, LcdControlFlag, Ppu};

use crate::bus::CgbCompatibility;

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(super) enum FetchMode {
    Background(BackgroundFetchMode),
    Sprite,
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(super) enum BackgroundFetchMode {
    Background,
    Window,
}

impl Default for FetchMode {
    fn default() -> Self {
        Self::Background(BackgroundFetchMode::Background)
    }
}

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct BgFetcherState {
    tile_data_addr: u16,
    tile_id: u8,
    tile_attr: BGMapFlags,

    data_lo: u8,
    data_hi: u8,
}

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct SpriteFetcherState {
    tile_addr: u16,
    data_low: u8,
    data_high: u8,
}

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(super) struct PixelSliceFetcherState {
    cycle: u8,
    fetch_mode: FetchMode,
    bg_state: BgFetcherState,
    sprite_state: SpriteFetcherState,
}

impl Ppu {
    #[allow(unused)]
    pub fn reset_fetcher(&mut self) {
        self.fifo_state.fetcher = PixelSliceFetcherState::default();
    }

    pub fn tick_fetcher(&mut self) {
        match self.fifo_state.fetcher.fetch_mode {
            FetchMode::Background(mode) => self.tick_bg(mode),
            FetchMode::Sprite => self.tick_sprite(),
        }
    }

    fn tick_bg(&mut self, bg_fetch_mode: BackgroundFetchMode) {
        self.fifo_state.fetcher.cycle = self.fifo_state.fetcher.cycle.saturating_add(1);

        match self.fifo_state.fetcher.cycle {
            // fetch tile id
            2 => {
                let (bit_10, bit_9_to_5, bit_4_to_0) = match bg_fetch_mode {
                    BackgroundFetchMode::Background => {
                        let bit_10 =
                            ((self.lcdc & LcdControlFlag::BGTileMapAddress as u8) >> 3) as u16;
                        let bit_9_to_5 = (self.ly.wrapping_add(self.scy) / 8) as u16;
                        let bit_4_to_0 = (self.fifo_state.lx.wrapping_add(self.scy) / 8) as u16;
                        (bit_10, bit_9_to_5, bit_4_to_0)
                    }
                    BackgroundFetchMode::Window => {
                        let bit_10 =
                            ((self.lcdc & LcdControlFlag::WindowTileMapAddress as u8) >> 6) as u16;
                        let bit_9_to_5 = (self.wy / 8) as u16;
                        let bit_4_to_0 = (self.fifo_state.lx / 8) as u16;
                        (bit_10, bit_9_to_5, bit_4_to_0)
                    }
                };

                assert!(bit_10 <= 1);
                assert!(bit_9_to_5 & 0b1111_1111_1110_0000 == 0);
                assert!(bit_4_to_0 & 0b1111_1111_1110_0000 == 0);

                let map_addr: u16 =
                    0b1001_1000_0000_0000 | (bit_10 << 10) | (bit_9_to_5 << 5) | bit_4_to_0;
                let is_signed_tile_addressing = self.is_signed_tile_addressing();

                // self.fifo_state.fetcher.bg_state.tile_id =
                //     self.get_adjusted_tile_index(map_addr, is_signed_tile_addressing);
                self.fifo_state.fetcher.bg_state.tile_id =
                    self.gpu_vram[0][map_addr as usize - 0x8000];

                let flags = self.gpu_vram[1][map_addr as usize - 0x8000];
                self.fifo_state.fetcher.bg_state.tile_attr = BGMapFlags::from(flags);
            }

            // fetch tile data lo byte
            4 => {
                let bit_12 = match self.lcdc & LcdControlFlag::BGAndWindowTileData as u8 != 0 {
                    true => 0,
                    false => (!self.fifo_state.fetcher.bg_state.tile_id & 0b1000_0000) >> 7,
                } as u16;

                let mut bit_3_to_1 = match bg_fetch_mode {
                    BackgroundFetchMode::Background => (self.ly.wrapping_add(self.scy) % 8) as u16,
                    BackgroundFetchMode::Window => (self.wy % 8) as u16,
                };

                let tile_attr = self.fifo_state.fetcher.bg_state.tile_attr;
                let tile_id = self.fifo_state.fetcher.bg_state.tile_id;
                let bank_index = match self.console_compatibility_mode {
                    CgbCompatibility::CgbOnly => tile_attr.tile_bank,
                    _ => 0,
                };

                if self.console_compatibility_mode.is_cgb_mode() && tile_attr.vertical_flip {
                    bit_3_to_1 = (!bit_3_to_1) & 0b0000_0000_0000_0111;
                }
                assert!(bit_3_to_1 <= 0b0000_0000_0000_0111);

                let tile_data_addr = 0b1000_0000_0000_0000
                    | (bit_12 << 12)
                    | ((tile_id as u16) << 4)
                    | (bit_3_to_1 << 1);
                let mut tile_byte_lo = self.gpu_vram[bank_index][tile_data_addr as usize - 0x8000];

                if self.console_compatibility_mode.is_cgb_mode() && tile_attr.horizontal_flip {
                    tile_byte_lo = tile_byte_lo.reverse_bits()
                }

                self.fifo_state.fetcher.bg_state.tile_data_addr = tile_data_addr;
                self.fifo_state.fetcher.bg_state.data_lo = tile_byte_lo;
            }

            // fetch tile data hi byte
            6 => {
                // TODO: do i need to do: 'the first time we reach here we go back to step 1'?

                let tile_attr = self.fifo_state.fetcher.bg_state.tile_attr;
                let tile_data_addr = self.fifo_state.fetcher.bg_state.tile_data_addr + 1;
                let bank_index = match self.console_compatibility_mode {
                    CgbCompatibility::CgbOnly => tile_attr.tile_bank,
                    _ => 0,
                };

                let mut tile_byte_hi = self.gpu_vram[bank_index][tile_data_addr as usize - 0x8000];
                if self.console_compatibility_mode.is_cgb_mode() && tile_attr.horizontal_flip {
                    tile_byte_hi = tile_byte_hi.reverse_bits()
                }

                self.fifo_state.fetcher.bg_state.data_hi = tile_byte_hi;
            }

            8..=u8::MAX => {
                if !self.fifo_state.bg_fifo.is_empty() {
                    return;
                }

                let tile_attr = self.fifo_state.fetcher.bg_state.tile_attr;
                let b1 = self.fifo_state.fetcher.bg_state.data_lo;
                let b2 = self.fifo_state.fetcher.bg_state.data_hi;

                for i in 0..8 {
                    let bx = 7 - i;
                    let color_index = ((b1 & (1 << bx)) >> bx) | ((b2 & (1 << bx)) >> bx) << 1;

                    let palette_index = match self.console_compatibility_mode {
                        CgbCompatibility::CgbOnly => tile_attr.bg_palette_number,
                        _ => 0,
                    } as u8;

                    let priority = match self.console_compatibility_mode {
                        CgbCompatibility::CgbOnly => Some(tile_attr.bg_oam_prio as u8),
                        _ => None,
                    };

                    let px = FifoPixel {
                        color_index,
                        palette_index,
                        sprite_info: None,
                        priority,
                    };

                    self.fifo_state.bg_fifo.push_back(px);
                }

                // (0..8u8)
                //     .rev()
                //     .map(|shift| {
                //         ((b1 & (1 << shift)) >> shift) | ((b2 & (1 << shift)) >> shift) << 1
                //     })
                //     .map(|color_index| {
                //         let palette_index = match self.console_compatibility_mode {
                //             CgbCompatibility::CgbOnly => tile_attr.bg_palette_number,
                //             _ => 0,
                //         } as u8;

                //         let priority = match self.console_compatibility_mode {
                //             CgbCompatibility::CgbOnly => Some(tile_attr.bg_oam_prio as u8),
                //             _ => None,
                //         };

                //         FifoPixel {
                //             color_index,
                //             palette_index,
                //             sprite_info: None,
                //             priority,
                //         }
                //     })
                //     .for_each(|px| self.fifo_state.bg_fifo.push_back(px));

                self.fifo_state.fetcher.cycle = 0;
            }

            _ => {}
        }
    }

    fn tick_sprite(&mut self) {
        unimplemented!()
    }
}
