use std::time::{Duration, Instant};
use std::thread;
mod chip8;
use chip8::Chip8;

const CYCLES_PER_FRAME: u32 = 700 / 60;
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);


fn draw() {

}

fn main() {
    let mut chip8 = Chip8::new();

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
    }
}