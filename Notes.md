### GB ASM

- https://eldred.fr/gb-asm-tutorial/index.html

### GBC IPS concerns

- https://discord.com/channels/465585922579103744/465586075830845475/912491020351078430
  > I'm not aware of any CGB test ROMs affected by IPS screen replacements, but if you want to develop your own test ROMs its probably a good idea to have a stock screen available to test with.
  > On the DMG there are test ROMs that are affected by biverting the original LCD, and some LCD <=> PPU desync behaviour that might not show up on replacement screens.

### GBC

- discord://discord.com/channels/465585922579103744/465586075830845475/909386242897088543

## PPU

- https://discord.com/channels/465585922579103744/465586075830845475/957856437927813170 special matt currie test

## HDMA

- https://github.com/TASEmulators/BizHawk/blob/master/src/BizHawk.Emulation.Cores/Consoles/Nintendo/GBHawk/GBC_GB_PPU.cs#L219-L220

- https://discord.com/channels/465585922579103744/465586075830845475/945049923181772800
  the only thing you're missing is that HDMA1/2, and HDMA3/4 are updated with new src/dst addresses after the dma is finished
  so basically just add the length to src/dst then write them back

- https://discord.com/channels/465585922579103744/465586075830845475/935953007995158598
  i don't think it's documented anywhere but HDMA5 is decremented at each hdma transfer, and HDMA1/2 and HDMA3/4 seem to get incremented by the transferred amount including in GDMA (according to samesuite dma tests)

## Annotated CGB BIOS disassembly

- http://www.its.caltech.edu/~costis/cgb_hack/gbc_bios.txt

## CGB "mode" selection

- https://discord.com/channels/465585922579103744/465586075830845475/921557609511804949

### Blargg

- log ASCII to 0xFF01

### MBC4

- https://twitter.com/LuigiBlood/status/1444084664952664073

### Sound Libraries

- https://discord.com/channels/465585922579103744/551430933836988438/911694138095829072

### Cool test roms

- https://github.com/pinobatch/little-things-gb
  - https://github.com/pinobatch/little-things-gb/tree/master/firstwhite

### SDL2 wasm example

- https://github.com/tanis2000/rust-sdl2-wasm

### Pinball

Just search for "pinball" in the #gb channel

- https://discord.com/channels/465585922579103744/465586075830845475/825443415188570143
- https://discord.com/channels/465585922579103744/465586075830845475/778959603257311263

### Color Correction

- http://web.archive.org/web/20210223205311/https://byuu.net/video/color-emulation/
- https://stackoverflow.com/questions/4409763/how-to-convert-from-rgb555-to-rgb888-in-c
