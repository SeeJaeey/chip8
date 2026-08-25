# CHIP-8 Interpreter

A CHIP-8 interpreter (emulator) written in Rust, featuring a real-time display via [`minifb`](https://crates.io/crates/minifb) and sound output via [`rodio`](https://crates.io/crates/rodio).

<!-- TODO: add a screenshot or GIF of the interpreter running a ROM, e.g. ![screenshot](docs/screenshot.png) -->

## About

CHIP-8 is an interpreted programming language from the 1970s, originally designed to make it easier to write games for 8-bit microcomputers. It's my first project for learning emulator development, since its instruction set is small (35 opcodes) but touches on the core concepts of every emulator: a fetch-decode-execute cycle, memory-mapped state, timers, and I/O.

This project implements:

- A full CPU core (registers, memory, stack, program counter, timers)
- All standard CHIP-8 opcodes
- A 64×32 monochrome display, rendered scaled up in a minifb window
- Keyboard input mapped to the original 16-key hex keypad
- A simple beep sound while the sound timer is active

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain, via `rustup`)
- A CHIP-8 ROM file to run (see [Finding ROMs](#finding-roms) below)

### Build

```bash
git clone https://github.com/SeeJaeey/chip8.git
cd chip8
cargo build --release
```

### Run

```bash
cargo run --release -- path/to/rom.ch8
```

## Controls

The original CHIP-8 keypad had 16 keys, laid out like this:

```
1 2 3 C
4 5 6 D
7 8 9 E
A 0 B F
```

This is mapped onto the following keys on a standard keyboard:

| CHIP-8 | Keyboard | CHIP-8 | Keyboard | CHIP-8 | Keyboard | CHIP-8 | Keyboard |
|:------:|:--------:|:------:|:--------:|:------:|:--------:|:------:|:--------:|
|   1    |    1     |   2    |    2     |   3    |    3     |   C    |    4     |
|   4    |    Q     |   5    |    W     |   6    |    E     |   D    |    R     |
|   7    |    A     |   8    |    S     |   9    |    D     |   E    |    F     |
|   A    |    Z     |   0    |    X     |   B    |    C     |   F    |    V     |

Press `Esc` to quit.

## Finding ROMs

This repository contains some example ROMs under /roms. Public-domain CHIP-8 ROMs and test suites you can use for development and testing include:

- [chip8-test-rom by corax89](https://github.com/corax89/chip8-test-rom) — checks opcode correctness
- [Timendus' chip8-test-suite](https://github.com/Timendus/chip8-test-suite) — more extensive test suite with visual output
- [Chip-8 Program Pack](https://github.com/kripod/chip8-roms) — collection of classic games (Pong, Tetris, Space Invaders, etc.)

## Project Structure

```
.
├── roms/             # Collection of different roms
├── src/
│   ├── cpu.rs        # CPU state
│   ├── decode.rs
│   ├── display.rs    # Framebuffer rendering & key mapping
│   ├── execute.rs
│   └── main.rs       # Entry point: window setup, main loop
└── Cargo.toml
```

## Acknowledgments

- [Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM) — the primary opcode reference used during implementation
- The CHIP-8 community for keeping decades-old test ROMs and documentation alive

## License

This project is licensed under the [MIT License](LICENSE).
