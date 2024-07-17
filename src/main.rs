mod chip8;

use macroquad::prelude::*;
use std::time::{Duration, SystemTime};

#[macroquad::main("BasicShapes")]
async fn main() {
    let path = String::from("roms/corax.ch8");
    let mut emulator = chip8::Chip8::new(path);

    // Chip8 timer should be updated at a rate of 60hz
    // So will simulate that by decreasing the timer every 16.67ms

    let mut curr_time = SystemTime::now();
    let tick_timer = Duration::from_nanos(16_670_000);

    loop {
        if let Ok(elapsed) = curr_time.elapsed() {
            if elapsed > tick_timer {
                emulator.tick();
                curr_time = SystemTime::now();
            }
        } else {
            eprintln!("Couldn't retrieve elapsed time from system timer");
        }

        emulator.start_cycle();

        emulator.update_screen();
        next_frame().await;
    }
}
