mod chip8;

use macroquad::prelude::*;

#[macroquad::main("BasicShapes")]
async fn main() {
    let path = String::from("roms/chip8-logo.ch8");
    let mut emulator = chip8::Chip8::new(path);

    loop {
        emulator.start_cycle();
        emulator.update_screen();
        next_frame().await
    }
}
