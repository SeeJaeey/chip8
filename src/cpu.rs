pub struct Cpu {
    // CPU registers
    pub v: [u8; 16],
    pub index: u16, // For memory addresses
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub sp: u8,
    pub pc: u16,
    pub stack: [u16; 16], // Chip-8 allows for up to 16 levels of nested subroutines
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            v: [0; 16],
            index: 0,
            delay_timer: 0,
            sound_timer: 0,
            sp: 0,
            pc: 0x200, // Program counter starts at 0x200
            stack: [0; 16],
        }
    }
}
