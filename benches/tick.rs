//! Throughput benchmark for `Chip8::tick()`. Runs a small self-looping
//! program that exercises a representative mix of instruction classes
//! (register set, arithmetic w/ carry, index set, RNG, conditional skip,
//! screen clear, jump) rather than hammering a single opcode.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use retro::chip8::{Chip8, Platform};

const TICKS_PER_ITER: u64 = 10_000;

fn build_program() -> Vec<u8> {
    let opcodes: [u16; 9] = [
        0x6000, // V0 = 0
        0x6100, // V1 = 0
        0x7101, // V1 += 1
        0x8014, // V0 += V1 (sets VF on carry)
        0xA300, // I = 0x300
        0xC0FF, // V0 = rand() & 0xFF
        0x3000, // skip next if V0 == 0
        0x00E0, // clear screen
        0x1200, // jump back to start
    ];
    opcodes.iter().flat_map(|op| op.to_be_bytes()).collect()
}

fn bench_tick(c: &mut Criterion) {
    let rom = build_program();

    c.bench_function("tick_mixed_instructions", |b| {
        b.iter_batched(
            || {
                let mut chip8 = Chip8::new(Platform::Chip48.quirks());
                chip8.load(&rom);
                chip8
            },
            |mut chip8| {
                for _ in 0..TICKS_PER_ITER {
                    chip8.tick();
                }
                chip8
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_tick);
criterion_main!(benches);
