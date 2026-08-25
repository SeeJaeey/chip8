use rodio::{Player, Source};

use crate::decode::decode;
use crate::execute::{Flow, execute};

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
    pub audio_player: Player,   // Audio player for sound output
    pub should_draw: bool,      // Flag to indicate when the display needs to be redrawn
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
            audio_player,
            should_draw: false,
        };
        cpu.load_font();
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
        let opcode = (high << 8) | low;
        let instruction = decode(opcode);

        // Process instruction
        let flow = execute(self, instruction);
        match flow {
            Flow::Next => self.pc += 2,
            Flow::Skip => self.pc += 4,
            Flow::Jump(addr) => self.pc = addr,
        }
    }

    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.audio_player.empty() {
                let beep = rodio::source::SineWave::new(440.0).amplify(0.2);
                self.audio_player.append(beep);
            }
            self.sound_timer -= 1;
            if self.sound_timer == 0 {
                self.audio_player.stop();
            }
        }
    }
}
