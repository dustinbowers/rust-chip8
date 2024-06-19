use macroquad::rand::rand;
use std::sync::{Arc, Mutex};

#[macro_use]
mod util;

pub type Screen = [[bool; 64]; 32];

const FONT_SET: [u8; 80] = [
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

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Key {
    Key0 = 0x0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    A,
    B,
    C,
    D,
    E,
    F,
}

#[derive(Debug)]
pub struct Chip8 {
    screen: Arc<Mutex<Screen>>,
    memory: [u8; 4096],
    stack: [u16; 16],
    keyboard: [bool; 16],

    v: [u8; 16], // 16 8-bit registers (note VF is a carry-flag register)
    pc: u16,     // Program/Instruction counter
    i: u16,      // Index register
    sp: u16,     // Stack pointer
    dt: u8,      // Delay timer
    st: u8,      // Sound timer

    halt_for_input: bool,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut c = Self {
            screen: Arc::new(Mutex::new([[false; 64]; 32])),
            memory: [0u8; 4096],
            stack: [0u16; 16],
            keyboard: [false; 16],
            v: [0u8; 16],
            pc: 0x200,
            i: 0,
            sp: 0,
            dt: 0,
            st: 0,
            halt_for_input: false,
        };
        c.load_font();
        return c;
    }

    #[inline]
    fn load_font(&mut self) {
        const FONT_OFFSET: usize = 0x050;
        for (i, v) in FONT_SET.iter().enumerate() {
            self.memory[i + FONT_OFFSET] = *v;
        }
    }

    pub fn load_rom(&mut self, bytes: Vec<u8>, start_offset: u16) -> Result<(), String> {
        let start_offset = start_offset as usize;
        if bytes.len() + start_offset >= 4096 {
            return Err("Invalid rom".to_string());
        }
        for (i, v) in bytes.iter().enumerate() {
            self.memory[i + start_offset] = *v;
        }
        Ok(())
    }

    pub fn get_screen(&self) -> Arc<Mutex<Screen>> {
        Arc::clone(&self.screen)
    }

    pub fn step(&mut self) -> Result<i32, ()> {
        let opcode = self.fetch_opcode();
        // self.inspect(opcode);

        match opcode & 0xF000 {
            0x0000 => {
                match get_kk!(opcode) {
                    0x00E0 => {
                        // CLS
                        let mut screen_writer = self.screen.lock().unwrap();
                        for row in screen_writer.iter_mut() {
                            for cell in row.iter_mut() {
                                *cell = false;
                            }
                        }
                    }
                    0x00EE => {
                        // RET
                        self.pc = self.stack[self.sp as usize];
                        self.sp -= 1;
                    }
                    _ => {
                        invalid_opcode!(opcode)
                    }
                }
            }
            0x1000 => {
                // (1nnn) JMP addr
                self.pc = get_nnn!(opcode);
            }
            0x2000 => {
                // (2nnn) CALL addr
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = get_nnn!(opcode);
            }
            0x3000 => {
                // (3xkk) SE Vx, byte - skip if equal
                if self.v[get_x!(opcode) as usize] == get_kk!(opcode) as u8 {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // (4xkk) SNE Vx, byte - skip if not equal
                if self.v[get_x!(opcode)] != get_kk!(opcode) as u8 {
                    self.pc += 2;
                }
            }
            0x5000 => {
                // (5xy0) - SE Vx, Vy - skip if registers are equal
                if self.v[get_x!(opcode)] == self.v[get_y!(opcode)] {
                    self.pc += 2;
                }
            }
            0x6000 => {
                // (6xkk) - LD Vx, byte - Load byte into register
                self.v[get_x!(opcode)] = get_kk!(opcode);
            }
            0x7000 => {
                // (7xkk) Add Vx, byte - Add byte to register
                let v_x = &mut self.v[get_x!(opcode)];
                let (s, _) =  v_x.overflowing_add(get_kk!(opcode));
                *v_x = s;
            }
            0x8000 => {
                //
                match get_n!(opcode) {
                    0x0 => {
                        // (8xy0) - LD Vx, Vy - move V_y into V_x
                        self.v[get_x!(opcode)] = self.v[get_y!(opcode)];
                    }
                    0x1 => {
                        // (8xy1) - OR Vx, Vy - Compute V_x |= V_y
                        self.v[get_x!(opcode)] |= self.v[get_y!(opcode)];
                    }
                    0x2 => {
                        // (8xy2) - AND Vx, Vy - Compute V_x &= V_y
                        self.v[get_x!(opcode)] &= self.v[get_y!(opcode)];
                    }
                    0x3 => {
                        // (8xy3) - XOR Vx, Vy - Compute V_x ^= V_y
                        self.v[get_x!(opcode)] ^= self.v[get_y!(opcode)];
                    }
                    0x4 => {
                        // 8xy4 - ADD Vx, Vy - Compute V_x += V_y, set overflow
                        let x = get_x!(opcode);
                        let y = get_y!(opcode);
                        let (s, overflow) = self.v[x].overflowing_add(self.v[y]);
                        self.v[x] = s;
                        self.v[0xF] = match overflow {
                            true => 1,
                            false => 0,
                        };
                    }
                    0x5 => {
                        // (8xy5) - SUB Vx, Vy - Compute V_x -= V_y, set underflow
                        let x = get_x!(opcode);
                        let y = get_y!(opcode);
                        let (s, underflow) = self.v[x].overflowing_sub(self.v[y]);
                        self.v[x] = s;
                        self.v[0xF] = match underflow {
                            true => 0,
                            false => 1,
                        };
                    }
                    0x6 => {
                        // (8xy6) - SHR Vx - Compute V_x >>= 1, store least-sig bit in VF
                        let x = get_x!(opcode);
                        let v_x = self.v[x];
                        self.v[0xF] = v_x & 0x1;
                        self.v[x] = v_x >> 1;
                    }
                    0x7 => {
                        // (8xy7) - SUBN Vx, Vy - Compute V_x = V_y - V_x, set underflow
                        let x = get_x!(opcode);
                        let y = get_y!(opcode);
                        let (s, underflow) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[x] = s;
                        self.v[0xF] = match underflow {
                            true => 0,
                            false => 1,
                        };
                    }
                    0xE => {
                        // (8xyE) - SHL Vx - Computer V_x <<= 1,
                        let x = get_x!(opcode);
                        let v_x = self.v[x];
                        self.v[0xF] = (v_x >> 7) & 0x1;
                        self.v[x] = v_x << 1;
                    }
                    _ => {
                        invalid_opcode!(opcode)
                    }
                }
            }
            0x9000 => {
                // (9xy0) - COND - skip if V_x != V_y
                match get_n!(opcode) {
                    0x0 => {
                        if self.v[get_x!(opcode)] != self.v[get_y!(opcode)] {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        invalid_opcode!(opcode);
                    }
                }
            }
            0xA000 => {
                // (Annn) - LDI - Load nnn into I
                self.i = get_nnn!(opcode);
            }
            0xB000 => {
                // (Bnnn) - JP V0, addr - Jump to V0 + addr
                self.pc = self.v[0] as u16 + get_nnn!(opcode);
            }
            0xC000 => {
                // (Cxkk) - RND Vx, byte - Bitwise and kk with random number [0,255]
                self.v[get_x!(opcode)] = (rand() % 256) as u8 & get_kk!(opcode);
            }
            0xD000 => {
                // (Dxyn) - DRW Vx, Vy, nibble
                let col = self.v[get_x!(opcode)];
                let row = self.v[get_y!(opcode)];
                let n = get_n!(opcode) as u8;
                let sprite_offset = self.i;
                self.v[0xF] = 0;
                for byte_ind in 0..n {
                    let sprite_byte = self.memory[(sprite_offset + byte_ind as u16) as usize];
                    for j in 0..8 {
                        let bit = (sprite_byte >> j) & 0x1;
                        let screen_x = (col + (7 - j)) % 64;
                        let screen_y = (row + byte_ind) % 32;

                        let mut screen_writer = self.screen.lock().unwrap();
                        let curr = &mut screen_writer[screen_y as usize][screen_x as usize];
                        if bit == 1 && *curr {
                            self.v[0xF] = 1;
                        }
                        let curr_val = match curr {
                            true => 1,
                            false => 0,
                        };
                        *curr = (curr_val ^ bit) == 1;
                    }
                }
            }
            0xE000 => {
                // User inputs
                match get_kk!(opcode) {
                    0x9E => {
                        // (Ex9E) - SKP Vx - Skip if V_x is pressed
                        if self.keyboard[self.v[get_x!(opcode)] as usize] {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        // (ExA1) - SKNP Vx - Skip if V_x isn't pressed
                        if !self.keyboard[self.v[get_x!(opcode)] as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        invalid_opcode!(opcode);
                    }
                }
            }
            0xF000 => {
                // Misc
                match get_kk!(opcode) {
                    0x07 => {
                        // (Fx07) - LD Vx, DT
                        self.v[get_x!(opcode)] = self.dt;
                    }
                    0x0A => {
                        // (Fx0A) - LD Vx, K
                        // Halt for input
                        match self.halt_for_input {
                            true => {
                                self.pc -= 2;
                            },
                            false => {
                                self.halt_for_input = true;
                            }
                        }
                    }
                    0x15 => {
                        // (Fx15) - LD DT, Vx
                        self.dt = self.v[get_x!(opcode)];
                    }
                    0x18 => {
                        // (Fx18) - LD ST, Vx
                        self.st = self.v[get_x!(opcode)];
                        if self.st > 0 {
                            // TODO: beeeeeep
                        }
                    }
                    0x1E => {
                        // (Fx1E) - ADD I, Vx
                        self.i += self.v[get_x!(opcode)] as u16;

                        // TODO: Add a flag for this?
                        // See: https://en.wikipedia.org/wiki/CHIP-8#cite_note-16
                    }
                    0x29 => {
                        // (Fx29) - LD F, Vx
                        self.i = (self.v[get_x!(opcode)] as u16) * 5 + 0x50;
                    }
                    0x33 => {
                        // (Fx33) - LD B, Vx
                        let v_x = self.v[get_x!(opcode)];
                        let i_usize = self.i as usize;
                        self.memory[i_usize] = (v_x as u16 / 100) as u8;
                        self.memory[i_usize + 1] = (v_x % 100) / 10;
                        self.memory[i_usize + 2] = v_x % 10;
                    }
                    0x55 => {
                        // (Fx55) - LD [I], Vx
                        let x = get_x!(opcode);
                        for i in 0..=x {
                            self.memory[self.i as usize + i] = self.v[i];
                        }
                    }
                    0x65 => {
                        // (Fx65) - LD Vx, [I]
                        let x = get_x!(opcode);
                        for i in 0..=x {
                            self.v[i] = self.memory[self.i as usize + i];
                        }
                    }
                    _ => {
                        invalid_opcode!(opcode);
                    }
                }
            }
            _ => {
                panic!("Invalid opcode: {:#?} ", opcode)
            }
        }

        Ok(1)
    }

    #[inline]
    fn fetch_opcode(&mut self) -> u16 {
        let byte1: u8 = self.memory[self.pc as usize];
        let byte2: u8 = self.memory[self.pc as usize + 1];

        self.pc += 2;
        ((byte1 as u16) << 8) | (byte2 as u16)
    }

    #[allow(dead_code)]
    fn inspect_opcode(&self, opcode: u16) {
        print!(
            "Opcode: {:#06x}, PC: {}, SP: {}, I: {}, ",
            opcode, self.pc, self.sp, self.i
        );
        print!("V [");
        for i in 0..16 {
            print!("{} ", self.v[i as usize])
        }
        print!("] ");
        println!();
    }

    // Should be called at a rate of 60Hz
    pub fn tick_timers(&mut self) -> (u8, u8) {
        if self.st > 0 {
            self.st -= 1;
        }
        if self.dt > 0 {
            self.dt -= 1
        }
        (self.st, self.dt)
    }

    pub fn get_state(&self) -> String {
        let mut s = format!(
            "PC: {:#X}\nSP: {:#X}\n I: {:#X}\nDT: {:#X}\nST: {:#X}",
            self.pc, self.sp, self.i, self.dt, self.st
        );

        let registers = self
            .v
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        s = format!("{}\nV: [{}]", s, registers);

        let stack = self
            .stack
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        s = format!("{}\nStack: [{}]", s, stack);
        s
    }

    // pub fn key_press(&mut self, key: Key) {
    //     self.keyboard[key as usize] = true;
    // }
    //
    // pub fn key_release(&mut self, key: Key) {
    //     self.keyboard[key as usize] = false;
    // }
    pub fn set_key_state(&mut self, key: Key, is_pressed: bool) {
        let cur_state = &mut self.keyboard[key as usize];
        if *cur_state != is_pressed {
            self.halt_for_input = false;
        }
        *cur_state = is_pressed;
    }

    pub fn reset_key_state(&mut self) {
        self.keyboard = [false; 16];
    }
}
