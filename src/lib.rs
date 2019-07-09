use rand::random;

const BUILTIN_SPRITES: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    mem: [u8; 0xFFF],
    v: [u8; 16],
    i: u16,
    pc: usize,
    screen: [u64; 32],
    delay: u8,
    sound: u8,
    sp: usize,
    stack: [u16; 16],
    keyboard: u16,
    halted: Option<u8>,
}

#[inline]
const fn r1(opcode: u16) -> usize {
    ((opcode & 0x0F00) >> 8) as usize
}

#[inline]
const fn r2(opcode: u16) -> usize {
    ((opcode & 0x00F0) >> 4) as usize
}

#[inline]
const fn cst(opcode: u16) -> u8 {
    opcode as u8
}

impl Chip8 {
    pub fn new(prg: Vec<u8>) -> Chip8 {
        let mut mem = [0; 0xFFF];
        for (i, b) in prg.iter().enumerate() {
            mem[0x200 + i] = *b;
        }
        for i in 0..80 {
            mem[i] = BUILTIN_SPRITES[i];
        }
        Chip8 {
            mem,
            v: [0; 16],
            i: 0,
            pc: 0x200,
            screen: [0; 32],
            delay: 0,
            sound: 0,
            sp: 0,
            stack: [0; 16],
            keyboard: 0,
            halted: None,
        }
    }

    pub fn decrement_delay(&mut self) {
        if self.delay > 0 {
            self.delay -= 1;
        }
    }

    pub fn sound(&self) -> u8 {
        self.sound
    }

    pub fn decrement_sound(&mut self) {
        if self.sound > 0 {
            self.sound -= 1;
        }
    }

    pub fn screen(&self) -> &[u64; 32] {
        &self.screen
    }

    pub fn press_key(&mut self, key: u8) {
        self.keyboard |= 0x1 << key;
        if self.halted.is_some() {
            let vx = self.halted.take().unwrap() as usize;
            self.v[vx] = key;
            self.pc += 2;
        }
    }

    pub fn release_key(&mut self, key: u8) {
        self.keyboard &= !(0x1 << key);
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        (self.keyboard & (0x1 << key)) != 0
    }

    pub fn step(&mut self) {
        if self.halted.is_some() {
            return;
        }
        let opcode: u16 = ((self.mem[self.pc] as u16) << 8) + self.mem[self.pc + 1] as u16;
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => self.screen = [0; 32],
                0x00EE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp] as usize;
                }
                _ => unreachable!("No such opcode: {:X}", opcode),
            },
            0x1000 => {
                self.pc = (opcode & 0x0FFF) as usize;
                return;
            }
            0x2000 => {
                self.stack[self.sp] = self.pc as u16;
                self.sp += 1;
                self.pc = (opcode & 0x0FFF) as usize;
                return;
            }
            0x3000 => {
                if self.v[r1(opcode)] == cst(opcode) {
                    self.pc += 2;
                }
            }
            0x4000 => {
                if self.v[r1(opcode)] != cst(opcode) {
                    self.pc += 2;
                }
            }
            0x5000 => {
                if self.v[r1(opcode)] == self.v[r2(opcode)] {
                    self.pc += 2;
                }
            }
            0x6000 => self.v[r1(opcode)] = cst(opcode),
            0x7000 => self.v[r1(opcode)] = self.v[r1(opcode)].wrapping_add(cst(opcode)),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.v[r1(opcode)] = self.v[r2(opcode)],
                0x0001 => self.v[r1(opcode)] |= self.v[r2(opcode)],
                0x0002 => self.v[r1(opcode)] &= self.v[r2(opcode)],
                0x0003 => self.v[r1(opcode)] ^= self.v[r2(opcode)],
                0x0004 => {
                    let (res, carry) = self.v[r1(opcode)].overflowing_add(self.v[r2(opcode)]);
                    self.v[0xF] = if carry { 1 } else { 0 };
                    self.v[r1(opcode)] = res;
                }
                0x0005 => {
                    let (res, borrow) = self.v[r1(opcode)].overflowing_sub(self.v[r2(opcode)]);
                    self.v[0xF] = if borrow { 0 } else { 1 };
                    self.v[r1(opcode)] = res;
                }
                0x0006 => {
                    self.v[0xF] = self.v[r2(opcode)] & 0x01;
                    self.v[r1(opcode)] = self.v[r2(opcode)].overflowing_shr(1).0;
                }
                0x0007 => {
                    let (res, borrow) = self.v[r2(opcode)].overflowing_sub(self.v[r1(opcode)]);
                    self.v[0xF] = if borrow { 0 } else { 1 };
                    self.v[r1(opcode)] = res;
                }
                0x000E => {
                    self.v[0xF] = (self.v[r2(opcode)] & 0b10000000) >> 7;
                    self.v[r1(opcode)] = self.v[r2(opcode)].overflowing_shl(1).0;
                }
                _ => unreachable!("No such opcode: {:X}", opcode),
            },
            0x9000 => {
                if self.v[r1(opcode)] != self.v[r2(opcode)] {
                    self.pc += 2;
                }
            }
            0xA000 => self.i = opcode & 0x0FFF,
            0xB000 => {
                self.pc = ((opcode & 0x0FFF) + self.v[0] as u16) as usize;
                return;
            }
            0xC000 => {
                self.v[r1(opcode)] = random::<u8>() & cst(opcode);
            }
            0xD000 => {
                let start: usize = self.i as usize;
                let n: usize = (opcode & 0x000F) as usize;
                let x = self.v[r1(opcode)];
                let mut y = self.v[r2(opcode)];
                y &= 31;
                for b in start..(n + start) {
                    self.draw_byte(self.mem[b], x as usize, y as usize);
                    y = (y + 1) & 31;
                }
            }
            0xE000 => match opcode & 0x00FF {
                0x009E => {
                    if self.keyboard & ((0x1 << self.v[r1(opcode)]) as u16) != 0 {
                        self.pc += 2;
                    }
                }
                0x00A1 => {
                    if self.keyboard & ((0x1 << self.v[r1(opcode)]) as u16) == 0 {
                        self.pc += 2;
                    }
                }
                _ => unreachable!("No such opcode: {:X}", opcode),
            },
            0xF000 => {
                match opcode & 0x00FF {
                    0x0007 => self.v[r1(opcode)] = self.delay,
                    // handled in key_press
                    0x000A => {
                        self.halted = Some(r1(opcode) as u8);
                        return;
                    }
                    0x0015 => self.delay = self.v[r1(opcode)],
                    0x0018 => self.sound = self.v[r1(opcode)],
                    0x001E => {
                        if self.i + self.v[r1(opcode)] as u16 > 0xFFF {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.i += self.v[r1(opcode)] as u16;
                    },
                    0x0029 => self.i = 5 * self.v[r1(opcode)] as u16,
                    0x0033 => {
                        let vx = self.v[r1(opcode)];
                        self.mem[self.i as usize] = vx as u8 / 100;
                        self.mem[self.i as usize + 1] = (vx as u8 / 10) % 10;
                        self.mem[self.i as usize + 2] = vx as u8 % 10;
                    }
                    0x0055 => {
                        for i in 0..=r1(opcode) {
                            self.mem[self.i as usize + i as usize] = self.v[i as usize];
                        }
                        self.i += r1(opcode) as u16 + 1;
                    }
                    0x0065 => {
                        for i in 0..=r1(opcode) {
                            self.v[i as usize] = self.mem[self.i as usize + i as usize];
                        }
                        self.i += r1(opcode) as u16 + 1
                    }
                    _ => unreachable!("No such opcode: {:X}", opcode),
                }
            }
            _ => unreachable!("No such opcode: {:X}", opcode),
        }
        self.pc += 2;
    }

    fn draw_byte(&mut self, byte: u8, x: usize, y: usize) {
        let mut to_store = 1;
        for i in 0..8 {
            let bit_to_write = (byte >> (7 - i) & 0x1) as u64;
            to_store ^= (!self.set_pixel(bit_to_write, (x + i) & 63, y)) as u8;
        }
        self.v[0xF] = !(to_store != 0) as u8;
    }

    fn set_pixel(&mut self, bit: u64, x: usize, y: usize) -> bool {
        let mask = 1 << (63 - x);
        let old_val = self.screen[y] & mask;
        self.screen[y] ^= bit << (63 - x);
        let new_val = self.screen[y] & mask;
        old_val > 0 && new_val == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cls() {
        let prog = vec![0x00, 0xE0];
        let mut chip = Chip8::new(prog);
        for i in 0..32 {
            chip.screen[i] = 0xFFFFFFFFFFFFFFFF;
        }
        chip.step();
        for i in 0..32 {
            assert_eq!(chip.screen[i], 0x0);
        }
    }

    #[test]
    fn ret() {
        let prog = vec![0x00, 0xEE];
        let mut chip = Chip8::new(prog);
        chip.sp = 1;
        chip.stack[1] = 0xAB;
        chip.step();
        assert_eq!(chip.sp, 0);
        assert_eq!(chip.pc, 2);
    }

    #[test]
    fn jp() {
        let prog = vec![0x1A, 0xBC];
        let mut chip = Chip8::new(prog);
        chip.step();
        assert_eq!(chip.pc, 0xABC);
    }

    #[test]
    fn call() {
        let prog = vec![0x2A, 0xBC];
        let mut chip = Chip8::new(prog);
        chip.step();
        assert_eq!(chip.sp, 1);
        assert_eq!(chip.stack[0], 0x200);
        assert_eq!(chip.pc, 0xABC);
    }

    #[test]
    fn se() {
        let prog = vec![0x30, 0x11, 0x31, 0x00];
        let mut chip = Chip8::new(prog);
        chip.step();
        assert_eq!(chip.pc, 0x202);
        chip.step();
        assert_eq!(chip.pc, 0x206);
    }

    #[test]
    fn sne() {
        let prog = vec![0x40, 0x00, 0x41, 0x11];
        let mut chip = Chip8::new(prog);
        chip.step();
        assert_eq!(chip.pc, 0x202);
        chip.step();
        assert_eq!(chip.pc, 0x206);
    }

    #[test]
    fn se_v() {
        let prog = vec![0x50, 0x10, 0x51, 0x20];
        let mut chip = Chip8::new(prog);
        chip.v[0] = 1;
        chip.step();
        assert_eq!(chip.pc, 0x202);
        chip.step();
        assert_eq!(chip.pc, 0x206);
    }

    #[test]
    fn ld() {
        let prog = vec![0x6E, 0x12];
        let mut chip = Chip8::new(prog);
        chip.step();
        assert_eq!(chip.v[0xE], 0x12);
    }

    #[test]
    fn add() {
        let prog = vec![0x7E, 0x12];
        let mut chip = Chip8::new(prog);
        chip.v[0xE] = 5;
        chip.step();
        assert_eq!(chip.v[0xE], 0x17);
    }

    #[test]
    fn ld_v() {
        let prog = vec![0x8E, 0x10];
        let mut chip = Chip8::new(prog);
        chip.v[0x1] = 5;
        chip.step();
        assert_eq!(chip.v[0xE], 0x5);
    }

    #[test]
    fn or() {
        let prog = vec![0x8E, 0x11];
        let mut chip = Chip8::new(prog);
        chip.v[0x1] = 0xF;
        chip.v[0xE] = 0xF;
        chip.step();
        assert_eq!(chip.v[0xE], 0xF);
    }

    #[test]
    fn and() {
        let prog = vec![0x8E, 0x12];
        let mut chip = Chip8::new(prog);
        chip.v[0x1] = 0xF;
        chip.step();
        assert_eq!(chip.v[0xE], 0x0);
    }

    #[test]
    fn xor() {
        let prog = vec![0x8E, 0x13];
        let mut chip = Chip8::new(prog);
        chip.v[0x1] = 0xF;
        chip.v[0xE] = 0xF;
        chip.step();
        assert_eq!(chip.v[0xE], 0x0);
    }

    #[test]
    fn add_overflow() {
        let prog = vec![0x8E, 0x14, 0x8E, 0x14];
        let mut chip = Chip8::new(prog);
        chip.v[0x1] = 0xFE;
        chip.v[0xE] = 0x1;
        chip.step();
        assert_eq!(chip.v[0xE], 0xFF);
        assert_eq!(chip.v[0xF], 0x0);
        chip.step();
        assert_eq!(chip.v[0xE], 0xFD);
        assert_eq!(chip.v[0xF], 0x1);
    }

    #[test]
    fn sub_overflow() {
        let prog = vec![0x8E, 0x15, 0x8E, 0x15];
        let mut chip = Chip8::new(prog);
        chip.v[0x1] = 0x1;
        chip.v[0xE] = 0x1;
        chip.step();
        assert_eq!(chip.v[0xE], 0x0);
        assert_eq!(chip.v[0xF], 0x1);
        chip.step();
        assert_eq!(chip.v[0xE], 0xFF);
        assert_eq!(chip.v[0xF], 0x0);
    }

    #[test]
    fn shr() {
        let prog = vec![0x8E, 0x06, 0x8E, 0xE6];
        let mut chip = Chip8::new(prog);
        chip.v[0x0] = 0x02;
        chip.step();
        assert_eq!(chip.v[0xE], 0x01);
        assert_eq!(chip.v[0x0], 0x02);
        assert_eq!(chip.v[0xF], 0x00);
        chip.step();
        assert_eq!(chip.v[0xE], 0x00);
        assert_eq!(chip.v[0xF], 0x01);
    }

    #[test]
    fn sub_overflow_rev() {
        let prog = vec![0x81, 0xE7, 0x81, 0xE7];
        let mut chip = Chip8::new(prog);
        chip.v[0x1] = 0x1;
        chip.v[0xE] = 0x1;
        chip.step();
        assert_eq!(chip.v[0x1], 0x0);
        assert_eq!(chip.v[0xE], 0x1);
        chip.step();
        assert_eq!(chip.v[0xE], 0x1);
        assert_eq!(chip.v[0x1], 0x1);
    }

    #[test]
    fn shl() {
        let prog = vec![0x8E, 0x0E, 0x8E, 0xEE];
        let mut chip = Chip8::new(prog);
        chip.v[0x0] = 0x40;
        chip.step();
        assert_eq!(chip.v[0xE], 0x80);
        assert_eq!(chip.v[0xF], 0x0);
        chip.step();
        assert_eq!(chip.v[0xE], 0x00);
        assert_eq!(chip.v[0xF], 0x01);
    }

    #[test]
    fn sne_v() {
        let prog = vec![0x90, 0x00, 0x90, 0x10];
        let mut chip = Chip8::new(prog);
        chip.v[0x0] = 0xFF;
        chip.step();
        assert_eq!(chip.pc, 0x202);
        chip.step();
        assert_eq!(chip.pc, 0x206);
    }

    #[test]
    fn ld_i() {
        let prog = vec![0xAA, 0xBC];
        let mut chip = Chip8::new(prog);
        chip.step();
        assert_eq!(chip.i, 0x0ABC);
    }

    #[test]
    fn jp_0() {
        let prog = vec![0xBA, 0xBC];
        let mut chip = Chip8::new(prog);
        chip.v[0x0] = 0x1;
        chip.step();
        assert_eq!(chip.pc, 0x0ABD);
    }

    #[test]
    fn set_pixel() {
        let mut chip = Chip8::new(vec![]);
        for j in 0..32 {
            for i in 0..64 {
                if i % 2 == 0 {
                    assert_eq!(chip.set_pixel(1, i, j), false);
                }
            }
            assert_eq!(chip.screen[j], 0xAAAAAAAAAAAAAAAA);
        }
        assert_eq!(chip.set_pixel(0, 0, 0), false);
        assert_eq!(chip.set_pixel(1, 62, 0), true);
        assert_eq!(chip.screen[0], 0xAAAAAAAAAAAAAAA8);
    }

    #[test]
    fn draw_byte() {
        let mut chip = Chip8::new(vec![]);
        for i in 0..32 {
            chip.draw_byte(0xEE, 60, i);
            chip.draw_byte(0xFF, 4, i);
            assert_eq!(chip.screen[i], 0xEFF000000000000E);
            assert_eq!(chip.v[0xF], 0);
        }
        chip.draw_byte(0xFF, 4, 0);
        assert_eq!(chip.screen[0], 0xE00000000000000E);
        assert_eq!(chip.v[0xF], 1);
    }

    #[test]
    fn drw() {
        let prog = vec![0xD0, 0x12];
        let mut chip = Chip8::new(prog);
        chip.mem[0x204] = 0xFF;
        chip.mem[0x205] = 0xFF;
        chip.i = 0x0204;
        chip.v[0x0] = 60;
        chip.v[0x1] = 31;
        chip.step();
        assert_eq!(chip.screen[0], 0xF00000000000000F);
        assert_eq!(chip.screen[31], 0xF00000000000000F);
        for i in 0..32 {
            if i != 0 && i != 31 {
                assert_eq!(chip.screen[i], 0x0000000000000000);
            }
        }
    }

    #[test]
    fn skp() {
        let prog = vec![0xE0, 0x9E, 0xE0, 0x9E];
        let mut chip = Chip8::new(prog);
        chip.step();
        assert_eq!(chip.pc, 0x202);
        chip.keyboard = 0x1;
        chip.step();
        assert_eq!(chip.pc, 0x206);
    }

    #[test]
    fn skpn() {
        let prog = vec![0xE0, 0xA1, 0xE0, 0xA1];
        let mut chip = Chip8::new(prog);
        chip.keyboard = 0x1;
        chip.step();
        assert_eq!(chip.pc, 0x202);
        chip.keyboard = 0x0;
        chip.step();
        assert_eq!(chip.pc, 0x206);
    }

    #[test]
    fn ldd() {
        let prog = vec![0xF0, 0x07];
        let mut chip = Chip8::new(prog);
        chip.delay = 0xA;
        chip.step();
        assert_eq!(chip.v[0x0], 0xA);
    }

    #[test]
    fn ldk() {
        let prog = vec![0xF0, 0x0A];
        let mut chip = Chip8::new(prog);
        chip.step();
        assert_eq!(chip.pc, 0x200);
        chip.step();
        chip.step();
        assert_eq!(chip.pc, 0x200);
        chip.press_key(1);
        assert_eq!(chip.pc, 0x202);
        assert_eq!(chip.v[0], 0x1);
    }

    #[test]
    fn ldd_set() {
        let prog = vec![0xF0, 0x15];
        let mut chip = Chip8::new(prog);
        chip.v[0] = 0xA;
        chip.step();
        assert_eq!(chip.delay, 0xA);
    }

    #[test]
    fn lds_set() {
        let prog = vec![0xF0, 0x18];
        let mut chip = Chip8::new(prog);
        chip.v[0] = 0xA;
        chip.step();
        assert_eq!(chip.sound, 0xA);
    }

    #[test]
    fn add_i() {
        let prog = vec![0xF0, 0x1E];
        let mut chip = Chip8::new(prog);
        chip.v[0] = 0xA;
        chip.step();
        assert_eq!(chip.i, 0xA);
    }

    #[test]
    fn ldf() {
        let prog = vec![0xF0, 0x29];
        let mut chip = Chip8::new(prog);
        chip.v[0] = 0xA;
        chip.step();
        assert_eq!(chip.i, 0x32);
    }

    #[test]
    fn ldb() {
        let prog = vec![0xF0, 0x33];
        let mut chip = Chip8::new(prog);
        chip.v[0] = 123;
        chip.step();
        assert_eq!(chip.mem[chip.i as usize], 0x1);
        assert_eq!(chip.mem[(chip.i + 1) as usize], 0x2);
        assert_eq!(chip.mem[(chip.i + 2) as usize], 0x3);
    }

    #[test]
    fn ld_store_regs() {
        let prog = vec![0xF4, 0x55];
        let mut chip = Chip8::new(prog);
        let values = vec![0x1, 0x2, 0x3, 0x4];
        for i in 0..4 {
            chip.v[i] = values[i];
        }
        chip.i = 0x300;
        chip.step();
        for i in 0..4 {
            assert_eq!(chip.mem[(0x300 + i) as usize], values[i as usize]);
        }
        assert_eq!(chip.i, 0x305);
    }

    #[test]
    fn ld_retrieve_regs() {
        let prog = vec![0xF5, 0x65];
        let mut chip = Chip8::new(prog);
        let values = vec![0x1, 0x2, 0x3, 0x4];
        chip.v[0x5] = 3;
        chip.i = 0x300;
        for i in 0..4 {
            chip.mem[(chip.i + i) as usize] = values[i as usize];
        }
        chip.step();
        for i in 0..4 {
            assert_eq!(chip.v[i], values[i]);
        }
    }

    #[test]
    fn is_pressed() {
        let mut chip = Chip8::new(vec![]);
        chip.keyboard = 0x1;
        assert!(chip.is_pressed(0));
    }

    #[test]
    fn press_key() {
        let mut chip = Chip8::new(vec![]);
        chip.press_key(1);
        chip.press_key(10);
        assert_eq!(chip.keyboard, 0b0000010000000010)
    }
}
