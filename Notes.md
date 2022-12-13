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
- http://blog.kevtris.org/blogfiles/Nitty%20Gritty%20Gameboy%20VRAM%20Timing.txt

## HDMA

Based on what I've read so far:

- HDMA is checked inbetween instructions (similar to interrupts), cannot be stopped by interrupts
- Has a 4t delay
- If a HDMA is requested, it is triggered on ppu non-mode 0 to mode 0 (rising edge)
  - So basically, if we're in mode 0 then we can start?
- This means turning off the lcd can trigger HDMA
- If the ppu is off then only one block gets copied
- If CPU is HALT or STOP, or if a speed switch is happening, then "HDMA won't happen" (what does this mean exactly?)

- https://github.com/TASEmulators/BizHawk/blob/master/src/BizHawk.Emulation.Cores/Consoles/Nintendo/GBHawk/GBC_GB_PPU.cs#L219-L220

- https://discord.com/channels/465585922579103744/465586075830845475/945049923181772800
  the only thing you're missing is that HDMA1/2, and HDMA3/4 are updated with new src/dst addresses after the dma is finished
  so basically just add the length to src/dst then write them back

- https://discord.com/channels/465585922579103744/465586075830845475/935953007995158598
  i don't think it's documented anywhere but HDMA5 is decremented at each hdma transfer, and HDMA1/2 and HDMA3/4 seem to get incremented by the transferred amount including in GDMA (according to samesuite dma tests)

- Godlike notes
  https://discord.com/channels/465585922579103744/465586075830845475/990683407086391356

- TCAGBD has a lot of info

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

### Pinball

Just search for "pinball" in the #gb channel

- https://discord.com/channels/465585922579103744/465586075830845475/825443415188570143
- https://discord.com/channels/465585922579103744/465586075830845475/778959603257311263

### Color Correction

- http://web.archive.org/web/20210223205311/https://byuu.net/video/color-emulation/
- https://stackoverflow.com/questions/4409763/how-to-convert-from-rgb555-to-rgb888-in-c

### Odd Mode

- "You're speed switching with LCDC on
  it causes odd mode"
- Odd mode is when the ppu/cpu are out of sync, there are 3 possible allignments
  "there's 4 alignments given 1 single-speed m-cycle is 4 dots, 3 of which are "odd""
- "Confirmed!
  Turning the LCD on in double speed mode, then switching back to single speed mode, will offset the PPU by either 1 or 3 T-cycles
  This means there are 3 different odd modes, and it's not possible to avoid odd mode when doing this switch"
- STOP resets DIV, therefore speedswitch resets DIV
- https://github.com/pokemon-speedrunning/gambatte-core/tree/master/test/hwtests/lcd_offset gambatte tests for oddmode

### Cgb Acid Hell

- discord://discord.com/channels/465585922579103744/465586075830845475/994213676271796266

### Web

- https://www.reddit.com/r/rust/comments/kyae22/is_possible_to_compile_the_std_lib_of_rust_to_wasm/
- https://rustwasm.github.io/wasm-bindgen/introduction.html
- https://web.dev/webassembly-threads/

Architecture:
Spawn web worker, use multiple shared arrays to handle communication. E.g. one for frame buffer, one for basic messages, etc

### Fifo

- https://github.com/gbdev/pandocs/pull/379

### Sound

- https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware
- https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html
- https://www.reddit.com/r/EmuDev/comments/5gkwi5/comment/dat3zni/

- TODO: Volume/envelope registeres need to turn the channel off if the initial volume is set to 0
