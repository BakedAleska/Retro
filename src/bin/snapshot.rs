//! Headless dev tool: runs a ROM for a fixed number of cycles with no
//! display/audio and dumps the resulting 64x32 display buffer to a PNG.
//!
//! Usage:
//!   snapshot <rom> <cycles> <out.png> [--platform=chip8|chip48] [--tap=cycle:key[:duration]]... [--scale=N] [--trace-from=N]
//!
//! `--tap=cycle:key[:duration]` presses the given hex key (0-F) starting at
//! the given cycle index for `duration` ticks (default 1), then releases it
//! — useful for scripting past a test ROM's "press a key to continue" menu
//! or a key-scan loop that needs the key held across several ticks.
//!
//! `--scale=N` nearest-neighbor upscales the output PNG (default 8), since a
//! 64x32 image is too small to read comfortably at 1:1.

use std::env;
use std::fs;

use retro::chip8::{Chip8, Platform, HEIGHT, WIDTH};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "usage: snapshot <rom> <cycles> <out.png> [--platform=chip8|chip48] [--tap=cycle:key]..."
        );
        std::process::exit(1);
    }

    let rom_path = &args[1];
    let cycles: u32 = args[2].parse().expect("cycles must be a number");
    let out_path = &args[3];

    let mut platform = Platform::Chip48;
    // (start_cycle, end_cycle_exclusive, key)
    let mut taps: Vec<(u32, u32, u8)> = Vec::new();
    let mut scale: u32 = 8;
    let mut trace_from: Option<u32> = None;

    for arg in &args[4..] {
        if let Some(value) = arg.strip_prefix("--platform=") {
            platform = match value {
                "chip8" => Platform::Chip8,
                "chip48" => Platform::Chip48,
                other => panic!("unknown platform: {other}"),
            };
        } else if let Some(value) = arg.strip_prefix("--tap=") {
            let mut parts = value.split(':');
            let cycle: u32 = parts
                .next()
                .expect("--tap expects cycle:key[:duration]")
                .parse()
                .expect("tap cycle must be a number");
            let key = u8::from_str_radix(
                parts.next().expect("--tap expects cycle:key[:duration]"),
                16,
            )
            .expect("tap key must be hex 0-F");
            let duration: u32 = parts.next().map_or(1, |d| d.parse().expect("tap duration must be a number"));
            taps.push((cycle, cycle + duration, key));
        } else if let Some(value) = arg.strip_prefix("--scale=") {
            scale = value.parse().expect("scale must be a number");
        } else if let Some(value) = arg.strip_prefix("--trace-from=") {
            trace_from = Some(value.parse().expect("trace-from must be a number"));
        }
    }

    let rom = fs::read(rom_path).expect("Failed to read ROM");
    let mut chip8 = Chip8::new(platform.quirks());
    chip8.load(&rom);

    for cycle in 0..cycles {
        for &(start, end, key) in &taps {
            chip8.set_key(key as usize, cycle >= start && cycle < end);
        }
        let opcode = chip8.tick();
        if trace_from.is_some_and(|from| cycle >= from) {
            let state = chip8.debug_state();
            eprintln!(
                "{cycle:>6} opcode={opcode:#06X} pc={:#05X} i={:#05X} sp={} v={:02X?}",
                state.pc, state.i, state.sp, state.registers
            );
        }
    }

    let display = chip8.display();
    let mut img = image::GrayImage::new(WIDTH as u32 * scale, HEIGHT as u32 * scale);
    for (i, &on) in display.iter().enumerate() {
        let x = (i % WIDTH) as u32;
        let y = (i / WIDTH) as u32;
        let value = image::Luma([if on { 255 } else { 0 }]);
        for dy in 0..scale {
            for dx in 0..scale {
                img.put_pixel(x * scale + dx, y * scale + dy, value);
            }
        }
    }

    img.save(out_path).expect("Failed to save PNG");
    println!("Wrote {out_path} after {cycles} cycles ({platform:?})");
}
