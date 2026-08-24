use minifb::{Key, Window};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
pub const SCALE: usize = 32;

pub fn draw(display_buffer: &[bool], framebuffer: &mut [u32]) {
    const ON: u32 = 0xFFFFFF;  // White
    const OFF: u32 = 0x000000; // Black

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let pixel_on = display_buffer[y * WIDTH + x];
            let color = if pixel_on { ON } else { OFF };

            // Write every chip8 pixel as a SCALE x SCALE block into the framebuffer
            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    let fx = x * SCALE + dx;
                    let fy = y * SCALE + dy;
                    framebuffer[fy * WIDTH * SCALE + fx] = color;
                }
            }
        }
    }
}

pub fn update_keys(window: &Window, key_inputs: &mut [bool; 16]) {
    let keymap = [
        (Key::Key1, 0x1), (Key::Key2, 0x2), (Key::Key3, 0x3), (Key::C, 0xC),
        (Key::Key4, 0x4), (Key::Key5, 0x5), (Key::Key6, 0x6), (Key::D, 0xD),
        (Key::Key7, 0x7), (Key::Key8, 0x8), (Key::Key9, 0x9), (Key::E, 0xE),
        (Key::A, 0xA), (Key::Key0, 0x0), (Key::B, 0xB), (Key::F, 0xF),
    ];
    for (key, chip8_key) in keymap {
        key_inputs[chip8_key] = window.is_key_down(key);
    }
}
