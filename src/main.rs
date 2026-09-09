use std::time::{Duration, Instant};
use std::{fs, thread};

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use retro::chip8::{Chip8, HEIGHT, WIDTH};
use retro::config;

mod audio;
mod input;

use audio::Beeper;
use input::{key_index, KEYS};

const CYCLES_PER_FRAME: u32 = 700 / 60;
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

const SCALE: i32 = 16;

fn main() {
    let quirks = config::load_quirks(&config::exe_relative("config.toml"));
    let mut chip8 = Chip8::new(quirks);
    let mut beeper = Beeper::new(&config::exe_relative("beep.wav"));

    let sdl_context = sdl2::init().expect("Failed to build sdl_context.");
    let sdl_video = sdl_context.video().expect("Failed to build sdl_video");
    let sdl_window = sdl_video
        .window("CHIP-8", (WIDTH as i32 * SCALE) as u32, (HEIGHT as i32 * SCALE) as u32)
        .position_centered()
        .build()
        .expect("Failed to build sdl_window");
    let mut sdl_canvas = sdl_window.into_canvas().build().expect("Failed to build sdl_canvas");
    let mut sdl_event_pump = sdl_context.event_pump().expect("Failed to build sdl_event_pump");

    let rom_path = std::env::args().nth(1).unwrap_or_else(|| "roms/5-Quirks.ch8".to_string());
    let rom = fs::read(&rom_path).unwrap_or_else(|err| panic!("Failed to load ROM {rom_path}: {err}"));
    chip8.load(&rom);

    loop {
        let frame_start = Instant::now();

        for _ in 0..CYCLES_PER_FRAME {
            chip8.tick();
        }

        let sound_active = chip8.decrement();
        beeper.set_playing(sound_active);

        for event in sdl_event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                return;
            }
        }

        for scancode in KEYS {
            let index = key_index(scancode).unwrap();
            let pressed = sdl_event_pump.keyboard_state().is_scancode_pressed(scancode);
            chip8.set_key(index as usize, pressed);
        }

        draw(&mut sdl_canvas, chip8.display());

        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            thread::sleep(FRAME_DURATION - elapsed);
        }
    }
}

fn draw(canvas: &mut sdl2::render::WindowCanvas, display: &[bool; WIDTH * HEIGHT]) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, &on) in display.iter().enumerate() {
        if !on {
            continue;
        }

        let row = i / WIDTH;
        let col = i % WIDTH;

        let rect = Rect::new(
            (col as i32) * SCALE,
            (row as i32) * SCALE,
            SCALE as u32,
            SCALE as u32,
        );
        canvas.fill_rect(rect).expect("Fill failed.");
    }

    canvas.present();
}
