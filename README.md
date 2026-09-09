# Retro — a CHIP-8 emulator in Rust

A CHIP-8 interpreter/emulator built from scratch in Rust, rendered with SDL2. It implements the full 35-instruction CHIP-8 instruction set, a configurable quirks layer (so it can emulate either the original COSMAC VIP interpreter or the CHIP-48/SUPER-CHIP variant), keyboard input, and sound.

## Highlights

- **Correct where it counts.** Passes the [Timendus CHIP-8 test suite](https://github.com/Timendus/chip8-test-suite)'s opcode (`3-corax+`) and flags (`4-flags`) ROMs with zero reported failures — see [Test ROM results](#test-rom-results) below.
- **123M+ emulated instructions/sec** on a representative instruction mix (measured with Criterion — see [Performance](#performance)), against a target workload of 700 instructions/sec. The interpreter is nowhere near the bottleneck; the 60Hz frame loop is.
- **Found and fixed a real bug via profiling**, not just guessing: the original implementation re-opened and re-decoded `beep.wav` from disk on every tick the sound timer was active (up to 60x/sec). Benchmarking that path showed it cost ~236µs per call — decoding once at startup instead cut the per-tick audio cost by roughly **400x**.
- **17 unit tests + 3 ROM-driven integration tests** covering the ambiguous/quirky corners of the spec (carry/borrow flags, shift source register, VF-reset-on-logic, memory increment behavior, sprite clipping, index overflow) under both supported quirk presets.
- **A headless snapshot/tracing tool** (`cargo run --bin snapshot`) that runs any ROM with no display or audio device and dumps the resulting screen to a PNG, or traces executed opcodes to stderr — used throughout development to verify against the test ROMs without eyeballing an SDL window.

## Controls

Standard CHIP-8 keypad, mapped onto a QWERTY keyboard:

```
1 2 3 4        1 2 3 C
Q W E R   ->   4 5 6 D
A S D F        7 8 9 E
Z X C V        A 0 B F
```

## Configurable quirks

`config.toml` selects which interpreter's ambiguous behavior to emulate:

```toml
version = "CHIP-48"   # or "CHIP-8"
```

| Quirk | CHIP-8 (original) | CHIP-48 |
|---|---|---|
| `8XY6`/`8XYE` shift source | VY | VX |
| `BNNN` jump-with-offset | `NNN + V0` | `NNN + VX` |
| `FX55`/`FX65` memory ops | increments `I` | leaves `I` unchanged |
| `8XY1`/`8XY2`/`8XY3` logic ops | resets `VF` to 0 | leaves `VF` untouched |
| `DXYN` sprite drawing | clips at screen edge (both presets) | clips at screen edge |

## Architecture

The emulation core is a plain library crate with no OS dependencies, so it can run headlessly in tests, benchmarks, and the snapshot tool — only `src/main.rs` (the SDL2 platform layer) touches a window, an audio device, or the filesystem beyond loading a ROM.

```
src/
├── lib.rs            # crate root, re-exports the public API
├── config.rs          # config.toml -> Quirks, exe-relative path resolution
├── chip8/
│   ├── mod.rs          # Chip8 struct, fetch/decode/tick, public API
│   ├── opcodes.rs       # instruction implementations + unit tests
│   ├── quirks.rs         # Quirks/Platform: the ambiguous-instruction config
│   └── font.rs            # built-in hex digit sprites
├── audio.rs           # SDL/rodio beep playback (decoded once, not per-tick)
├── input.rs            # keyboard scancode <-> CHIP-8 key mapping
├── main.rs             # SDL2 window, event loop, 700Hz/60Hz timing
└── bin/
    └── snapshot.rs      # headless dev tool: ROM -> PNG / opcode trace

tests/test_roms.rs      # integration tests against roms/*.ch8
benches/                # criterion throughput + audio-path benchmarks
```

## Test ROM results

Verified with the headless snapshot tool (`cargo run --bin snapshot`), and pinned down as regression tests in `tests/test_roms.rs`:

| ROM | Result |
|---|---|
| `IBM Logo.ch8` | Renders correctly |
| `Corax+.ch8` (opcode test) | **22/22 checks pass**, 0 failures |
| `Flags.ch8` (carry/borrow/shift flag test) | **47/47 checks pass**, 0 failures |
| `5-Quirks.ch8`, `6-Keypad.ch8` | Interactive, menu-driven ROMs (select platform/target with a keypress before running) — the opcodes they exercise are covered by the unit tests above; verify these two by running them directly in the SDL app |

<p>
  <img src="docs/screenshots/ibm-logo.png" width="256" alt="IBM logo test ROM"/>
  <img src="docs/screenshots/corax-plus.png" width="256" alt="Corax+ opcode test — all passing"/>
  <img src="docs/screenshots/flags.png" width="256" alt="Flags test — all passing"/>
</p>

## Performance

Measured with `cargo bench` (Criterion), on the developer's machine:

- **`tick()` throughput:** ~81µs per 10,000 emulated instructions across a representative mix (register ops, arithmetic with carry, indexing, RNG, conditional skips, screen clear) ⇒ **~123M instructions/sec**. CHIP-8 programs run at roughly 700 instructions/sec, so the interpreter uses on the order of 0.0006% of a frame's instruction budget.
- **Audio path fix:** the original `decrement()` opened and decoded `beep.wav` from disk on every tick the sound timer was nonzero. Benchmarked cost: **236µs/call**. The rewritten `audio::Beeper` decodes once at startup and clones a cached sample buffer per tick instead: **0.58µs/call — ~405x cheaper**, and zero filesystem/decoder work once a beep starts.

Reproduce: `cargo bench`.

## Building and running

Requires the SDL2 development libraries (see the [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2) setup instructions for your platform).

```sh
cargo run --release                       # plays roms/5-Quirks.ch8 by default
cargo run --release -- roms/Pong.rom      # or pass a ROM path

cargo test                                # unit + integration tests
cargo bench                               # throughput/audio benchmarks

# headless dev tool: dump a ROM's screen to a PNG after N cycles
cargo run --bin snapshot -- roms/Corax+.ch8 400 out.png
# ...or trace executed opcodes to stderr, and/or script key input:
cargo run --bin snapshot -- roms/5-Quirks.ch8 800 out.png --trace-from=0 --tap=390:2:60
```

## License

MIT
