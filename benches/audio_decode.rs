//! Measures the concrete cost of the audio bug this rewrite fixed: the
//! original `Chip8::decrement()` re-opened and re-decoded `beep.wav` from
//! disk on every tick the sound timer was nonzero (up to 60x/sec). The
//! current `audio::Beeper` decodes it once at startup instead. This bench
//! quantifies the per-call cost that used to run 60x/sec during any beep.

use criterion::{criterion_group, criterion_main, Criterion};
use rodio::{Decoder, Source};
use std::fs::File;
use std::io::BufReader;

fn beep_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("beep.wav")
}

fn bench_decode_per_call(c: &mut Criterion) {
    // What the old code did every tick the sound timer was active.
    c.bench_function("beep_open_and_decode_per_call", |b| {
        b.iter(|| {
            let file = BufReader::new(File::open(beep_path()).unwrap());
            let decoded = Decoder::new(file).unwrap();
            let _samples: Vec<i16> = decoded.convert_samples().collect();
        })
    });
}

fn bench_clone_cached_buffer(c: &mut Criterion) {
    // What the new Beeper does per tick: append a cheap clone of an
    // already-decoded buffer instead of touching the filesystem/decoder.
    let file = BufReader::new(File::open(beep_path()).unwrap());
    let decoded = Decoder::new(file).unwrap();
    let channels = decoded.channels();
    let sample_rate = decoded.sample_rate();
    let samples: Vec<i16> = decoded.convert_samples().collect();
    let buffer = rodio::buffer::SamplesBuffer::new(channels, sample_rate, samples);

    c.bench_function("beep_clone_cached_buffer", |b| {
        b.iter(|| {
            let _clone = std::hint::black_box(buffer.clone());
        })
    });
}

criterion_group!(benches, bench_decode_per_call, bench_clone_cached_buffer);
criterion_main!(benches);
