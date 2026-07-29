use std::time::{Duration, Instant};
use std::{fs, thread};
use sdl2;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::keyboard::Scancode;

mod chip8;
use chip8::Chip8;

const CYCLES_PER_FRAME: u32 = 700 / 60;
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

const WIDTH: i32 = 64;
const HEIGHT: i32 = 32;

const SCALE: i32 = 16;

const KEYS: [Scancode; 16] = [
    Scancode::X,    // 0x0
    Scancode::Num1, // 0x1
    Scancode::Num2, // 0x2
    Scancode::Num3, // 0x3
    Scancode::Q,    // 0x4
    Scancode::W,    // 0x5
    Scancode::E,    // 0x6
    Scancode::A,    // 0x7
    Scancode::S,    // 0x8
    Scancode::D,    // 0x9
    Scancode::Z,    // 0xA
    Scancode::C,    // 0xB
    Scancode::Num4, // 0xC
    Scancode::R,    // 0xD
    Scancode::F,    // 0xE
    Scancode::V,    // 0xF
];

fn main() {
    let mut chip8 = Chip8::new();

    let sdl_context = sdl2::init().expect("Failed to build sdl_context.");
    let sdl_video = sdl_context.video().expect("Failed to build sdl_video");
    let sdl_window = sdl_video.window("CHIP-8", (WIDTH * SCALE) as u32, (HEIGHT * SCALE) as u32).position_centered().build().expect("Failed to build sdl_window");
    let mut sdl_canvas = sdl_window.into_canvas().build().expect("Failed to build sdl_canvas");
    let mut sdl_event_pump = sdl_context.event_pump().expect("Failed to build sdl_event_pump");

    let rom = fs::read("./roms/Flags.ch8").expect("Failed to load rom file.");
    chip8.load(rom);

    fn match_scancode(scancode: Scancode) -> Option<u8> {
        match scancode {
            Scancode::Num1 => Some(0x1),
            Scancode::Num2 => Some(0x2),
            Scancode::Num3 => Some(0x3),
            Scancode::Num4 => Some(0xC),
            Scancode::Q => Some(0x4),
            Scancode::W => Some(0x5),
            Scancode::E => Some(0x6),
            Scancode::R => Some(0xD),
            Scancode::A => Some(0x7),
            Scancode::S => Some(0x8),
            Scancode::D => Some(0x9),
            Scancode::F => Some(0xE),
            Scancode::Z => Some(0xA),
            Scancode::X => Some(0x0),
            Scancode::C => Some(0xB),
            Scancode::V => Some(0xF),
            _ => None
        }
    }

    loop {
        let frame_start = Instant::now();

        for _ in 0..CYCLES_PER_FRAME {
            chip8.tick();
        }

        chip8.decrement();

        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            thread::sleep(FRAME_DURATION - elapsed);
        }

        for event in sdl_event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    return;
                },
                _ => {
                    //println!("No event match found for {:?}", event);
                }
            }
        }

        let display = chip8.display();

        sdl_canvas.set_draw_color(Color::RGB(0, 0, 0));
        sdl_canvas.clear();

        for (i, value) in display.iter().enumerate() {
            if *value {
                sdl_canvas.set_draw_color(Color::RGB(255, 255, 255));

                // Translate from 1d -> 2d.
                let row = i / WIDTH as usize;
                let col = i - (row * WIDTH as usize);

                // Convert to screen coordinates.
                let screen_x = col * SCALE as usize;
                let screen_y = row * SCALE as usize;

                // Draw
                let filled_rect = Rect::new(screen_x as i32, screen_y as i32, SCALE as u32, SCALE as u32);
                sdl_canvas.fill_rect(filled_rect).expect("Fill failed.");
            }
        }

        for scancode in KEYS {
            let index = match_scancode(scancode).unwrap();
            if sdl_event_pump.keyboard_state().is_scancode_pressed(scancode) {

                chip8.set_key(index as usize, true)
            } else {
                chip8.set_key(index as usize, false)
            }
        }

        sdl_canvas.present();
    }
}