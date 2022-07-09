# ![Partyboy](https://user-images.githubusercontent.com/16002713/176773858-80ffaed3-a88a-42bf-a821-1189da071900.png)

A Game Boy Color emulator.

## Features

- Ability to play Game Boy Color games as well as Game Boy games in "compatibility mode"
- Cycle accurate CPU
- Support for most cartridge types
- Uses the boot rom from sameboy which is MIT lisenced

## Tests

See [this file](TestReport.md) for a report on all implemented tests

## Controls

| Button | Keyboard     |
| ------ | ------------ |
| A      | <kbd>O</kbd> |
| B      | <kbd>K</kbd> |
| START  | <kbd>M</kbd> |
| SELECT | <kbd>N</kbd> |
| UP     | <kbd>W</kbd> |
| DOWN   | <kbd>S</kbd> |
| LEFT   | <kbd>A</kbd> |
| RIGHT  | <kbd>D</kbd> |

You can also hold <kbd>TAB</kbd> to enable turbo, which will disable the frame limiter.

## Usage (CLI)

```
partyboy 1.0
A Gameboy color emulator

USAGE:
    partyboy [FLAGS] --rom <rom_path>

FLAGS:
    -l, --log        Enables file logging.
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -r, --rom <rom_path>    The path to the rom to load.
```

## TODO

- Improve UI to not rely on CLI
- Audio support
- Cycle accurate FIFO PPU

## Build Instructions

First, make sure you have the following dependentcies:

- [Rust](https://www.rust-lang.org/tools/install)
- SDL2 - read the requirements [here](https://github.com/Rust-SDL2/rust-sdl2#requirements) for your given OS

Then just run `cargo b` in the root directory of the repo.

## Running Tests

You will need to install python3 to run script that will download the test roms.
Once that is done, download the scripts:

`python .\scripts\download_test_roms.py`

Once that is done, run the tests by running the following in the root of the repo:

`cargo nextest run` or `python .\scripts\test_local.py`

## References

- https://gbdev.io/pandocs/
- https://izik1.github.io/gbops/
- https://rgbds.gbdev.io/docs/v0.5.1/gbz80.7
- https://tcrf.net/Notes:Game_Boy_Color_Bootstrap_ROM
- Many more...
