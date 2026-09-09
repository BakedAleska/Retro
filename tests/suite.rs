//! Regression tests against the CHIP-8 test ROMs in `roms/`.
//!
//! These ROMs draw a "pass" or "fail" glyph from a fixed address in their
//! own memory for each sub-check they run, then jump to themselves in an
//! infinite loop once done. The exact addresses below were confirmed by
//! tracing execution with `cargo run --bin snapshot -- <rom> <cycles>
//! out.png --trace-from=0` and reading the ROM's disassembly: each test
//! sets I to a "fail" glyph address by default, then overwrites it with the
//! "pass" glyph address only if the actual result matched the expected one.
//! Asserting zero fails observed (and at least one pass, and that the ROM's
//! self-jump halt was actually reached) is a meaningful regression check
//! without needing to hand-verify a screenshot on every change.

use retro::chip8::{Chip8, Platform};

/// Runs `rom` until it executes `halt_opcode` (its own "loop forever" tail)
/// or `max_cycles` is exhausted, recording every draw at `marker_draw_opcode`
/// as a pass or fail depending on which glyph address `I` held.
struct RomResult {
    reached_halt: bool,
    passes: u32,
    fails: u32,
}

fn run_marker_rom(
    rom: &[u8],
    platform: Platform,
    marker_draw_opcode: u16,
    pass_addr: usize,
    fail_addr: usize,
    halt_opcode: u16,
    max_cycles: u32,
) -> RomResult {
    let mut chip8 = Chip8::new(platform.quirks());
    chip8.load(rom);

    let mut passes = 0;
    let mut fails = 0;

    for _ in 0..max_cycles {
        let opcode = chip8.tick();
        if opcode == halt_opcode {
            return RomResult {
                reached_halt: true,
                passes,
                fails,
            };
        }
        if opcode == marker_draw_opcode {
            let i = chip8.debug_state().i;
            if i == pass_addr {
                passes += 1;
            } else if i == fail_addr {
                fails += 1;
            }
        }
    }

    RomResult {
        reached_halt: false,
        passes,
        fails,
    }
}

#[test]
fn corax_plus_all_checks_pass() {
    let rom: &[u8] = include_bytes!("../roms/Corax+.ch8");
    let result = run_marker_rom(rom, Platform::Chip48, 0xDAB4, 0x4A5, 0x4A1, 0x149C, 400);

    assert!(result.reached_halt, "Corax+ never reached its halt loop");
    assert_eq!(result.fails, 0, "Corax+ reported failing opcode checks");
    assert!(result.passes > 0, "Corax+ reported no checks at all");
}

#[test]
fn flags_all_checks_pass() {
    let rom: &[u8] = include_bytes!("../roms/Flags.ch8");
    let result = run_marker_rom(rom, Platform::Chip48, 0xDAB3, 0x555, 0x558, 0x1542, 1200);

    assert!(result.reached_halt, "Flags never reached its halt loop");
    assert_eq!(result.fails, 0, "Flags reported failing flag checks");
    assert!(result.passes > 0, "Flags reported no checks at all");
}

#[test]
fn ibm_logo_draws_without_crashing() {
    let rom: &[u8] = include_bytes!("../roms/IBM Logo.ch8");
    let mut chip8 = Chip8::new(Platform::Chip48.quirks());
    chip8.load(rom);

    for _ in 0..30 {
        chip8.tick();
    }

    let lit_pixels = chip8.display().iter().filter(|&&on| on).count();
    assert!(lit_pixels > 100, "expected the IBM logo sprite to be drawn");
}
