mod font;
mod opcodes;
mod quirks;

pub use quirks::{Platform, Quirks};

use font::{FONT, FONT_START};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
pub const DISPLAY_SIZE: usize = WIDTH * HEIGHT;

const PC_START: usize = 0x200;

pub struct Chip8 {
    memory: [u8; 4096],
    display: [bool; DISPLAY_SIZE],
    pc: usize,
    i: usize,
    sp: usize,
    stack: [usize; 16],
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16],
    keypad: [bool; 16],
    quirks: Quirks,

    waiting_for_release: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct DebugState {
    pub pc: usize,
    pub i: usize,
    pub sp: usize,
    pub registers: [u8; 16],
}

impl Chip8 {
    pub fn new(quirks: Quirks) -> Self {
        let mut memory = [0u8; 4096];
        memory[FONT_START..FONT_START + FONT.len()].copy_from_slice(&FONT);

        Chip8 {
            memory,
            display: [false; DISPLAY_SIZE],
            pc: PC_START,
            i: 0,
            sp: 0,
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            keypad: [false; 16],
            quirks,

            waiting_for_release: None,
        }
    }

    pub fn set_key(&mut self, index: usize, pressed: bool) {
        self.keypad[index] = pressed;
    }

    pub fn display(&self) -> &[bool; DISPLAY_SIZE] {
        &self.display
    }

    pub fn load(&mut self, rom: &[u8]) {
        self.memory[PC_START..PC_START + rom.len()].copy_from_slice(rom);
    }

    /// Fetches, decodes, and executes a single instruction. Returns the
    /// opcode that was executed, mainly so debug/dev tools can trace it.
    pub fn tick(&mut self) -> u16 {
        let opcode: u16 = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
        self.pc += 2;

        self.execute(opcode);
        opcode
    }

    /// A snapshot of internal state, for debug tooling only.
    pub fn debug_state(&self) -> DebugState {
        DebugState {
            pc: self.pc,
            i: self.i,
            sp: self.sp,
            registers: self.registers,
        }
    }

    /// Steps the delay and sound timers down by one, as CHIP-8 expects at 60Hz.
    /// Returns whether the sound timer is still active after the step, so a
    /// platform layer can drive a beep without this core owning any audio IO.
    pub fn decrement(&mut self) -> bool {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
        self.sound_timer > 0
    }
}
