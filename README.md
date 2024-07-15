# Chip8 Emulator
Rust implementation of the Chip8 instruction set.

The original screen dimensions where 64x32 but I have scaled that
to 800x600

I use [Marcoquad](https://macroquad.rs/) to display the graphics.

Emulator tested using [Timendus test suite](https://github.com/Timendus/chip8-test-suite/tree/main)

# Run locally
You only need nix-shell installed.

```terminal
git clone git@github.com:r3zv/chip8.git

cd chip8

nix-shell shell.nix

cargo run
```

# References
- https://github.com/mattmikolay/chip-8
- https://chip-8.github.io/links/
- https://github.com/JohnEarnest/chip8Archive/tree/master/roms
