use crate::chip8::Chip8;
use crate::decode::Instruction;
use crate::display::{HEIGHT, WIDTH};

pub enum Flow {
    Next,      // pc += 2
    Skip,      // pc += 4
    Jump(u16), // pc = addr
}

pub fn execute(chip8: &mut Chip8, instruction: Instruction) -> Flow {
    match instruction {
        Instruction::Cls => {
            chip8.display_buffer = [false; 64 * 32];
            chip8.should_draw = true;
            Flow::Next
        }
        Instruction::Ret => {
            chip8.cpu.sp -= 1;
            Flow::Jump(chip8.cpu.stack[chip8.cpu.sp as usize])
        }
        Instruction::JpAddr(addr) | Instruction::SysAddr(addr) => Flow::Jump(addr),
        Instruction::CallAddr(addr) => {
            chip8.cpu.stack[chip8.cpu.sp as usize] = chip8.cpu.pc + 2;
            chip8.cpu.sp += 1;
            Flow::Jump(addr)
        }
        Instruction::SeVxByte(x, byte) => {
            if chip8.cpu.v[x] == byte {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::SneVxByte(x, byte) => {
            if chip8.cpu.v[x] != byte {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::SeVxVy(x, y) => {
            if chip8.cpu.v[x] == chip8.cpu.v[y] {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::LdVxByte(x, byte) => {
            chip8.cpu.v[x] = byte;
            Flow::Next
        }
        Instruction::AddVxByte(x, byte) => {
            chip8.cpu.v[x] = chip8.cpu.v[x].wrapping_add(byte);
            Flow::Next
        }
        Instruction::LdVxVy(x, y) => {
            chip8.cpu.v[x] = chip8.cpu.v[y];
            Flow::Next
        }
        Instruction::OrVxVy(x, y) => {
            chip8.cpu.v[x] |= chip8.cpu.v[y];
            Flow::Next
        }
        Instruction::AndVxVy(x, y) => {
            chip8.cpu.v[x] &= chip8.cpu.v[y];
            Flow::Next
        }
        Instruction::XorVxVy(x, y) => {
            chip8.cpu.v[x] ^= chip8.cpu.v[y];
            Flow::Next
        }
        Instruction::AddVxVy(x, y) => {
            let (result, carry) = chip8.cpu.v[x].overflowing_add(chip8.cpu.v[y]);
            chip8.cpu.v[x] = result;
            chip8.cpu.v[0xF] = carry as u8;
            Flow::Next
        }
        Instruction::SubVxVy(x, y) => {
            let (result, borrow) = chip8.cpu.v[x].overflowing_sub(chip8.cpu.v[y]);
            chip8.cpu.v[x] = result;
            chip8.cpu.v[0xF] = !borrow as u8;
            Flow::Next
        }
        Instruction::ShrVx(x) => {
            chip8.cpu.v[0xF] = chip8.cpu.v[x] & 1; // Store least significant bit
            chip8.cpu.v[x] >>= 1;
            Flow::Next
        }
        Instruction::SubnVxVy(x, y) => {
            let (result, borrow) = chip8.cpu.v[y].overflowing_sub(chip8.cpu.v[x]);
            chip8.cpu.v[x] = result;
            chip8.cpu.v[0xF] = !borrow as u8;
            Flow::Next
        }
        Instruction::ShlVx(x) => {
            chip8.cpu.v[0xF] = (chip8.cpu.v[x] & 0b1000_0000) >> 7; // Store most significant bit
            chip8.cpu.v[x] <<= 1;
            Flow::Next
        }
        Instruction::SneVxVy(x, y) => {
            if chip8.cpu.v[x] != chip8.cpu.v[y] {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::LdIAddr(addr) => {
            chip8.cpu.index = addr;
            Flow::Next
        }
        Instruction::JpV0Addr(addr) => Flow::Jump(addr + chip8.cpu.v[0] as u16),
        Instruction::RndVxByte(x, byte) => {
            chip8.cpu.v[x] = rand::random::<u8>() & byte;
            Flow::Next
        }
        Instruction::DrwVxVyNibble(x, y, nibble) => {
            let sprite_height = nibble;
            let start_col = chip8.cpu.v[x] as usize;
            let start_row = chip8.cpu.v[y] as usize;
            chip8.cpu.v[0xF] = 0; // Reset collision flag

            for sprite_row in 0..sprite_height {
                let y = (start_row + sprite_row as usize) % HEIGHT;
                let byte = chip8.memory.0[chip8.cpu.index as usize + sprite_row as usize];
                for sprite_col in 0..8 {
                    let x = (start_col + sprite_col) % WIDTH;
                    let pixel = ((byte >> (7 - sprite_col)) & 1) == 1;
                    let buffer_index = y * WIDTH + x;
                    if pixel && chip8.display_buffer[buffer_index] {
                        chip8.cpu.v[0xF] = 1; // Collision detected
                    }
                    chip8.display_buffer[buffer_index] ^= pixel;
                }
            }

            chip8.should_draw = true;
            Flow::Next
        }
        Instruction::SkpVx(x) => {
            if chip8.key_inputs[chip8.cpu.v[x] as usize] {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::SknpVx(x) => {
            if !chip8.key_inputs[chip8.cpu.v[x] as usize] {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::LdVxDt(x) => {
            chip8.cpu.v[x] = chip8.cpu.delay_timer;
            Flow::Next
        }
        Instruction::LdVxK(x) => {
            for (i, &key_pressed) in chip8.key_inputs.iter().enumerate() {
                if key_pressed {
                    chip8.cpu.v[x] = i as u8;
                    return Flow::Next;
                }
            }
            Flow::Jump(chip8.cpu.pc)
        }
        Instruction::LdDtVx(x) => {
            chip8.cpu.delay_timer = chip8.cpu.v[x];
            Flow::Next
        }
        Instruction::LdStVx(x) => {
            chip8.cpu.sound_timer = chip8.cpu.v[x];
            Flow::Next
        }
        Instruction::AddIVx(x) => {
            chip8.cpu.index = chip8.cpu.index.wrapping_add(chip8.cpu.v[x] as u16);
            Flow::Next
        }
        Instruction::LdFVx(x) => {
            chip8.cpu.index = chip8.cpu.v[x] as u16 * 5;
            Flow::Next
        }
        Instruction::LdBVx(x) => {
            chip8.memory.0[chip8.cpu.index as usize] = chip8.cpu.v[x] / 100;
            chip8.memory.0[chip8.cpu.index as usize + 1] = (chip8.cpu.v[x] / 10) % 10;
            chip8.memory.0[chip8.cpu.index as usize + 2] = chip8.cpu.v[x] % 10;
            Flow::Next
        }
        Instruction::LdIVx(x) => {
            for i in 0..=x {
                chip8.memory.0[chip8.cpu.index as usize + i] = chip8.cpu.v[i];
            }
            Flow::Next
        }
        Instruction::LdVxI(x) => {
            for i in 0..=x {
                chip8.cpu.v[i] = chip8.memory.0[chip8.cpu.index as usize + i];
            }
            Flow::Next
        }
        Instruction::Unknown(opcode) => {
            panic!("Encountered unknown opcode: {:?}", opcode);
        }
    }
}
