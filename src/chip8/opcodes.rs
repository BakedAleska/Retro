use super::font::FONT_START;
use super::Chip8;

impl Chip8 {
    #[allow(clippy::collapsible_match)] // clearer as an inner guard than a match-arm condition here
    pub(super) fn execute(&mut self, opcode: u16) {
        let first_nibble: u16 = opcode >> 12 & 0xF;
        let second_nibble: u16 = opcode >> 8 & 0xF;
        let third_nibble: u16 = opcode >> 4 & 0xF;
        let last_nibble: u16 = opcode & 0xF;
        let last_byte: u16 = opcode & 0xFF;

        match first_nibble {
            0 => match last_nibble {
                0x0 => self.display = [false; super::DISPLAY_SIZE],
                0xE => {
                    // Return from subroutine.
                    if self.sp != 0 {
                        self.sp -= 1;
                        self.pc = self.stack[self.sp];
                    }
                }
                _ => {}
            },
            1 => {
                // Jump.
                self.pc = (opcode & 0xFFF) as usize;
            }
            2 => {
                // Call subroutine.
                if self.sp < self.stack.len() {
                    self.stack[self.sp] = self.pc;
                    self.sp += 1;
                    self.pc = (opcode & 0xFFF) as usize;
                }
            }
            3 => {
                if self.registers[second_nibble as usize] == (opcode & 0xFF) as u8 {
                    self.pc += 2;
                }
            }
            4 => {
                if self.registers[second_nibble as usize] != (opcode & 0xFF) as u8 {
                    self.pc += 2;
                }
            }
            5 => {
                if self.registers[second_nibble as usize] == self.registers[third_nibble as usize] {
                    self.pc += 2;
                }
            }
            6 => {
                self.registers[second_nibble as usize] = (opcode & 0xFF) as u8;
            }
            7 => {
                self.registers[second_nibble as usize] =
                    self.registers[second_nibble as usize].wrapping_add((opcode & 0xFF) as u8);
            }
            8 => self.execute_arithmetic(second_nibble as usize, third_nibble as usize, last_nibble),
            9 => {
                if self.registers[second_nibble as usize] != self.registers[third_nibble as usize] {
                    self.pc += 2;
                }
            }
            0xA => self.i = (opcode & 0xFFF) as usize,
            0xB => {
                let base = if self.quirks.jump_offset_vx {
                    self.registers[second_nibble as usize]
                } else {
                    self.registers[0]
                };
                self.pc = ((opcode & 0xFFF) as usize + base as usize) & 0xFFF;
            }
            0xC => {
                self.registers[second_nibble as usize] = rand::random::<u8>() & (opcode & 0xFF) as u8;
            }
            0xD => self.draw_sprite(second_nibble as usize, third_nibble as usize, last_nibble),
            0xE => match last_nibble {
                0xE => {
                    if self.keypad[self.registers[second_nibble as usize] as usize] {
                        self.pc += 2;
                    }
                }
                0x1 => {
                    if !self.keypad[self.registers[second_nibble as usize] as usize] {
                        self.pc += 2;
                    }
                }
                _ => {}
            },
            0xF => self.execute_f(second_nibble as usize, last_byte),
            _ => println!("Unknown instruction: {:#X}", opcode),
        }
    }

    fn execute_arithmetic(&mut self, vx: usize, vy: usize, op: u16) {
        match op {
            0x0 => self.registers[vx] = self.registers[vy],
            0x1 => {
                self.registers[vx] |= self.registers[vy];
                if self.quirks.vf_reset_on_logic {
                    self.registers[0xF] = 0;
                }
            }
            0x2 => {
                self.registers[vx] &= self.registers[vy];
                if self.quirks.vf_reset_on_logic {
                    self.registers[0xF] = 0;
                }
            }
            0x3 => {
                self.registers[vx] ^= self.registers[vy];
                if self.quirks.vf_reset_on_logic {
                    self.registers[0xF] = 0;
                }
            }
            0x4 => {
                let (result, carry) = self.registers[vx].overflowing_add(self.registers[vy]);
                self.registers[vx] = result;
                self.registers[0xF] = carry as u8;
            }
            0x5 => {
                let (result, borrow) = self.registers[vx].overflowing_sub(self.registers[vy]);
                self.registers[vx] = result;
                self.registers[0xF] = !borrow as u8;
            }
            0x6 => {
                let source = if self.quirks.shift_uses_vx {
                    self.registers[vx]
                } else {
                    self.registers[vy]
                };
                let dropped_bit = source & 1;
                self.registers[vx] = source >> 1;
                self.registers[0xF] = dropped_bit;
            }
            0x7 => {
                let (result, borrow) = self.registers[vy].overflowing_sub(self.registers[vx]);
                self.registers[vx] = result;
                self.registers[0xF] = !borrow as u8;
            }
            0xE => {
                let source = if self.quirks.shift_uses_vx {
                    self.registers[vx]
                } else {
                    self.registers[vy]
                };
                let dropped_bit = (source >> 7) & 1;
                self.registers[vx] = source << 1;
                self.registers[0xF] = dropped_bit;
            }
            _ => {}
        }
    }

    fn draw_sprite(&mut self, vx: usize, vy: usize, height: u16) {
        let origin_x = self.registers[vx] as usize % super::WIDTH;
        let origin_y = self.registers[vy] as usize % super::HEIGHT;

        self.registers[0xF] = 0;

        for row in 0..height as usize {
            let sprite_byte = self.memory[self.i + row];
            let screen_y = origin_y + row;

            if screen_y >= super::HEIGHT && self.quirks.clip_sprites {
                continue;
            }
            let screen_y = screen_y % super::HEIGHT;

            for col in 0..8 {
                let screen_x = origin_x + col;

                if screen_x >= super::WIDTH && self.quirks.clip_sprites {
                    continue;
                }
                let screen_x = screen_x % super::WIDTH;

                let sprite_pixel = (sprite_byte >> (7 - col)) & 1 == 1;
                if !sprite_pixel {
                    continue;
                }

                let display_pixel = &mut self.display[screen_y * super::WIDTH + screen_x];
                if *display_pixel {
                    *display_pixel = false;
                    self.registers[0xF] = 1;
                } else {
                    *display_pixel = true;
                }
            }
        }
    }

    fn execute_f(&mut self, vx: usize, last_byte: u16) {
        match last_byte {
            0x07 => self.registers[vx] = self.delay_timer,
            0x0A => self.wait_for_key(vx),
            0x15 => self.delay_timer = self.registers[vx],
            0x18 => self.sound_timer = self.registers[vx],
            0x1E => {
                let sum = self.i + self.registers[vx] as usize;
                self.registers[0xF] = (sum > 0xFFF) as u8;
                self.i = sum & 0xFFF;
            }
            0x29 => self.i = FONT_START + (self.registers[vx] as usize & 0xF) * 5,
            0x33 => {
                let num = self.registers[vx];
                self.memory[self.i] = num / 100;
                self.memory[self.i + 1] = (num / 10) % 10;
                self.memory[self.i + 2] = num % 10;
            }
            0x55 => {
                for reg in 0..=vx {
                    self.memory[self.i + reg] = self.registers[reg];
                }
                if !self.quirks.memory_leaves_i {
                    self.i += vx + 1;
                }
            }
            0x65 => {
                for reg in 0..=vx {
                    self.registers[reg] = self.memory[self.i + reg];
                }
                if !self.quirks.memory_leaves_i {
                    self.i += vx + 1;
                }
            }
            _ => {}
        }
    }

    /// FX0A: blocks until a key is pressed and then released, writing the key to VX.
    fn wait_for_key(&mut self, vx: usize) {
        match self.waiting_for_release {
            Some(key_index) => {
                if self.keypad[key_index] {
                    self.pc -= 2;
                } else {
                    self.waiting_for_release = None;
                }
            }
            None => {
                let pressed_key = self.keypad.iter().position(|&pressed| pressed);
                match pressed_key {
                    Some(index) => {
                        self.registers[vx] = index as u8;
                        self.waiting_for_release = Some(index);
                        self.pc -= 2;
                    }
                    None => self.pc -= 2,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Chip8;
    use crate::chip8::{Platform, Quirks};

    fn chip8(quirks: Quirks) -> Chip8 {
        Chip8::new(quirks)
    }

    fn run(chip8: &mut Chip8, program: &[u16]) {
        let bytes: Vec<u8> = program.iter().flat_map(|op| op.to_be_bytes()).collect();
        chip8.load(&bytes);
        for _ in 0..program.len() {
            chip8.tick();
        }
    }

    #[test]
    fn add_sets_carry_on_overflow() {
        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x60FF, 0x6102, 0x8014]); // V0=0xFF, V1=2, V0+=V1
        assert_eq!(c.registers[0], 0x01);
        assert_eq!(c.registers[0xF], 1);
    }

    #[test]
    fn add_clears_carry_without_overflow() {
        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x6001, 0x6102, 0x8014]);
        assert_eq!(c.registers[0], 0x03);
        assert_eq!(c.registers[0xF], 0);
    }

    #[test]
    fn sub_sets_vf_when_no_borrow() {
        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x6005, 0x6103, 0x8015]); // V0=5, V1=3, V0-=V1
        assert_eq!(c.registers[0], 2);
        assert_eq!(c.registers[0xF], 1);
    }

    #[test]
    fn sub_clears_vf_on_borrow() {
        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x6003, 0x6105, 0x8015]); // V0=3, V1=5, V0-=V1
        assert_eq!(c.registers[0], 0xFE);
        assert_eq!(c.registers[0xF], 0);
    }

    #[test]
    fn subn_reverses_operand_order() {
        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x6003, 0x6105, 0x8017]); // V0=3, V1=5, V0 = V1-V0
        assert_eq!(c.registers[0], 2);
        assert_eq!(c.registers[0xF], 1);
    }

    #[test]
    fn shift_right_uses_vx_under_chip48_quirk() {
        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x6003, 0x6180, 0x8016]); // V0=3, V1=0x80, V0 >>= (VX quirk: uses V0)
        assert_eq!(c.registers[0], 1);
        assert_eq!(c.registers[0xF], 1); // dropped bit from V0 (3 = 0b11)
    }

    #[test]
    fn shift_right_uses_vy_under_chip8_quirk() {
        let mut c = chip8(Platform::Chip8.quirks());
        run(&mut c, &[0x6003, 0x6180, 0x8016]); // V0=3, V1=0x80, V0 = V1 >> 1
        assert_eq!(c.registers[0], 0x40);
        assert_eq!(c.registers[0xF], 0); // dropped bit from V1 (0x80, lsb 0)
    }

    #[test]
    fn logic_ops_reset_vf_under_chip8_quirk_only() {
        let mut c = chip8(Platform::Chip8.quirks());
        run(&mut c, &[0x60FF, 0x610F, 0x6F01, 0x8012]); // VF=1, then V0 = V0 AND V1
        assert_eq!(c.registers[0xF], 0, "CHIP-8 quirk should reset VF after AND");

        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x60FF, 0x610F, 0x6F01, 0x8012]);
        assert_eq!(c.registers[0xF], 1, "CHIP-48 quirk should leave VF untouched");
    }

    #[test]
    fn bcd_splits_digits() {
        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x60FF, 0xA300, 0xF033]); // V0=255 -> [2,5,5]
        assert_eq!(&c.memory[0x300..0x303], &[2, 5, 5]);
    }

    #[test]
    fn memory_load_store_increments_i_without_quirk() {
        let mut c = chip8(Platform::Chip8.quirks());
        run(&mut c, &[0x6011, 0x6122, 0xA400, 0xF155]); // store V0,V1 at I=0x400, I should advance by 2
        assert_eq!(c.i, 0x402);
        assert_eq!(&c.memory[0x400..0x402], &[0x11, 0x22]);
    }

    #[test]
    fn memory_load_store_leaves_i_under_chip48_quirk() {
        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x6011, 0x6122, 0xA400, 0xF155]);
        assert_eq!(c.i, 0x400);
        assert_eq!(&c.memory[0x400..0x402], &[0x11, 0x22]);
    }

    #[test]
    fn index_overflow_sets_vf() {
        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x60FF, 0xAFFF, 0xF01E]); // I = 0xFFF + 0xFF, wraps past 0xFFF
        assert_eq!(c.registers[0xF], 1);
        assert_eq!(c.i, (0xFFF + 0xFF) & 0xFFF);
    }

    #[test]
    fn jump_with_offset_uses_v0_by_default() {
        let mut c = chip8(Platform::Chip8.quirks());
        run(&mut c, &[0x6010, 0x6205, 0xB200]); // V0=0x10, jump to 0x200 + V0
        assert_eq!(c.pc, 0x210);
    }

    #[test]
    fn jump_with_offset_uses_vx_under_chip48_quirk() {
        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x6010, 0x6205, 0xB200]); // V2=5, jump to 0x200 + V2 (X=2)
        assert_eq!(c.pc, 0x205);
    }

    #[test]
    fn stack_push_pop_round_trips() {
        let mut c = chip8(Platform::Chip48.quirks());
        run(&mut c, &[0x2210]); // call 0x210
        assert_eq!(c.pc, 0x210);
        assert_eq!(c.sp, 1);

        c.memory[0x210] = 0x00;
        c.memory[0x211] = 0xEE; // RET
        c.tick();
        assert_eq!(c.pc, 0x202);
        assert_eq!(c.sp, 0);
    }

    #[test]
    fn draw_clips_by_default() {
        let mut c = chip8(Platform::Chip48.quirks());
        c.memory[0x300] = 0xFF; // full row sprite
        run(&mut c, &[0x603D, 0x6100, 0xA300, 0xD011]); // V0=61 (x, near right edge), V1=0 (y)
        let on_screen: usize = c.display.iter().filter(|&&p| p).count();
        assert_eq!(on_screen, 3, "sprite should clip at the 64px screen edge");
    }

    #[test]
    fn draw_sets_vf_on_collision() {
        let mut c = chip8(Platform::Chip48.quirks());
        c.memory[0x300] = 0x80; // single lit pixel, top-left of sprite
        run(
            &mut c,
            &[0x6000, 0x6100, 0xA300, 0xD011, 0xD011], // draw the same pixel twice
        );
        assert_eq!(c.registers[0xF], 1, "second draw should report a collision");
        assert!(!c.display[0], "XOR of the same pixel twice should erase it");
    }
}
