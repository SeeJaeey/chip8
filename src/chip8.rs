use rodio::{Player, Source};

use crate::cpu::Cpu;
use crate::decode::decode;
use crate::execute::{Flow, execute};
use crate::memory::Memory;

pub struct Chip8 {
    pub cpu: Cpu,
    pub memory: Memory,
    pub key_inputs: [bool; 16],
    pub display_buffer: [bool; 64 * 32], // Display buffer for a 64x32 monochrome display
    audio_player: Player,                // Audio player for sound output
    pub should_draw: bool,
}

impl Chip8 {
    pub fn new(audio_player: Player) -> Self {
        let mut chip8 = Chip8 {
            cpu: Cpu::new(),
            memory: Memory::new(),
            key_inputs: [false; 16],
            display_buffer: [false; 64 * 32],
            audio_player,
            should_draw: false,
        };
        chip8.memory.load_font();
        chip8
    }

    pub fn cycle(&mut self) {
        // Fetch instruction
        let high = self.memory.0[self.cpu.pc as usize] as u16;
        let low = self.memory.0[self.cpu.pc as usize + 1] as u16;
        let opcode = (high << 8) | low;
        let instruction = decode(opcode);

        // Process instruction
        let flow = execute(self, instruction);
        match flow {
            Flow::Next => self.cpu.pc += 2,
            Flow::Skip => self.cpu.pc += 4,
            Flow::Jump(addr) => self.cpu.pc = addr,
        }
    }

    pub fn decrement_timers(&mut self) {
        if self.cpu.delay_timer > 0 {
            self.cpu.delay_timer -= 1;
        }
        if self.cpu.sound_timer > 0 {
            if self.audio_player.empty() {
                let beep = rodio::source::SineWave::new(440.0).amplify(0.2);
                self.audio_player.append(beep);
            }
            self.cpu.sound_timer -= 1;
            if self.cpu.sound_timer == 0 {
                self.audio_player.stop();
            }
        }
    }
}
