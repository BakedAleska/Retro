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

const FONT_START: usize = 0x50;
const PC_START: usize = 0x200;

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0u8; 4096];
        memory[FONT_START..FONT_START + FONT.len()].copy_from_slice(&FONT);

        Chip8 {
            memory,
            display: [false; 64 * 32],
            pc: PC_START,
            i: 0,
            sp: 0,
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            keypad: [false; 16],
        }
    }

    pub fn display(&self) -> &[bool; 2048] {
        &self.display
    }

    pub fn load(&mut self, rom: Vec<u8>) {
        self.memory[PC_START..PC_START + rom.len()].copy_from_slice(&*rom);
    }

    pub fn tick(&mut self) {

        // Fetch
        let opcode: u16 = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
        self.pc += 2;

        let first_nibble: u16 = opcode >> 12 & 0xF;
        let second_nibble: u16 = opcode >> 8 & 0xF;
        let third_nibble: u16 = opcode >> 4 & 0xF;
        let last_nibble: u16 = opcode & 0xF;

        match first_nibble {
            0x0 => {
                match last_nibble {
                    0x0 => {
                        // Clear screen.
                        self.display = [false; 64 * 32];
                    },
                    0xE => {
                        // Subroutines.
                        // If there's something to pop, pop the last address from stack, and set pc to it.
                        if (self.sp != 0) {

                            self.pc = self.stack[self.sp - 1];
                            self.sp -= 1
                        }
                    }
                    _ => {}
                }
            },
            0x1 => {
                // Jump.
               self.pc = (opcode & 0xFFF) as usize;
            },
            0x6 => {
                // Set.
                self.registers[second_nibble as usize] = (opcode & 0xFF) as u8;
            },
            0x7 => {
                // Add.
                self.registers[second_nibble as usize] = self.registers[second_nibble as usize].wrapping_add((opcode & 0xFF) as u8);
            },
            0xA => {
                // Set index.
                self.i = (opcode & 0xFFF) as usize;
            },
            0xD => {
                // Draw.
                let x_coord = self.registers[second_nibble as usize];
                let y_coord = self.registers[third_nibble as usize];

                let wrap_x = x_coord % 64;
                let wrap_y = y_coord % 32;

                let vf = &mut self.registers[0xF];
                *vf = 0;

                for row in 0..last_nibble {
                    let sprite_byte = self.memory[self.i + row as usize];

                    for col in 0..8 {
                        let shift = 7 - col;

                        let sprite_pixel = sprite_byte >> shift & 1;

                        let screen_x = wrap_x + col;
                        let screen_y = wrap_y + row as u8;

                        if wrap_x + col > 63 || (wrap_y + row as u8) > 31 {
                            continue;
                        }

                        let display_index = (screen_y as usize * 64 + screen_x as usize);
                        let display_pixel = &mut self.display[display_index];

                        if (sprite_pixel == 1) {
                            if (*display_pixel == true) {
                                *display_pixel = false;
                                *vf = 1;
                            } else {
                                *display_pixel = true;
                            }
                        }
                    }
                }
            }
            _ => {
                println!("Unkown instruction: {:#X}", opcode)
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