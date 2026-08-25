use crate::cpu::Cpu;
use crate::decode::Instruction;
use crate::display::{HEIGHT, WIDTH};

pub enum Flow {
    Next,      // pc += 2
    Skip,      // pc += 4
    Jump(u16), // pc = addr
}

pub fn execute(cpu: &mut Cpu, instruction: Instruction) -> Flow {
    match instruction {
        Instruction::Cls => {
            cpu.display_buffer = [false; 64 * 32];
            cpu.should_draw = true;
            Flow::Next
        }
        Instruction::Ret => {
            cpu.sp -= 1;
            Flow::Jump(cpu.stack[cpu.sp as usize])
        }
        Instruction::JpAddr(addr) | Instruction::SysAddr(addr) => Flow::Jump(addr),
        Instruction::CallAddr(addr) => {
            cpu.stack[cpu.sp as usize] = cpu.pc + 2;
            cpu.sp += 1;
            Flow::Jump(addr)
        }
        Instruction::SeVxByte(x, byte) => {
            if cpu.v[x] == byte {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::SneVxByte(x, byte) => {
            if cpu.v[x] != byte {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::SeVxVy(x, y) => {
            if cpu.v[x] == cpu.v[y] {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::LdVxByte(x, byte) => {
            cpu.v[x] = byte;
            Flow::Next
        }
        Instruction::AddVxByte(x, byte) => {
            cpu.v[x] = cpu.v[x].wrapping_add(byte);
            Flow::Next
        }
        Instruction::LdVxVy(x, y) => {
            cpu.v[x] = cpu.v[y];
            Flow::Next
        }
        Instruction::OrVxVy(x, y) => {
            cpu.v[x] |= cpu.v[y];
            Flow::Next
        }
        Instruction::AndVxVy(x, y) => {
            cpu.v[x] &= cpu.v[y];
            Flow::Next
        }
        Instruction::XorVxVy(x, y) => {
            cpu.v[x] ^= cpu.v[y];
            Flow::Next
        }
        Instruction::AddVxVy(x, y) => {
            let (result, carry) = cpu.v[x].overflowing_add(cpu.v[y]);
            cpu.v[x] = result;
            cpu.v[0xF] = carry as u8;
            Flow::Next
        }
        Instruction::SubVxVy(x, y) => {
            let (result, borrow) = cpu.v[x].overflowing_sub(cpu.v[y]);
            cpu.v[x] = result;
            cpu.v[0xF] = !borrow as u8;
            Flow::Next
        }
        Instruction::ShrVx(x) => {
            cpu.v[0xF] = cpu.v[x] & 1; // Store least significant bit
            cpu.v[x] >>= 1;
            Flow::Next
        }
        Instruction::SubnVxVy(x, y) => {
            let (result, borrow) = cpu.v[y].overflowing_sub(cpu.v[x]);
            cpu.v[x] = result;
            cpu.v[0xF] = !borrow as u8;
            Flow::Next
        }
        Instruction::ShlVx(x) => {
            cpu.v[0xF] = (cpu.v[x] & 0b1000_0000) >> 7; // Store most significant bit
            cpu.v[x] <<= 1;
            Flow::Next
        }
        Instruction::SneVxVy(x, y) => {
            if cpu.v[x] != cpu.v[y] {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::LdIAddr(addr) => {
            cpu.index = addr;
            Flow::Next
        }
        Instruction::JpV0Addr(addr) => Flow::Jump(addr + cpu.v[0] as u16),
        Instruction::RndVxByte(x, byte) => {
            cpu.v[x] = rand::random::<u8>() & byte;
            Flow::Next
        }
        Instruction::DrwVxVyNibble(x, y, nibble) => {
            let sprite_height = nibble;
            let start_col = cpu.v[x] as usize;
            let start_row = cpu.v[y] as usize;
            cpu.v[0xF] = 0; // Reset collision flag

            for sprite_row in 0..sprite_height {
                let y = (start_row + sprite_row as usize) % HEIGHT;
                let byte = cpu.memory[cpu.index as usize + sprite_row as usize];
                for sprite_col in 0..8 {
                    let x = (start_col + sprite_col) % WIDTH;
                    let pixel = ((byte >> (7 - sprite_col)) & 1) == 1;
                    let buffer_index = y * WIDTH + x;
                    if pixel && cpu.display_buffer[buffer_index] {
                        cpu.v[0xF] = 1; // Collision detected
                    }
                    cpu.display_buffer[buffer_index] ^= pixel;
                }
            }

            cpu.should_draw = true;
            Flow::Next
        }
        Instruction::SkpVx(x) => {
            if cpu.key_inputs[cpu.v[x] as usize] {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::SknpVx(x) => {
            if !cpu.key_inputs[cpu.v[x] as usize] {
                Flow::Skip
            } else {
                Flow::Next
            }
        }
        Instruction::LdVxDt(x) => {
            cpu.v[x] = cpu.delay_timer;
            Flow::Next
        }
        Instruction::LdVxK(x) => {
            for (i, &key_pressed) in cpu.key_inputs.iter().enumerate() {
                if key_pressed {
                    cpu.v[x] = i as u8;
                    return Flow::Next;
                }
            }
            Flow::Jump(cpu.pc)
        }
        Instruction::LdDtVx(x) => {
            cpu.delay_timer = cpu.v[x];
            Flow::Next
        }
        Instruction::LdStVx(x) => {
            cpu.sound_timer = cpu.v[x];
            Flow::Next
        }
        Instruction::AddIVx(x) => {
            cpu.index = cpu.index.wrapping_add(cpu.v[x] as u16);
            Flow::Next
        }
        Instruction::LdFVx(x) => {
            cpu.index = cpu.v[x] as u16 * 5;
            Flow::Next
        }
        Instruction::LdBVx(x) => {
            cpu.memory[cpu.index as usize] = cpu.v[x] / 100;
            cpu.memory[cpu.index as usize + 1] = (cpu.v[x] / 10) % 10;
            cpu.memory[cpu.index as usize + 2] = cpu.v[x] % 10;
            Flow::Next
        }
        Instruction::LdIVx(x) => {
            for i in 0..=x {
                cpu.memory[cpu.index as usize + i] = cpu.v[i];
            }
            Flow::Next
        }
        Instruction::LdVxI(x) => {
            for i in 0..=x {
                cpu.v[i] = cpu.memory[cpu.index as usize + i];
            }
            Flow::Next
        }
        Instruction::Unknown(opcode) => {
            panic!("Encountered unknown opcode: {:?}", opcode);
        }
    }
}
