pub struct Chip8 {
    memory: [u8; 4096], // 4 kilobytes of RAM.
    display: [bool; 64 * 32], // 64 / 32 pixel "screen".
    pc: usize, // Program counter.
    i: usize, // 16 bit index register
    sp: usize, // Stack pointer.
    stack: [usize; 16], // Stack; holds 16 bit addresses.
    delay_timer: u8, // Decremented at a rate of 60 Hz until it reaches 0.
    sound_timer: u8, // Functions like the delay timer but gives off a beeping sound as long as it isn't 0.
    registers: [u8; 16], // 16, 8-bit general-purpose variable registers.
    keypad: [bool; 16], // Current press / released state of keys 0-F
}
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

const FONT_START_ADDRESS: usize = 0x50;

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0u8; 4096];
        memory[FONT_START_ADDRESS..FONT_START_ADDRESS + FONT.len()].copy_from_slice(&FONT);

        Chip8 {
            memory,
            display: [false; 64 * 32],
            pc: 0x200,
            i: 0,
            sp: 0,
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            keypad: [false; 16],
        }
    }

    pub fn tick(&mut self) {
        // Fetch
        let opcode: u16 = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
        self.pc += 2;

        // Decode / Execute
        match opcode {
            0x00E0 => {
                self.display = [false; 64 * 32];
            },
            _ => {
                println!("Unknown instruction: {:#X}", opcode)
            }
        }
    }

    pub fn decrement(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);

        if self.sound_timer > 0 {
            println!("BEEP"); // !todo: Add beep sound to sound_timer decrement
        }

        self.sound_timer = self.sound_timer.saturating_sub(1);
    }
}