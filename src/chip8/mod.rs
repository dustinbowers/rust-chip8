use macroquad::rand::rand;
use quirks::Mode::*;
use quirks::Quirks;
use std::sync::{Arc, Mutex};

#[macro_use]
mod util;
pub mod quirks;
pub mod types;

pub const DISPLAY_ROWS: usize = 64;
pub const DISPLAY_COLS: usize = 128;

pub struct Chip8 {
    screen: Arc<Mutex<types::Screen>>,
    memory: Vec<u8>,     // [u8; 4096],
    stack: Vec<u16>,     // [u16; 16],
    keyboard: Vec<bool>, // [bool; 16],

    v: Vec<u8>,   // 16 8-bit registers (note VF is a carry-flag register) = [u8; 16]
    rpl: Vec<u8>, // 8 8-bit additional RPL flags in Super-Chip
    pc: u16,      // Program/Instruction counter
    i: u16,       // Index register
    sp: u16,      // Stack pointer
    dt: u8,       // Delay timer
    st: u8,       // Sound timer

    super_chip_enabled: bool,
    hires_mode: bool,
    halt_input_register: u8,
    halt_for_input: bool,
    wait_for_vblank: bool,
    quirks: Quirks,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut c = Self {
            screen: Arc::new(Mutex::new(vec![vec![false; DISPLAY_COLS]; DISPLAY_ROWS])),
            memory: vec![0u8; 4096],
            stack: vec![0u16; 16],
            keyboard: vec![false; 16],
            v: vec![0u8; 16],
            rpl: vec![0u8, 8],
            pc: 0x200,
            i: 0,
            sp: 0,
            dt: 0,
            st: 0,
            super_chip_enabled: true,
            hires_mode: false,
            halt_input_register: 0,
            halt_for_input: false,
            wait_for_vblank: false,
            quirks: Quirks::new(SuperChipModern),
        };
        c.load_font();
        return c;
    }

    pub fn get_quirks_mode(&self) -> &Quirks {
        return &self.quirks;
    }

    pub fn set_quirks_mode(&mut self, quirks: Quirks) {
        self.quirks = quirks;
    }

    fn load_font(&mut self) {
        for (i, v) in types::FONT_SET.iter().enumerate() {
            self.memory[i + types::FONT_OFFSET] = *v;
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

    pub fn get_screen(&self) -> Arc<Mutex<types::Screen>> {
        Arc::clone(&self.screen)
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
        let st = self.st;
        let dt = self.dt;
        if self.st > 0 {
            self.st -= 1;
        }
        if self.dt > 0 {
            self.dt -= 1
        }
        (st, dt)
    }

    pub fn v_blank(&mut self) {
        self.wait_for_vblank = false;
    }

    pub fn get_state(&self) -> String {
        let mut s = format!(
            "Opcode: {:#X}\nPC: {:#X}\nSP: {:#X}\nI: {:#X}\nDT: {:#X}\nST: {:#X}",
            self.fetch_opcode(),
            self.pc,
            self.sp,
            self.i,
            self.dt,
            self.st
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

        let keyboard = self
            .keyboard
            .iter()
            .map(|&x| match x {
                true => "X",
                false => "-",
            })
            .collect::<Vec<&str>>()
            .join(" ");
        s = format!("{}\nKeys: [{}]", s, keyboard);

        s = format!("{}\nhalt_for_input: {:?}", s, self.halt_for_input);
        s = format!("{}\nhires: {:?}", s, self.hires_mode);

        s
    }

    pub fn set_key_state(&mut self, key: types::Key, is_pressed: bool) {
        let cur_state = &mut self.keyboard[key as usize];

        if self.halt_for_input == true && *cur_state == true && is_pressed == false {
            self.v[self.halt_input_register as usize] = key as u8;
            self.halt_for_input = false;
        }
        *cur_state = is_pressed;
    }

    #[inline]
    fn fetch_opcode(&self) -> u16 {
        let byte1: u8 = self.memory[self.pc as usize];
        let byte2: u8 = self.memory[self.pc as usize + 1];

        ((byte1 as u16) << 8) | (byte2 as u16)
    }

    pub fn step(&mut self) -> Result<i32, ()> {
        if self.halt_for_input {
            return Ok(0);
        }
        if self.wait_for_vblank {
            return Ok(0);
        }
        let opcode = self.fetch_opcode();
        self.pc += 2;
        // self.inspect(opcode);

        match opcode & 0xF000 {
            0x0000 => {
                match get_kk!(opcode) {
                    0x00C0..=0x00CF => {
                        // 00CN*    Scroll display N lines down
                        ensure_super_chip!(self.super_chip_enabled);
                        let mut screen_writer = self.screen.lock().unwrap();
                        let n = get_n!(opcode) as usize;
                        let mut scroll_distance = n;

                        // SuperChip 'modern' low-res scrolling requires doubling
                        // See: https://github.com/Timendus/chip8-test-suite/blob/main/legacy-superchip.md#how-a-design-flaw-morphed-over-time
                        if self.hires_mode == false {
                            scroll_distance *= 2;
                        }

                        screen_writer.rotate_right(scroll_distance);
                        for i in 0..scroll_distance {
                            screen_writer[i] = vec![false; DISPLAY_COLS];
                        }
                    }
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
                    0x00FB => {
                        // 00FB*    Scroll display 4 pixels right
                        ensure_super_chip!(self.super_chip_enabled);
                        let mut screen_writer = self.screen.lock().unwrap();

                        // QUIRK: Scrolling in superchip lowres 'modern' (incorrectly) requires doubling.
                        //        In legacy, it doesn't
                        let mut scroll_distance = 4;
                        if self.hires_mode == false {
                            scroll_distance *= 2;
                        }

                        for row in 0..DISPLAY_ROWS {
                            screen_writer[row].rotate_right(scroll_distance);
                            for c in 0..scroll_distance {
                                screen_writer[row][c] = false;
                            }
                        }
                    }
                    0x00FC => {
                        // 00FC*    Scroll display 4 pixels left
                        ensure_super_chip!(self.super_chip_enabled);
                        let mut screen_writer = self.screen.lock().unwrap();

                        let mut scroll_distance = 4;
                        if self.hires_mode == false {
                            scroll_distance *= 2;
                        }

                        for row in 0..DISPLAY_ROWS {
                            screen_writer[row].rotate_left(scroll_distance);
                            for c in DISPLAY_COLS - scroll_distance..DISPLAY_COLS {
                                screen_writer[row][c] = false;
                            }
                        }
                    }
                    0x00FD => {
                        // 00FD*    Exit CHIP interpreter
                        ensure_super_chip!(self.super_chip_enabled);
                        todo!();
                    }
                    0x00FE => {
                        // 00FE*    Disable extended screen mode
                        ensure_super_chip!(self.super_chip_enabled);
                        self.hires_mode = false;
                    }
                    0x00FF => {
                        // 00FF*    Enable extended screen mode for full-screen graphics
                        ensure_super_chip!(self.super_chip_enabled);
                        self.hires_mode = true;
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
                if self.v[get_x!(opcode)] == get_kk!(opcode) {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // (4xkk) SNE Vx, byte - skip if not equal
                if self.v[get_x!(opcode)] != get_kk!(opcode) {
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
                let (s, _) = v_x.overflowing_add(get_kk!(opcode));
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
                        // reset Vf for Chip, not SCHIP
                        if self.quirks.vf_reset {
                            self.v[0xF] = 0;
                        }
                    }
                    0x2 => {
                        // (8xy2) - AND Vx, Vy - Compute V_x &= V_y
                        self.v[get_x!(opcode)] &= self.v[get_y!(opcode)];
                        if self.quirks.vf_reset {
                            self.v[0xF] = 0;
                        }
                    }
                    0x3 => {
                        // (8xy3) - XOR Vx, Vy - Compute V_x ^= V_y
                        self.v[get_x!(opcode)] ^= self.v[get_y!(opcode)];
                        if self.quirks.vf_reset {
                            self.v[0xF] = 0;
                        }
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
                        if self.quirks.shifting_vx == false {
                            let y = get_y!(opcode);
                            self.v[x] = self.v[y];
                        }
                        let v_x = self.v[x];
                        self.v[x] = v_x >> 1;
                        self.v[0xF] = v_x & 0x1;
                    }
                    0x7 => {
                        // (8xy7) - SUBN Vx, Vy - Compute V_x = V_y - V_x, set borrow-flag in VF
                        let x = get_x!(opcode);
                        let y = get_y!(opcode);
                        let (s, borrow) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[x] = s;
                        self.v[0xF] = match borrow {
                            true => 0,
                            false => 1,
                        };
                    }
                    0xE => {
                        // (8xyE) - SHL Vx - Computer V_x <<= 1,
                        let x = get_x!(opcode);
                        if self.quirks.shifting_vx == false {
                            let y = get_y!(opcode);
                            self.v[x] = self.v[y];
                        }
                        let v_x = self.v[x];
                        self.v[x] = v_x << 1;
                        self.v[0xF] = (v_x >> 7) & 0x1;
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
                // (Bnnn) - JP V0, addr - Jump to V0 (or Vx) + addr
                if self.quirks.jump_plus_vx {
                    self.pc = self.v[get_x!(opcode)] as u16 + get_nnn!(opcode);
                } else {
                    self.pc = self.v[0] as u16 + get_nnn!(opcode);
                }
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
                self.draw_sprite(col, row, n);

                if self.quirks.display_wait {
                    self.wait_for_vblank = true;
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
                        // (Fx0A) - LD Vx, K - Halt for input, then store it in Vx
                        let x = get_x!(opcode);
                        self.halt_input_register = x as u8;
                        self.halt_for_input = true;
                    }
                    0x15 => {
                        // (Fx15) - LD DT, Vx
                        self.dt = self.v[get_x!(opcode)];
                    }
                    0x18 => {
                        // (Fx18) - LD ST, Vx
                        self.st = self.v[get_x!(opcode)];
                    }
                    0x1E => {
                        // (Fx1E) - ADD I, Vx
                        self.i += self.v[get_x!(opcode)] as u16;
                    }
                    0x29 => {
                        // (Fx29) - LD F, Vx
                        self.i = (self.v[get_x!(opcode)] as u16) * 5 + 0x50;
                    }
                    0x30 => {
                        // FX30*    Point I to 10-byte font sprite for digit VX (0..9)
                        ensure_super_chip!(self.super_chip_enabled);
                        let v_x = self.v[get_x!(opcode)];
                        self.i = ((types::FONT_OFFSET + 80) + (v_x as usize * 10)) as u16
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
                        // (Fx55) - LD [I], Vx - Store V0..VX in memory starting at i
                        let x = get_x!(opcode);
                        for i in 0..=x {
                            self.memory[self.i as usize + i] = self.v[i];
                        }
                        if self.quirks.load_store_index_increase {
                            self.i += x as u16 + 1;
                        }
                    }
                    0x65 => {
                        // (Fx65) - LD Vx, [I] - Load V0..VX in memory starting at i
                        let x = get_x!(opcode);
                        for i in 0..=x {
                            self.v[i] = self.memory[self.i as usize + i];
                        }
                        if self.quirks.load_store_index_increase {
                            self.i += x as u16 + 1;
                        }
                    }
                    0x75 => {
                        // FX75*    Store V0..VX in RPL user flags (X <= 7)
                        ensure_super_chip!(self.super_chip_enabled);
                        let x = get_x!(opcode);
                        if x > 8 {
                            invalid_opcode!(opcode);
                        }
                        self.rpl[x] = self.v[x];
                    }
                    0x85 => {
                        // FX85*    Read V0..VX from RPL user flags (X <= 7)
                        ensure_super_chip!(self.super_chip_enabled);
                        let x = get_x!(opcode);
                        if x > 8 {
                            invalid_opcode!(opcode);
                        }
                        self.v[x] = self.rpl[x];
                    }
                    _ => {
                        invalid_opcode!(opcode);
                    }
                }
            }
            _ => {
                invalid_opcode!(opcode);
            }
        }
        Ok(1)
    }

    fn draw_sprite(&mut self, col: u8, row: u8, n: u8) {
        let mut screen_writer = self.screen.lock().unwrap();
        let sprite_offset = self.i;
        self.v[0xF] = 0;
        if n == 0 && self.hires_mode {
            // draw a SuperChip 16x16 sprite
            for r in 0..16u16 {
                let mem_loc = (sprite_offset + (2 * r)) as usize;
                let sprite_word =
                    (self.memory[mem_loc] as u16) << 8 | self.memory[mem_loc + 1] as u16;
                for c in 0..16 {
                    let bit = (sprite_word >> c) & 0x1 == 1;
                    let mut screen_x = col % DISPLAY_COLS as u8;
                    let mut screen_y = row % DISPLAY_ROWS as u8;
                    screen_x += 16 - c;
                    screen_y += r as u8;
                    if self.quirks.clipping {
                        if screen_x as usize >= DISPLAY_COLS {
                            continue;
                        }
                        if screen_y as usize >= DISPLAY_ROWS {
                            continue;
                        }
                    }
                    let curr = &mut screen_writer[screen_y as usize][screen_x as usize];
                    if bit && *curr {
                        self.v[0xF] = 1;
                    }
                    *curr ^= bit;
                }
            }
        } else {
            for byte_ind in 0..n {
                let sprite_byte = self.memory[(sprite_offset + byte_ind as u16) as usize];

                //draw a Chip8 8xN sprite
                for j in 0..8 {
                    let bit = (sprite_byte >> j) & 0x1 == 1;

                    if self.hires_mode {
                        let mut screen_x = col % DISPLAY_COLS as u8;
                        let mut screen_y = row % DISPLAY_ROWS as u8;
                        screen_x += 7 - j;
                        screen_y += byte_ind;

                        if self.quirks.clipping {
                            if screen_x as usize >= DISPLAY_COLS {
                                continue;
                            }
                            if screen_y as usize >= DISPLAY_ROWS {
                                continue;
                            }
                        }

                        let curr = &mut screen_writer[screen_y as usize][screen_x as usize];
                        if bit && *curr {
                            self.v[0xF] = 1;
                        }
                        *curr ^= bit;
                    } else {
                        let mut screen_x = col % (DISPLAY_COLS / 2) as u8;
                        let mut screen_y = row % (DISPLAY_ROWS / 2) as u8;

                        screen_x += 7 - j;
                        screen_y += byte_ind;

                        if self.quirks.clipping {
                            if screen_x as usize >= DISPLAY_COLS / 2 {
                                continue;
                            }
                            if screen_y as usize >= DISPLAY_ROWS / 2 {
                                continue;
                            }
                        }

                        screen_x *= 2;
                        screen_y *= 2;

                        // let mut screen_writer = self.screen.lock().unwrap();
                        for i in 0..2u8 {
                            for j in 0..2u8 {
                                let curr = &mut screen_writer[(screen_y as u8 + j) as usize]
                                    [(screen_x as u8 + i) as usize];
                                if bit && *curr {
                                    self.v[0xF] = 1;
                                }
                                *curr ^= bit;
                            }
                        }
                    }
                }
            }
        }
    }
}
