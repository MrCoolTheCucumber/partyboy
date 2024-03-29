<div align="center">
    <img src="https://user-images.githubusercontent.com/16002713/176773858-80ffaed3-a88a-42bf-a821-1189da071900.png" alt="Logo">
    <br/>
    <img alt="GitHub Workflow Status (branch)" src="https://img.shields.io/github/actions/workflow/status/MrCoolTheCucumber/partyboy/build.yml?branch=master&style=flat-square">
    <img alt="GitHub last commit" src="https://img.shields.io/github/last-commit/MrCoolTheCucumber/partyboy?style=flat-square">
    <img alt="GitHub issues" src="https://img.shields.io/github/issues/MrCoolTheCucumber/partyboy?style=flat-square">
    <br/>
    A Game Boy Color emulator.
    <br/>
    <a href="https://mrcoolthecucumber.com/partyboy">Try it out in the browser</a>
</div>

---

<div align="center">
    <img alt="cgb boot" src="https://user-images.githubusercontent.com/16002713/206076824-d8f6bd3b-e383-4d2c-9bad-494303dd5d22.gif">
    <img alt="zelda intro" src="https://user-images.githubusercontent.com/16002713/206078363-c5231156-bea5-48c9-8f83-283d35bda2a7.gif">
</div>

## Features

- Ability to play Game Boy Color games as well as Game Boy games in the game boy colors DMG mode
- Fairly high accuracy
- Support for most cartridge types
- Audio
- Biosless mode
- Save states
- Rewinding
- Web support

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

You can also hold <kbd>SPACE</kbd> to enable turbo, which will disable the frame limiter. And hold <kbd>Q</kbd> to rewind!

## Usage (CLI)

```
partyboy 1.0
A Gameboy color emulator

USAGE:
    partyboy.exe [FLAGS] [OPTIONS]

FLAGS:
    -l, --log        Enables file logging.
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --bios <bios_path>    The path to the bios to use.
    -r, --rom <rom_path>      The path to the rom to load.
```

## TODO

- [ ] Dissasembler for debug frontend
- [ ] Make `frontend_debug` work for web
- [ ] Implement some sort of mass rom test suite similar to [sameboy](https://sameboy.github.io/automation/)?

## Build Instructions

First, make sure you have the following dependentcies:

- [Rust](https://www.rust-lang.org/tools/install)
- On linux you might need the following packages: `build-essentials, cmake, libfontconfig1-dev, alsa, libasound2-dev`

Then just run `cargo b` in the root directory of the repo.

## Running Tests

You will need to install:

- python
- [nextest](https://nexte.st/)

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
