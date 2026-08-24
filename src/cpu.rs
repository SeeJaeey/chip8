use rodio::{Player, Source};

pub struct Cpu {
    // CPU registers
    pub v: [u8; 16],
    pub index: u16, // For memory addresses
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub sp: u8,
    pub pc: u16,

    // Memory
    pub memory: [u8; 4096], // 4KB of RAM (0x000
    pub stack: [u16; 16],   // Chip-8 allows for up to 16 levels of nested subroutines

    // Other CPU state variables
    pub key_inputs: [bool; 16], // Array to hold the state of key inputs (16 keys)
    pub display_buffer: [bool; 64 * 32], // Display buffer for a 64x32 monochrome display
    pub opcode: u16,
    pub audio_player: Player, // Audio player for sound output
    pub exit: bool, // Flag to indicate when to exit the main loop
    pub should_draw: bool, // Flag to indicate when the display needs to be redrawn
}

impl Cpu {
    pub fn new(audio_player: Player) -> Self {
        let mut cpu = Cpu {
            v: [0; 16],
            index: 0,
            delay_timer: 0,
            sound_timer: 0,
            sp: 0,
            pc: 0x200, // Program counter starts at 0x200
            memory: [0; 4096],
            stack: [0; 16],
            key_inputs: [false; 16],
            display_buffer: [false; 64 * 32],
            opcode: 0,
            audio_player,
            exit: false,
            should_draw: false,
        };
        //cpu.load_font(); // TODO: enable later
        cpu
    }

    // Additional methods for CPU operations
    fn load_font(&mut self) {
        let fontset: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F

        ];
        for i in 0..80 {
            self.memory[i] = fontset[i];
        }
    }

    pub fn load_rom(&mut self, rom_path: &str) {
        println!("Loading {}", rom_path);
        let binary = std::fs::read(rom_path).expect("Failed to read ROM file");
        for (i, &byte) in binary.iter().enumerate() {
            self.memory[i + 0x200] = byte;
        }
    }

    pub fn cycle(&mut self) {
        // Fetch instruction
        let high = self.memory[self.pc as usize] as u16;
        let low = self.memory[self.pc as usize + 1] as u16;
        self.opcode = (high << 8) | low;

        // Process instruction
        self.process_opcode();

        self.pc += 2;

        // Decrement timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            if self.sound_timer != 0 {
                if self.audio_player.empty() {
                    let beep = rodio::source::SineWave::new(440.0).amplify(0.2);
                    self.audio_player.append(beep);
                }
            } else {
                self.audio_player.stop();
            }
        }
    }

    pub fn process_opcode(&mut self) {
        // Documentation: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#00E0
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let y = (self.opcode & 0x00F0 >> 4) as usize;
        let kk = (self.opcode & 0x00FF) as u8;

        match self.opcode {
            0x00E0 => {
                // 00E0 - CLS
                // Clear the display
                self.display_buffer = [false; 64 * 32];
                self.should_draw = true;
            }
            0x00EE => {
                // 00EE - RET
                // Return from a subroutine
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            }
            opcode if opcode & 0xF000 == 0x0000 => {
                // 0nnn - SYS addr
                // Jump to a machine code routine at nnn
                self.pc = opcode & 0x0FFF;
            }
            opcode if opcode & 0xF000 == 0x1000 => {
                // 1nnn - JP addr
                // Jump to location at nnn
                self.pc = opcode & 0x0FFF;
            }
            opcode if opcode & 0xF000 == 0x2000 => {
                // 2nnn - CALL addr
                // Call subroutine at nnn
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = opcode & 0x0FFF;
            }
            opcode if opcode & 0xF000 == 0x3000 => {
                // 3xkk - SE Vx, byte
                // Skip next instruction if Vx = kk
                if self.v[x] == kk {
                    self.pc += 2;
                }
            }
            opcode if opcode & 0xF000 == 0x4000 => {
                // 4xkk - SNE Vx, byte
                // Skip next instruction if Vx != kk
                if self.v[x] != kk {
                    self.pc += 2;
                }
            }
            opcode if opcode & 0xF000 == 0x5000 => {
                // 5xy0 - SE Vx, Vy
                // Skip next instruction if Vx = Vy
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            opcode if opcode & 0xF000 == 0x6000 => {
                // 6xkk - LD Vx, byte
                // Set Vx = kk
                self.v[x] = kk;
            }
            opcode if opcode & 0xF000 == 0x7000 => {
                // 7xkk - ADD Vx, byte
                // Set Vx = Vx + kk
                self.v[x] += kk;
            }
            opcode if opcode & 0xF000 == 0x8000 => {
                match opcode & 0x000F {
                    0x0000 => {
                        // 8xy0 - LD Vx, Vy
                        // Set Vx = Vy
                        self.v[x] = self.v[y];
                    }
                    0x0001 => {
                        // 8xy1 - OR Vx, Vy
                        // Set Vx = Vx OR Vy
                        self.v[x] |= self.v[y];
                    }
                    0x0002 => {
                        // 8xy2 - AND Vx, Vy
                        // Set Vx = Vx AND Vy
                        self.v[x] &= self.v[y];
                    }
                    0x0003 => {
                        // 8xy3 - XOR Vx, Vy
                        // Set Vx = Vx XOR Vy
                        self.v[x] ^= self.v[y];
                    }
                    0x0004 => {
                        // 8xy4 - ADD Vx, Vy
                        // Set Vx = Vx + Vy, set VF = carry
                        if self.v[x] as u16 + self.v[y] as u16 > 0b1111_1111 {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[x] = self.v[x].wrapping_add(self.v[y]);
                    }
                    0x0005 => {
                        // 8xy5 - SUB Vx, Vy
                        // Set Vx = Vx - Vy, set VF = NOT borrow
                        if self.v[x] > self.v[y] {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                    }
                    0x0006 => {
                        // 8xy6 - SHR Vx {, Vy}
                        // Set Vx = Vx SHR 1
                        self.v[0xF] = self.v[x] & 0b0000_0001; // Store least significant bit
                        self.v[x] >>= 1;
                    }
                    0x0007 => {
                        // 8xy7 - SUBN Vx, Vy
                        // Set Vx = Vy - Vx, set VF = NOT borrow
                        if self.v[y] > self.v[x] {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                    }
                    0x000E => {
                        // 8xyE - SHL Vx {, Vy}
                        // Set Vx = Vx SHL 1
                        self.v[0xF] = self.v[x] & 0b1000_0000; // Store most significant bit
                        self.v[x] <<= 1;
                    }
                    _ => {
                        println!("Unknown opcode: {:04X}", self.opcode);
                    }
                }
            }
            opcode if opcode & 0xF000 == 0x9000 => {
                // 9xy0 - SNE Vx, Vy
                // Skip next instruction if Vx != Vy
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            opcode if opcode & 0xF000 == 0xA000 => {
                // Annn - LD I, addr
                // Set I = nnn
                self.index = opcode & 0x0FFF;
            }
            opcode if opcode & 0xF000 == 0xB000 => {
                // Bnnn - JP V0, addr
                // Jump to location nnn + V0
                self.pc = opcode & 0x0FFF + self.v[0] as u16; // TODO: overflow possible?
            }
            opcode if opcode & 0xF000 == 0xC000 => {
                // Cxkk - RND Vx, byte
                // Set Vx = random byte AND kk
                self.v[x] = rand::random::<u8>() & kk;
            }
            opcode if opcode & 0xF000 == 0xD000 => {
                // Dxyn - DRW Vx, Vy, nibble
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
                // TODO: Implement
                
                self.should_draw = true;
            }
            opcode if opcode & 0xF000 == 0xE000 => {
                match opcode & 0x00FF {
                    0x009E => {
                        // Ex9E - SKP Vx
                        // Skip next instruction if key with the value of Vx is pressed
                        if self.key_inputs[self.v[x] as usize] {
                            self.pc += 2;
                        }
                    }
                    0x00A1 => {
                        // ExA1 - SKNP Vx
                        // Skip next instruction if key with the value of Vx is not pressed
                        if !self.key_inputs[self.v[x] as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        println!("Unknown opcode: {:04X}", self.opcode);
                    }
                }
            }
            opcode if opcode & 0xF000 == 0xF000 => {
                match opcode & 0x00FF {
                    0x0007 => {
                        // Fx07 - LD Vx, DT
                        // Set Vx = delay timer value
                        self.v[x] = self.delay_timer;
                    }
                    0x000A => {
                        // Fx0A - LD Vx, K
                        // Wait for a key press, store the value of the key in Vx
                        // TODO: Implement
                    }
                    0x0015 => {
                        // Fx15 - LD DT, Vx
                        // Set delay timer = Vx
                        self.delay_timer = self.v[x];
                    }
                    0x0018 => {
                        // Fx18 - LD ST, Vx
                        // Set sound timer = Vx
                        self.sound_timer = self.v[x];
                    }
                    0x001E => {
                        // Fx1E - ADD I, Vx
                        // Set I = I + Vx
                        self.index += self.v[x] as u16; // TODO: overflow possible?
                    }
                    0x0029 => {
                        // Fx29 - LD F, Vx
                        // Set I = location of sprite for digit Vx
                        self.index = (self.v[x] as u16 * 5) & 0xFFF;
                    }
                    0x0033 => {
                        // Fx33 - LD B, Vx
                        // Store BCD representation of Vx in memory locations I, I+1, and I+2
                        self.memory[self.index as usize] = self.v[x] / 100;
                        self.memory[self.index as usize + 1] = (self.v[x] / 10) % 10;
                        self.memory[self.index as usize + 2] = self.v[x] % 10;
                    }
                    0x0055 => {
                        // Fx55 - LD [I], Vx
                        // Store registers V0 through Vx in memory starting at location I
                        for i in 0..=x {
                            self.memory[self.index as usize + i] = self.v[i];
                        }
                    }
                    0x0065 => {
                        // Fx65 - LD Vx, [I]
                        // Read registers V0 through Vx from memory starting at location I
                        for i in 0..=x {
                            self.v[i] = self.memory[self.index as usize + i];
                        }
                    }
                    _ => {
                        println!("Unknown opcode: {:04X}", self.opcode);
                    }
                }
            }
            _ => {
                println!("Unknown opcode: {:04X}", self.opcode);
            }
        }
    }
}
