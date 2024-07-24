use macroquad::{prelude::*, rand};

#[derive(Debug)]
enum Instruction {
    Clear,                          // 00E0
    Return,                         // 00EE
    Jump(usize),                    // 1NNN
    SubRoutine(usize),              // 2NNN
    SkipOnXeqV(u8, u8),             // 3XNN
    SkipOnXneqV(u8, u8),            // 4XNN
    SkipOnXeqY(u8, u8),             // 5XY0
    LoadNormalRegister(u8, u8),     // 6XNN
    AddToNormalRegister(u8, u8),    // 7XNN
    SetXtoY(u8, u8),                // 8XY0
    SetXtoXorY(u8, u8),             // 8XY1
    SetXtoXandY(u8, u8),            // 8XY2
    SetXtoXxorY(u8, u8),            // 8XY3
    AddYtoX(u8, u8),                // 8XY4
    SubYfromX(u8, u8),              // 8XY5
    SetXtoYshiftRightOnce(u8, u8),  // 8XY6
    SetXtoYMinusX(u8, u8),          // 8XY7
    SetXtoYshiftLeftOnce(u8, u8),   // 8XYE
    SkipOnXneqY(u8, u8),            // 9XY0
    LoadIndexRegister(u16),         // ANNN
    JumpByRegister(usize),          // BNNN
    LoadRegisterWithRandom(u8, u8), // CNNN
    DrawSprite(u8, u8, u8),         // DXYN
    StoreDeelayInRegister(u8),      // FX07
    SetDeelayFromRegister(u8),      // FX15
    SetSoundTimerFromRegister(u8),  // FX18
    AddRegisterToIndex(u8),         // FX1E
    StoreRegisterInBCD(u8),         // FX33
    StoreRegistersInMemmory(u8),    // FX55
    FillRegisters(u8),              // FX65
}

#[derive(Debug)]
pub struct Chip8 {
    // registers, VF often used as a flag
    v: [u8; 16],

    // index register
    i: u16,

    pc: usize,
    ram: [u8; 4 * 1024],

    // count down with a frequency of 60hz
    deelay: u8,

    // cound down only for values greater than 0x01
    sound_timer: u8,

    // 0 = black, 1 = white
    // to draw a sprite we XOR with the screen data
    // if the sprite is offscreen we modulo 64 and 32
    // every sprite is 8 pixels wide and height [1, 15]
    screen: [[u8; 64]; 32],

    screen_update: bool,

    stack: Vec<usize>,

    // if set skips the next instruction
    // to be executed
    skip_instruction: bool,
}

impl Chip8 {
    pub fn new(path: String) -> Self {
        // TODO: load the font
        let rom_data = std::fs::read(path).expect("No source file found");
        let mut ram = [0; 4 * 1024];
        for i in 0..rom_data.len() {
            ram[0x200 + i] = rom_data[i];
        }

        Chip8 {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            ram,
            deelay: 0,
            sound_timer: 0,
            screen: [[0; 64]; 32],
            screen_update: false,
            stack: Vec::new(),
            skip_instruction: false,
        }
    }

    pub fn tick(&mut self) {
        if self.deelay > 0 {
            self.deelay -= 1;
        }

        if self.sound_timer > 1 {
            self.sound_timer -= 1;
        }
    }

    pub fn update_screen(&self) {
        let pixel_width = screen_width() / 64.0;
        let pixel_height = screen_height() / 32.0;

        for y in 0..self.screen.len() {
            for x in 0..self.screen[0].len() {
                if self.screen[y][x] == 1 {
                    draw_rectangle(
                        pixel_width * x as f32,
                        pixel_height * y as f32,
                        pixel_width,
                        pixel_height,
                        ORANGE,
                    )
                } else {
                    draw_rectangle(
                        pixel_width * x as f32,
                        pixel_height * y as f32,
                        pixel_width,
                        pixel_height,
                        BLACK,
                    )
                }
            }
        }
    }

    fn exec(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Clear => {
                for i in 0..self.screen.len() {
                    for j in 0..self.screen[i].len() {
                        self.screen[i][j] = 0;
                    }
                }
            }

            Instruction::LoadNormalRegister(register, value) => {
                self.v[register as usize] = value;
            }

            Instruction::LoadIndexRegister(value) => {
                self.i = value;
            }

            Instruction::SkipOnXeqV(register, value) => {
                self.skip_instruction = self.v[register as usize] == value;
            }

            Instruction::SkipOnXneqV(register, value) => {
                self.skip_instruction = self.v[register as usize] != value;
            }

            Instruction::SkipOnXeqY(x_register, y_register) => {
                self.skip_instruction = self.v[x_register as usize] == self.v[y_register as usize];
            }

            Instruction::SkipOnXneqY(x_register, y_register) => {
                self.skip_instruction = self.v[x_register as usize] != self.v[y_register as usize];
            }

            Instruction::SetXtoY(x_register, y_register) => {
                self.v[x_register as usize] = self.v[y_register as usize];
            }

            Instruction::SetXtoXorY(x_register, y_register) => {
                self.v[x_register as usize] |= self.v[y_register as usize];
            }

            Instruction::SetXtoXandY(x_register, y_register) => {
                self.v[x_register as usize] &= self.v[y_register as usize];
            }

            Instruction::SetXtoXxorY(x_register, y_register) => {
                self.v[x_register as usize] ^= self.v[y_register as usize];
            }

            Instruction::AddYtoX(x_register, y_register) => {
                let value: u16 = (self.v[x_register as usize] as u16
                    + self.v[y_register as usize] as u16)
                    & 0x00FF;

                // Set VF to 01 if a carry occurs, else 00
                self.v[0xF] = 0;
                if value > 0xFF {
                    self.v[0xF] = 1;
                }

                let value = value.try_into().expect("Couldn't convert u16 to u8");
                self.v[x_register as usize] = value;
            }

            Instruction::SubYfromX(x_register, y_register) => {
                let (value, overflowed) =
                    self.v[x_register as usize].overflowing_sub(self.v[y_register as usize]);

                // Set VF to 00 if a borrow occurs, else 01
                self.v[0xF] = 1;
                if overflowed {
                    self.v[0xF] = 0;
                }

                self.v[x_register as usize] = value;
            }

            Instruction::SetXtoYshiftRightOnce(x_register, y_register) => {
                self.v[0xF] = y_register & (1 << 0);
                self.v[x_register as usize] = self.v[y_register as usize] >> 1;
            }

            Instruction::SetXtoYshiftLeftOnce(x_register, y_register) => {
                self.v[0xF] = y_register & (1 << 7);
                self.v[x_register as usize] = self.v[y_register as usize] << 1;
            }

            Instruction::SetXtoYMinusX(x_register, y_register) => {
                let (value, overflowed) =
                    self.v[y_register as usize].overflowing_sub(self.v[x_register as usize]);

                // Set VF to 00 if a borrow occurs, else 01
                self.v[0xF] = 1;
                if overflowed {
                    self.v[0xF] = 0;
                }

                self.v[x_register as usize] = value;
            }

            Instruction::AddToNormalRegister(register, value) => {
                let value: u8 = ((self.v[register as usize] as u16 + value as u16) & 0x00FF)
                    .try_into()
                    .expect("Couldn't convert u16 to u8");
                self.v[register as usize] = value;
            }

            Instruction::JumpByRegister(address) => {
                self.pc = address + self.v[0] as usize;
            }

            Instruction::LoadRegisterWithRandom(register, value) => {
                let random_value: u8 = rand::rand() as u8;
                self.v[register as usize] = value & random_value;
            }

            Instruction::DrawSprite(x_register, y_register, num_bytes) => {
                let x_start = self.v[x_register as usize] % 64;
                let y_start = self.v[y_register as usize] % 32;
                self.v[0xF] = 0;

                assert!(num_bytes <= 0xF);
                for y in 0..num_bytes {
                    let sprite_data = self.ram[self.i as usize + y as usize];
                    for x in 0..8 {
                        if y + y_start >= 32 || x + x_start >= 64 {
                            continue;
                        }

                        // Chip8 uses big-endian
                        let mut bit_value = 0;
                        if sprite_data & (1 << (7 - x)) != 0 {
                            bit_value = 1;
                        }

                        let prev_pixel_value =
                            self.screen[(y + y_start) as usize][(x + x_start) as usize];
                        self.screen[(y + y_start) as usize][(x + x_start) as usize] ^= bit_value;

                        if prev_pixel_value == 1
                            && self.screen[(y + y_start) as usize][(x + x_start) as usize] == 0
                        {
                            self.v[0xF] = 1;
                        }
                    }
                }

                self.screen_update = true;
                self.update_screen();
            }

            Instruction::Jump(address) => {
                self.pc = address;
            }

            Instruction::Return => {
                assert!(self.stack.len() != 0);
                self.pc = self.stack.pop().unwrap();
            }

            Instruction::SubRoutine(address) => {
                self.stack.push(self.pc);
                self.pc = address as usize;
            }

            Instruction::StoreDeelayInRegister(register) => {
                self.v[register as usize] = self.deelay;
            }

            Instruction::SetDeelayFromRegister(register) => {
                self.deelay = self.v[register as usize];
            }

            Instruction::SetSoundTimerFromRegister(register) => {
                self.sound_timer = self.v[register as usize];
            }

            Instruction::AddRegisterToIndex(register) => {
                self.i += self.v[register as usize] as u16;
            }

            Instruction::FillRegisters(last_register) => {
                for register in 0..=last_register {
                    self.v[register as usize] = self.ram[self.i as usize + register as usize];
                }
                self.i = self.i + last_register as u16 + 1;
            }

            Instruction::StoreRegistersInMemmory(last_register) => {
                for register in 0..=last_register {
                    self.ram[self.i as usize + register as usize] = self.v[register as usize];
                    self.v[register as usize] = self.ram[self.i as usize + register as usize];
                }
                self.i = self.i + last_register as u16 + 1;
            }

            Instruction::StoreRegisterInBCD(register) => {
                self.ram[self.i as usize] = self.v[register as usize] / 100;
                self.ram[self.i as usize + 1] = self.v[register as usize] % 100 / 10;
                self.ram[self.i as usize + 2] = self.v[register as usize] % 10;
            }
        }
    }

    pub fn start_cycle(&mut self) {
        let opcode: u16 = (u16::from(self.ram[self.pc]) << 8) + u16::from(self.ram[self.pc + 1]);
        self.pc += 2;

        if self.skip_instruction {
            self.skip_instruction = false;
            return;
        }

        match opcode & 0xF000 {
            0x0000 => {
                if opcode & 0x0FFF == 0x00E0 {
                    self.exec(Instruction::Clear);
                } else if opcode & 0x0FFF == 0x00EE {
                    self.exec(Instruction::Return);
                } else {
                    println!("Ignored");
                }
            }

            0x1000 => {
                let address = (opcode & 0x0FFF) as usize;
                self.exec(Instruction::Jump(address));
            }

            0x2000 => {
                let address = (opcode & 0x0FFF) as usize;
                self.exec(Instruction::SubRoutine(address));
            }

            0x3000 => {
                let register: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
                let value: u8 = (opcode & 0x00FF).try_into().unwrap();
                self.exec(Instruction::SkipOnXeqV(register, value));
            }

            0x4000 => {
                let register: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
                let value: u8 = (opcode & 0x00FF).try_into().unwrap();
                self.exec(Instruction::SkipOnXneqV(register, value));
            }

            0x5000 => {
                let x_register: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
                let y_register: u8 = ((opcode & 0x00F0) >> 4).try_into().unwrap();
                let value: u8 = (opcode & 0x000F).try_into().unwrap();
                if value == 0 {
                    self.exec(Instruction::SkipOnXeqY(x_register, y_register));
                }
            }

            0x6000 => {
                let register: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
                let value: u8 = (opcode & 0x00FF).try_into().unwrap();
                self.exec(Instruction::LoadNormalRegister(register, value));
            }

            0x7000 => {
                let register: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
                let value: u8 = (opcode & 0x00FF).try_into().unwrap();
                self.exec(Instruction::AddToNormalRegister(register, value));
            }

            0x8000 => {
                let x_register: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
                let y_register: u8 = ((opcode & 0x00F0) >> 4).try_into().unwrap();
                let op: u8 = (opcode & 0x000F).try_into().unwrap();

                match op {
                    0x0 => {
                        self.exec(Instruction::SetXtoY(x_register, y_register));
                    }

                    0x1 => {
                        self.exec(Instruction::SetXtoXorY(x_register, y_register));
                    }

                    0x2 => {
                        self.exec(Instruction::SetXtoXandY(x_register, y_register));
                    }

                    0x3 => {
                        self.exec(Instruction::SetXtoXxorY(x_register, y_register));
                    }

                    0x4 => {
                        self.exec(Instruction::AddYtoX(x_register, y_register));
                    }

                    0x5 => {
                        self.exec(Instruction::SubYfromX(x_register, y_register));
                    }

                    0x6 => {
                        self.exec(Instruction::SetXtoYshiftRightOnce(x_register, y_register));
                    }

                    0x7 => {
                        self.exec(Instruction::SetXtoYMinusX(x_register, y_register));
                    }

                    0xE => {
                        self.exec(Instruction::SetXtoYshiftLeftOnce(x_register, y_register));
                    }

                    _ => (),
                }
            }

            0x9000 => {
                let x_register: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
                let y_register: u8 = ((opcode & 0x00F0) >> 4).try_into().unwrap();
                let value: u8 = (opcode & 0x000F).try_into().unwrap();
                if value == 0 {
                    self.exec(Instruction::SkipOnXneqY(x_register, y_register));
                }
            }

            0xA000 => {
                let value = opcode & 0x0FFF;
                self.exec(Instruction::LoadIndexRegister(value));
            }

            0xB000 => {
                let address = opcode & 0x0FFF;
                self.exec(Instruction::JumpByRegister(address as usize));
            }

            0xC000 => {
                let register: u8 = (opcode & 0xF000) as u8;
                let value: u8 = (opcode & 0x00FF) as u8;
                self.exec(Instruction::LoadRegisterWithRandom(register, value));
            }

            0xD000 => {
                let x_register: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
                let y_register: u8 = ((opcode & 0x00F0) >> 4).try_into().unwrap();
                let num_bytes: u8 = (opcode & 0x000F).try_into().unwrap();

                self.exec(Instruction::DrawSprite(x_register, y_register, num_bytes));
            }

            0xF000 => {
                let value: u8 = (opcode & 0x0F00) as u8;

                let sub_opcode = opcode & 0x00FF;
                match sub_opcode {
                    0x07 => {
                        self.exec(Instruction::StoreDeelayInRegister(value));
                    }

                    0x0A => {
                        todo!();
                    }

                    0x15 => {
                        self.exec(Instruction::SetDeelayFromRegister(value));
                    }

                    0x18 => {
                        self.exec(Instruction::SetSoundTimerFromRegister(value));
                    }

                    0x1E => {
                        self.exec(Instruction::AddRegisterToIndex(value));
                    }

                    0x29 => {
                        todo!("");
                    }

                    0x33 => {
                        self.exec(Instruction::StoreRegisterInBCD(value));
                    }

                    0x55 => {
                        self.exec(Instruction::StoreRegistersInMemmory(value));
                    }

                    0x65 => {
                        self.exec(Instruction::FillRegisters(value));
                    }

                    _ => eprintln!(
                        "Unsupported instruction found: {:04X} with subopcode: {:02X}",
                        opcode, sub_opcode
                    ),
                }
            }

            _ => eprintln!("Unsupported instruction found: {:04X}", opcode),
        }
    }
}
