/*
   This file is based on the following src code:
   https://github.com/TASEmulators/BizHawk/blob/d4bb5e047e07dbfc078d171c99c80eafd01b5eb0/src/BizHawk.Emulation.Cores/Consoles/Nintendo/Gameboy/Gambatte.cs#L401-L529

   The gameboy color bios will initialize a color palette for gameboy games based on a hash of the cart header title.
   In order to still have colors for game boy games when booting in bios skip mode we need to replicate this functionality.
*/

use crate::cartridge::Cartridge;

pub fn get_color_palettes(cartridge: &dyn Cartridge) -> [u32; 12] {
    let hash = (0..16)
        .map(|i| cartridge.read_rom(0x134 + i) as u64)
        .sum::<u64>();

    // These are RGB32 colors
    match hash & 0xFF {
        0x01 | 0x10 | 0x29 | 0x52 | 0x5D | 0x68 | 0x6D | 0xF6 => [
            0xFFFFFF, 0xFFAD63, 0x843100, 0x000000, 0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
            0xFFFFFF, 0x7BFF31, 0x008400, 0x000000,
        ],
        0x0C | 0x16 | 0x35 | 0x67 | 0x75 | 0x92 | 0x99 | 0xB7 => [
            0xFFFFFF, 0xFFAD63, 0x843100, 0x000000, 0xFFFFFF, 0xFFAD63, 0x843100, 0x000000,
            0xFFFFFF, 0xFFAD63, 0x843100, 0x000000,
        ],
        0x14 => [
            0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000, 0xFFFFFF, 0x7BFF31, 0x008400, 0x000000,
            0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
        ],
        0x15 | 0xDB => [
            0xFFFFFF, 0xFFFF00, 0xFF0000, 0x000000, 0xFFFFFF, 0xFFFF00, 0xFF0000, 0x000000,
            0xFFFFFF, 0xFFFF00, 0xFF0000, 0x000000,
        ],
        0x17 | 0x8B => [
            0xFFFFFF, 0x7BFF31, 0x008400, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
        ],
        0x19 => [
            0xFFFFFF, 0xFF9C00, 0xFF0000, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
        ],
        0x1D => [
            0xA59CFF, 0xFFFF00, 0x006300, 0x000000, 0xFF6352, 0xD60000, 0x630000, 0x000000,
            0xFF6352, 0xD60000, 0x630000, 0x000000,
        ],
        0x34 => [
            0xFFFFFF, 0x7BFF00, 0xB57300, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
        ],
        0x36 => [
            0x52DE00, 0xFF8400, 0xFFFF00, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0x63A5FF, 0x0000FF,
            0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
        ],
        0x39 | 0x43 | 0x97 => [
            0xFFFFFF, 0xFFAD63, 0x843100, 0x000000, 0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
            0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
        ],
        0x3C => [
            0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000, 0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
            0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
        ],
        0x3D => [
            0xFFFFFF, 0x52FF00, 0xFF4200, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
        ],
        0x3E | 0xE0 => [
            0xFFFFFF, 0xFF9C00, 0xFF0000, 0x000000, 0xFFFFFF, 0xFF9C00, 0xFF0000, 0x000000,
            0xFFFFFF, 0x5ABDFF, 0xFF0000, 0x0000FF,
        ],
        0x49 | 0x5C => [
            0xA59CFF, 0xFFFF00, 0x006300, 0x000000, 0xFF6352, 0xD60000, 0x630000, 0x000000,
            0x0000FF, 0xFFFFFF, 0xFFFF7B, 0x0084FF,
        ],
        0x4B | 0x90 | 0x9A | 0xBD => [
            0xFFFFFF, 0x7BFF31, 0x008400, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
        ],
        0x4E => [
            0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            0xFFFFFF, 0xFFFF7B, 0x0084FF, 0xFF0000,
        ],
        0x58 => [
            0xFFFFFF, 0xA5A5A5, 0x525252, 0x000000, 0xFFFFFF, 0xA5A5A5, 0x525252, 0x000000,
            0xFFFFFF, 0xA5A5A5, 0x525252, 0x000000,
        ],
        0x59 => [
            0xFFFFFF, 0xADAD84, 0x42737B, 0x000000, 0xFFFFFF, 0xFF7300, 0x944200, 0x000000,
            0xFFFFFF, 0x5ABDFF, 0xFF0000, 0x0000FF,
        ],
        0x69 | 0xF2 => [
            0xFFFFFF, 0xFFFF00, 0xFF0000, 0x000000, 0xFFFFFF, 0xFFFF00, 0xFF0000, 0x000000,
            0xFFFFFF, 0x5ABDFF, 0xFF0000, 0x0000FF,
        ],
        0x6B => [
            0xFFFFFF, 0x8C8CDE, 0x52528C, 0x000000, 0xFFC542, 0xFFD600, 0x943A00, 0x4A0000,
            0xFFFFFF, 0x5ABDFF, 0xFF0000, 0x0000FF,
        ],
        0x6F => [
            0xFFFFFF, 0xFFCE00, 0x9C6300, 0x000000, 0xFFFFFF, 0xFFCE00, 0x9C6300, 0x000000,
            0xFFFFFF, 0xFFCE00, 0x9C6300, 0x000000,
        ],
        0x70 => [
            0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000, 0xFFFFFF, 0x00FF00, 0x318400, 0x004A00,
            0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
        ],
        0x71 | 0xFF => [
            0xFFFFFF, 0xFF9C00, 0xFF0000, 0x000000, 0xFFFFFF, 0xFF9C00, 0xFF0000, 0x000000,
            0xFFFFFF, 0xFF9C00, 0xFF0000, 0x000000,
        ],
        0x86 | 0xA8 => [
            0xFFFF9C, 0x94B5FF, 0x639473, 0x003A3A, 0xFFC542, 0xFFD600, 0x943A00, 0x4A0000,
            0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
        ],
        0x88 => [
            0xA59CFF, 0xFFFF00, 0x006300, 0x000000, 0xA59CFF, 0xFFFF00, 0x006300, 0x000000,
            0xA59CFF, 0xFFFF00, 0x006300, 0x000000,
        ],
        0x8C => [
            0xFFFFFF, 0xADAD84, 0x42737B, 0x000000, 0xFFFFFF, 0xFF7300, 0x944200, 0x000000,
            0xFFFFFF, 0xADAD84, 0x42737B, 0x000000,
        ],
        0x95 => [
            0xFFFFFF, 0x52FF00, 0xFF4200, 0x000000, 0xFFFFFF, 0x52FF00, 0xFF4200, 0x000000,
            0xFFFFFF, 0x5ABDFF, 0xFF0000, 0x0000FF,
        ],
        0x9C => [
            0xFFFFFF, 0x8C8CDE, 0x52528C, 0x000000, 0xFFFFFF, 0x8C8CDE, 0x52528C, 0x000000,
            0xFFC542, 0xFFD600, 0x943A00, 0x4A0000,
        ],
        0x9D => [
            0xFFFFFF, 0x8C8CDE, 0x52528C, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            0xFFFFFF, 0xFFAD63, 0x843100, 0x000000,
        ],
        0xA2 | 0xF7 => [
            0xFFFFFF, 0xFFAD63, 0x843100, 0x000000, 0xFFFFFF, 0x7BFF31, 0x008400, 0x000000,
            0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
        ],
        0xAA => [
            0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000,
        ],
        0xC9 => [
            0xFFFFCE, 0x63EFEF, 0x9C8431, 0x5A5A5A, 0xFFFFFF, 0xFF7300, 0x944200, 0x000000,
            0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
        ],
        0xCE | 0xD1 | 0xF0 => [
            0x6BFF00, 0xFFFFFF, 0xFF524A, 0x000000, 0xFFFFFF, 0xFFFFFF, 0x63A5FF, 0x0000FF,
            0xFFFFFF, 0xFFAD63, 0x843100, 0x000000,
        ],
        0xE8 => [
            0x000000, 0x008484, 0xFFDE00, 0xFFFFFF, 0x000000, 0x008484, 0xFFDE00, 0xFFFFFF,
            0x000000, 0x008484, 0xFFDE00, 0xFFFFFF,
        ],
        0x0D => match cartridge.read_rom(0x137) {
            0x45 => [
                0xFFFFFF, 0x8C8CDE, 0x52528C, 0x000000, 0xFFC542, 0xFFD600, 0x943A00, 0x4A0000,
                0xFFC542, 0xFFD600, 0x943A00, 0x4A0000,
            ],
            0x52 => [
                0xFFFFFF, 0xFFFF00, 0xFF0000, 0x000000, 0xFFFFFF, 0xFFFF00, 0xFF0000, 0x000000,
                0xFFFFFF, 0x5ABDFF, 0xFF0000, 0x0000FF,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0x18 => match cartridge.read_rom(0x137) {
            0x4B => [
                0xFFFFFF, 0x8C8CDE, 0x52528C, 0x000000, 0xFFC542, 0xFFD600, 0x943A00, 0x4A0000,
                0xFFFFFF, 0x5ABDFF, 0xFF0000, 0x0000FF,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0x27 => match cartridge.read_rom(0x137) {
            0x42 => [
                0xA59CFF, 0xFFFF00, 0x006300, 0x000000, 0xFF6352, 0xD60000, 0x630000, 0x000000,
                0x0000FF, 0xFFFFFF, 0xFFFF7B, 0x0084FF,
            ],
            0x4E => [
                0xFFFFFF, 0x7BFF31, 0x008400, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0x28 => match cartridge.read_rom(0x137) {
            0x41 => [
                0x000000, 0x008484, 0xFFDE00, 0xFFFFFF, 0x000000, 0x008484, 0xFFDE00, 0xFFFFFF,
                0x000000, 0x008484, 0xFFDE00, 0xFFFFFF,
            ],
            0x46 => [
                0xFFFFFF, 0x7BFF31, 0x008400, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0x46 => match cartridge.read_rom(0x137) {
            0x45 => [
                0xB5B5FF, 0xFFFF94, 0xAD5A42, 0x000000, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A,
                0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A,
            ],
            0x52 => [
                0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000, 0xFFFF00, 0xFF0000, 0x630000, 0x000000,
                0xFFFFFF, 0x7BFF31, 0x008400, 0x000000,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0x61 => match cartridge.read_rom(0x137) {
            0x41 => [
                0xFFFFFF, 0x7BFF31, 0x008400, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
            ],
            0x45 => [
                0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0x66 => match cartridge.read_rom(0x137) {
            0x45 => [
                0xFFFFFF, 0x7BFF00, 0xB57300, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0x6A => match cartridge.read_rom(0x137) {
            0x49 => [
                0xFFFFFF, 0x52FF00, 0xFF4200, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
            0x4B => [
                0xFFFFFF, 0x8C8CDE, 0x52528C, 0x000000, 0xFFC542, 0xFFD600, 0x943A00, 0x4A0000,
                0xFFFFFF, 0x5ABDFF, 0xFF0000, 0x0000FF,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0xA5 => match cartridge.read_rom(0x137) {
            0x41 => [
                0x000000, 0x008484, 0xFFDE00, 0xFFFFFF, 0x000000, 0x008484, 0xFFDE00, 0xFFFFFF,
                0x000000, 0x008484, 0xFFDE00, 0xFFFFFF,
            ],
            0x52 => [
                0xFFFFFF, 0xFFAD63, 0x843100, 0x000000, 0xFFFFFF, 0x7BFF31, 0x008400, 0x000000,
                0xFFFFFF, 0x7BFF31, 0x008400, 0x000000,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0xB3 => match cartridge.read_rom(0x137) {
            0x42 => [
                0xA59CFF, 0xFFFF00, 0x006300, 0x000000, 0xFF6352, 0xD60000, 0x630000, 0x000000,
                0x0000FF, 0xFFFFFF, 0xFFFF7B, 0x0084FF,
            ],
            0x52 => [
                0xFFFFFF, 0x52FF00, 0xFF4200, 0x000000, 0xFFFFFF, 0x52FF00, 0xFF4200, 0x000000,
                0xFFFFFF, 0x5ABDFF, 0xFF0000, 0x0000FF,
            ],
            0x55 => [
                0xFFFFFF, 0xADAD84, 0x42737B, 0x000000, 0xFFFFFF, 0xFF7300, 0x944200, 0x000000,
                0xFFFFFF, 0xFF7300, 0x944200, 0x000000,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0xBF => match cartridge.read_rom(0x137) {
            0x20 => [
                0xFFFFFF, 0x8C8CDE, 0x52528C, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
            0x43 => [
                0x6BFF00, 0xFFFFFF, 0xFF524A, 0x000000, 0xFFFFFF, 0xFFFFFF, 0x63A5FF, 0x0000FF,
                0xFFFFFF, 0xFFAD63, 0x843100, 0x000000,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0xC6 => match cartridge.read_rom(0x137) {
            0x41 => [
                0xFFFFFF, 0xADAD84, 0x42737B, 0x000000, 0xFFFFFF, 0xFF7300, 0x944200, 0x000000,
                0xFFFFFF, 0x5ABDFF, 0xFF0000, 0x0000FF,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0xD3 => match cartridge.read_rom(0x137) {
            0x49 => [
                0xFFFFFF, 0xADAD84, 0x42737B, 0x000000, 0xFFFFFF, 0xFFAD63, 0x843100, 0x000000,
                0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
            ],
            0x52 => [
                0xFFFFFF, 0x8C8CDE, 0x52528C, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0x8C8CDE, 0x52528C, 0x000000,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        0xF4 => match cartridge.read_rom(0x137) {
            0x20 => [
                0xFFFFFF, 0x7BFF00, 0xB57300, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
            0x2D => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0x63A5FF, 0x0000FF, 0x000000,
            ],
            _ => [
                0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
                0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            ],
        },
        _ => [
            0xFFFFFF, 0x7BFF31, 0x0063C5, 0x000000, 0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
            0xFFFFFF, 0xFF8484, 0x943A3A, 0x000000,
        ],
    }
}
