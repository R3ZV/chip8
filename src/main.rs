mod chip8;

use inquire::{InquireError, Select};
use macroquad::prelude::*;
use std::fs;
use std::time::{Duration, SystemTime};

fn get_roms() -> Vec<String> {
    let entries = fs::read_dir("roms").expect("No roms folder");
    let mut roms = Vec::new();
    for entry in entries {
        let rom = entry.unwrap();
        let rom = rom
            .file_name()
            .into_string()
            .expect("Can't convert file name to String");
        roms.push(rom);
    }
    roms
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let options = get_roms();
    let ans = Select::new("What ROM do you want to run?", options).prompt();
    let selected_rom = ans.expect("No rom selected");

    let rom_path = format!("roms/{selected_rom}");
    let mut emulator = chip8::Chip8::new(rom_path);

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
