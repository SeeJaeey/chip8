use minifb::{Key, Window, WindowOptions};
use rodio::{DeviceSinkBuilder, Player};
use std::env;

mod chip8;
mod cpu;
mod decode;
mod display;
mod execute;
mod memory;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_rom>", args[0]);
        return;
    }

    let audio_stream =
        DeviceSinkBuilder::open_default_sink().expect("Failed to open default audio sink");
    let audio_player = Player::connect_new(audio_stream.mixer());

    let mut chip8 = chip8::Chip8::new(audio_player);

    chip8.memory.load_rom(&args[1]);

    let mut window = Window::new(
        "Chip8",
        display::WIDTH * display::SCALE,
        display::HEIGHT * display::SCALE,
        WindowOptions::default(),
    )
    .unwrap();
    window.set_target_fps(60);

    let mut framebuffer =
        vec![0u32; display::WIDTH * display::SCALE * display::HEIGHT * display::SCALE];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for _ in 0..10 {
            // Run multiple cycles per frame for speed -> 60Hz * 10 = 600Hz
            chip8.cycle();
        }

        chip8.decrement_timers();

        if chip8.should_draw {
            display::draw(&chip8.display_buffer, &mut framebuffer);
            window
                .update_with_buffer(
                    &framebuffer,
                    display::WIDTH * display::SCALE,
                    display::HEIGHT * display::SCALE,
                )
                .unwrap();
            chip8.should_draw = false;
        } else {
            window.update(); // Poll events
        }

        display::update_keys(&window, &mut chip8.key_inputs);
    }
}
