use rodio::{DeviceSinkBuilder, Player};
use std::{env, thread, time};

mod cpu;

fn main() {
    // Test sound functionality
    let audio_stream =
        DeviceSinkBuilder::open_default_sink().expect("Failed to open default audio sink");
    let audio_player = Player::connect_new(audio_stream.mixer());

    let mut chip8 = cpu::Cpu::new(audio_player);

    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     eprintln!("Usage: {} <path_to_rom>", args[0]);
    //     return;
    // }
    // chip8.load_rom(&args[1]);

    chip8.sound_timer = 180;
    while !chip8.exit {
        println!("Sound Timer: {}", chip8.sound_timer);

        chip8.cycle();
        thread::sleep(time::Duration::from_millis(16));
    }
}
