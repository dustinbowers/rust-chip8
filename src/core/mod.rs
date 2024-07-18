use crate::core::error::CoreErrorType::*;
use crate::core::error::*;
use macroquad::rand::rand;
use quirks::Mode::*;
use quirks::Quirks;
use std::sync::{Arc, Mutex};

#[macro_use]
mod util;
pub mod error;
pub mod quirks;
pub mod types;

pub const DISPLAY_ROWS: usize = 64;
pub const DISPLAY_COLS: usize = 128;
pub const DISPLAY_LAYERS: usize = 2;

macro_rules! err_info {
    () => {
        format!("{}, line: {}", file!(), line!())
    };
}

pub struct Sound {
    pub pitch: u8,
    pub pattern: Vec<u8>,
    dirty: bool,
}
impl Sound {
    pub fn new() -> Self {
        Self {
            pitch: 247,
            pattern: vec![
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff,
            ],
            dirty: true,
        }
    }
}
pub struct Chip8 {
    screen: Arc<Mutex<types::Screen>>,
    memory: Vec<u8>,     // [u8; 2^16],
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
    halted_for_input: bool,
    waiting_for_vblank: bool,
    quirks: Quirks,
    sound: Sound,
    bit_plane_selector: u8,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut c = Self {
            screen: Arc::new(Mutex::new(vec![
                vec![
                    vec![false; DISPLAY_LAYERS];
                    DISPLAY_COLS
                ];
                DISPLAY_ROWS
            ])),
            memory: vec![0u8; 1 << 16],
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
            halted_for_input: false,
            waiting_for_vblank: false,
            quirks: Quirks::new(XoChip),
            sound: Sound::new(),
            bit_plane_selector: 1,
        };
        c.load_font();
        return c;
    }

    // TODO: remove this
    pub fn chaos(&mut self) {
        // Move the PC to a random location and let the chaos begin
        let r = ((rand() % 128) * 2 + 0x200) as u16;
        self.pc = r;
    }

    pub fn quirks_mode(&self) -> &Quirks {
        return &self.quirks;
    }

    pub fn set_quirks_mode(&mut self, quirks: Quirks) {
        self.quirks = quirks;
    }

    pub fn set_core_mode(&mut self, mode: &String) {
        let mode = mode.to_lowercase();
        match mode.as_str() {
            "chip8modern" | "chip8" => self.quirks = Quirks::new(Chip8Modern),
            "superchipmodern" | "superchip" => self.quirks = Quirks::new(SuperChipModern),
            "superchiplegacy" => self.quirks = Quirks::new(SuperChipLegacy),
            "xochip" => self.quirks = Quirks::new(XoChip),
            _ => {
                // TODO: handle this more gracefully
                panic!("Unknown core mode: {}", mode.as_str());
            }
        };
    }

    fn load_font(&mut self) {
        for (i, v) in types::FONT_SET.iter().enumerate() {
            self.memory[i + types::FONT_OFFSET] = *v;
        }
    }

    pub fn load_rom(&mut self, bytes: Vec<u8>, start_offset: u16) -> Result<usize, CoreError> {
        let start_offset = start_offset as usize;
        if bytes.len() + start_offset >= 1 << 16 {
            return Err(CoreError::new(
                err_info!(),
                InvalidRom(format!(
                    "Rom byte size {} + start_offset > 1<<16",
                    bytes.len()
                )),
            ));
        }
        for (i, v) in bytes.iter().enumerate() {
            self.memory[i + start_offset] = *v;
        }
        Ok(bytes.len())
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

    pub fn get_sound(&mut self) -> Option<&Sound> {
        match self.sound.dirty {
            true => {
                self.sound.dirty = false;
                Some(&self.sound)
            }
            false => None,
        }
    }

    pub fn v_blank(&mut self) {
        self.waiting_for_vblank = false;
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

        s = format!("{}\nhalt_for_input: {:?}", s, self.halted_for_input);
        s = format!("{}\nhires: {:?}", s, self.hires_mode);
        s = format!(
            "{}\nbit_plane_select: 0b{:04b} ({:?})",
            s, self.bit_plane_selector, self.bit_plane_selector
        );

        s
    }

    pub fn set_key_state(&mut self, key: types::Key, is_pressed: bool) {
        let cur_state = &mut self.keyboard[key as usize];

        if self.halted_for_input == true && *cur_state == true && is_pressed == false {
            self.v[self.halt_input_register as usize] = key as u8;
            self.halted_for_input = false;
        }
        *cur_state = is_pressed;
    }

    #[inline]
    fn fetch_opcode(&self) -> u16 {
        let byte1: u8 = self.memory[self.pc as usize];
        let byte2: u8 = self.memory[self.pc as usize + 1];

        ((byte1 as u16) << 8) | (byte2 as u16)
    }

    #[inline]
    fn skip_opcode(&mut self) {
        // XO-Chip support: skip ahead 2 opcodes if the double-width opcode 0xF000 is next
        if self.fetch_opcode() == 0xF000 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    pub fn step(&mut self) -> Result<i32, CoreError> {
        if self.halted_for_input {
            return Ok(1);
        }
        if self.waiting_for_vblank {
            return Ok(1);
        }
        let opcode = self.fetch_opcode();
        self.pc += 2;

        match opcode & 0xF000 {
            0x0000 => {
                match get_nnn!(opcode) {
                    0x00C0..=0x00CF => {
                        // Note: technically, 00C0 causes a crash on hardware
                        //       but xo-chip treats it as a nop... so I'm leaving it

                        // (00CN)*    Scroll display N lines down
                        ensure_super_chip!(self.super_chip_enabled);
                        let n = get_n!(opcode) as usize;
                        let mut scroll_distance = n;

                        // SuperChip 'modern' low-res scrolling requires doubling
                        // See: https://github.com/Timendus/chip8-test-suite/blob/main/legacy-superchip.md#how-a-design-flaw-morphed-over-time
                        if self.hires_mode == false {
                            scroll_distance *= 2;
                        }

                        for layer in 0..DISPLAY_LAYERS {
                            if (self.bit_plane_selector >> layer) & 0b1 == 1 {
                                self.scroll_plane_down(scroll_distance, layer);
                            }
                        }
                    }
                    0x00D0..=0x00DF => {
                        // XO-CHIP: (00DN) Scroll display N lines up

                        let n = get_n!(opcode) as usize;
                        let mut scroll_distance = n;

                        // SuperChip 'modern' low-res scrolling requires doubling
                        // See: https://github.com/Timendus/chip8-test-suite/blob/main/legacy-superchip.md#how-a-design-flaw-morphed-over-time
                        if self.hires_mode == false {
                            scroll_distance *= 2;
                        }

                        for layer in 0..DISPLAY_LAYERS {
                            if (self.bit_plane_selector >> layer) & 0b1 == 1 {
                                self.scroll_plane_up(scroll_distance, layer);
                            }
                        }
                    }
                    0x00E0 => {
                        // CLS
                        for layer in 0..DISPLAY_LAYERS {
                            if (self.bit_plane_selector >> layer) & 0b1 == 1 {
                                self.clear_layer(layer);
                            }
                        }
                    }
                    0x00EE => {
                        // RET
                        if self.sp == 0 {
                            return Err(CoreError::new(
                                err_info!(),
                                StackOverflow(self.pc, self.sp),
                            ));
                        }
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                    }
                    0x00FB => {
                        // 00FB*    Scroll display 4 pixels right
                        ensure_super_chip!(self.super_chip_enabled);

                        for layer in 0..DISPLAY_LAYERS {
                            if (self.bit_plane_selector >> layer) & 0b1 == 1 {
                                self.scroll_layer_right(layer);
                            }
                        }
                    }
                    0x00FC => {
                        // 00FC*    Scroll display 4 pixels left
                        ensure_super_chip!(self.super_chip_enabled);

                        for layer in 0..DISPLAY_LAYERS {
                            if (self.bit_plane_selector >> layer) & 0b1 == 1 {
                                self.scroll_layer_left(layer);
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
                        for layer in 0..DISPLAY_LAYERS {
                            self.clear_layer(layer);
                        }
                    }
                    0x00FF => {
                        // 00FF*    Enable extended screen mode
                        ensure_super_chip!(self.super_chip_enabled);
                        self.hires_mode = true;
                        for layer in 0..DISPLAY_LAYERS {
                            self.clear_layer(layer);
                        }
                    }
                    _ => {
                        return Err(CoreError::new(err_info!(), InvalidOpcode(self.pc, opcode)));
                    }
                }
            }
            0x1000 => {
                // (1nnn) JMP addr
                self.pc = get_nnn!(opcode);
            }
            0x2000 => {
                // (2nnn) CALL addr
                if self.sp > 15 {
                    return Err(CoreError::new(err_info!(), StackOverflow(self.pc, self.sp)));
                }
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = get_nnn!(opcode);
            }
            0x3000 => {
                // (3xkk) SE Vx, byte - skip if equal
                if self.v[get_x!(opcode)] == get_kk!(opcode) {
                    self.skip_opcode();
                }
            }
            0x4000 => {
                // (4xkk) SNE Vx, byte - skip if not equal
                if self.v[get_x!(opcode)] != get_kk!(opcode) {
                    self.skip_opcode()
                }
            }
            0x5000 => {
                match get_n!(opcode) {
                    0x0 => {
                        // (5xy0) - SE Vx, Vy - skip if registers are equal
                        if self.v[get_x!(opcode)] == self.v[get_y!(opcode)] {
                            self.skip_opcode();
                        }
                    }
                    0x2 => {
                        // XO-CHIP: (5xy2) - write registers vX to vY to memory pointed to by I
                        let x = get_x!(opcode);
                        let y = get_y!(opcode);
                        let dist = x.abs_diff(y);

                        if x + dist > self.memory.len() - 1 {
                            return Err(CoreError::new(
                                err_info!(),
                                InvalidMemoryAccess(self.pc, self.i as usize + x + dist),
                            ));
                        }
                        if x < y {
                            for z in 0..dist + 1 {
                                self.memory[self.i as usize + z] = self.v[x + z];
                            }
                        } else {
                            for z in 0..dist + 1 {
                                self.memory[self.i as usize + z] = self.v[x - z];
                            }
                        }
                    }
                    0x3 => {
                        // XO-CHIP: (5xy3) - load registers vX to vY from memory pointed to by I
                        let x = get_x!(opcode);
                        let y = get_y!(opcode);
                        let dist = x.abs_diff(y);
                        if x + dist > self.memory.len() - 1 {
                            return Err(CoreError::new(
                                err_info!(),
                                InvalidMemoryAccess(self.pc, self.i as usize + x + dist),
                            ));
                        }
                        if x < y {
                            for z in 0..dist + 1 {
                                self.v[x + z] = self.memory[self.i as usize + z];
                            }
                        } else {
                            for z in 0..dist + 1 {
                                self.v[x - z] = self.memory[self.i as usize + z];
                            }
                        }
                    }
                    _ => {
                        return Err(CoreError::new(err_info!(), InvalidOpcode(self.pc, opcode)));
                    }
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
                        return Err(CoreError::new(err_info!(), InvalidOpcode(self.pc, opcode)));
                    }
                }
            }
            0x9000 => {
                // (9xy0) - COND - skip if V_x != V_y
                match get_n!(opcode) {
                    0x0 => {
                        if self.v[get_x!(opcode)] != self.v[get_y!(opcode)] {
                            self.skip_opcode();
                        }
                    }
                    _ => {
                        return Err(CoreError::new(err_info!(), InvalidOpcode(self.pc, opcode)));
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
                self.v[get_x!(opcode)] = rand() as u8 & get_kk!(opcode);
            }
            0xD000 => {
                // (Dxyn) - DRW Vx, Vy, nibble
                let col = self.v[get_x!(opcode)];
                let row = self.v[get_y!(opcode)];
                let n = get_n!(opcode) as u8;

                let mut page_num = 0;
                self.v[0xF] = 0;
                for layer in 0..DISPLAY_LAYERS {
                    if (self.bit_plane_selector >> layer) & 0b1 == 1 {
                        self.draw_sprite(col, row, n, page_num, layer)?;
                        page_num += 1;
                    }
                }

                if self.quirks.display_wait {
                    self.waiting_for_vblank = true;
                }
            }
            0xE000 => {
                // User inputs
                match get_kk!(opcode) {
                    0x9E => {
                        // (Ex9E) - SKP Vx - Skip if V_x is pressed
                        let x = get_x!(opcode);
                        if self.keyboard[(self.v[x] & 0xF) as usize] {
                            self.skip_opcode();
                        }
                    }
                    0xA1 => {
                        // (ExA1) - SKNP Vx - Skip if V_x isn't pressed
                        let x = get_x!(opcode);
                        if !self.keyboard[(self.v[x] & 0xF) as usize] {
                            self.skip_opcode();
                        }
                    }
                    _ => {
                        return Err(CoreError::new(err_info!(), InvalidOpcode(self.pc, opcode)));
                    }
                }
            }
            0xF000 => {
                match get_nnn!(opcode) {
                    0x000 => {
                        // XO-CHIP Support: (0xF000) - assign next 16 bit word to i
                        let byte1 = self.memory[self.pc as usize] as u16;
                        let byte2 = self.memory[self.pc as usize + 1] as u16;
                        self.i = (byte1 << 8) | byte2;
                        self.pc += 2;
                    }
                    0x002 => {
                        // XO-CHIP Support: (0xF002) - load 16 bytes audio pattern pointed to by I into audio pattern buffer
                        if self.i as usize + 15 > self.memory.len() - 1 {
                            return Err(CoreError::new(
                                err_info!(),
                                InvalidMemoryAccess(self.pc, self.i as usize + 15),
                            ));
                        }
                        for offset in 0..16 {
                            self.sound.pattern[offset] = self.memory[self.i as usize + offset];
                        }
                        self.sound.dirty = true;
                    }
                    _ => {
                        // Misc (Fx--)
                        match get_kk!(opcode) {
                            0x01 => {
                                // XO-Chip Support: (0xFX01) - select bit planes to draw on when drawing with Dxy0/Dxyn
                                self.bit_plane_selector = get_x!(opcode) as u8;
                            }
                            0x07 => {
                                // (Fx07) - LD Vx, DT
                                self.v[get_x!(opcode)] = self.dt;
                            }
                            0x0A => {
                                // (Fx0A) - LD Vx, K - Halt for input, then store it in Vx
                                let x = get_x!(opcode);
                                self.halt_input_register = x as u8;
                                self.halted_for_input = true;
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
                                let v_x = self.v[get_x!(opcode)] as u16;
                                if self.i as usize > self.memory.len() - (v_x as usize) {
                                    return Err(CoreError::new(
                                        err_info!(),
                                        InvalidMemoryPtr(self.pc, self.i as usize + v_x as usize),
                                    ));
                                }
                                self.i += v_x;
                            }
                            0x29 => {
                                // (Fx29) - LD F, Vx
                                let v_x = self.v[get_x!(opcode)] & 0xF;
                                self.i = (v_x as u16) * 5 + 0x50;
                            }
                            0x30 => {
                                // FX30*    Point I to 10-byte font sprite for digit VX (0..9)
                                ensure_super_chip!(self.super_chip_enabled);
                                let v_x = self.v[get_x!(opcode)] & 0xF;
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
                            0x3A => {
                                // XO-CHIP Support: (0xFX3a) - set audio pitch
                                let x = self.v[get_x!(opcode)];
                                self.sound.pitch = x;
                            }
                            0x55 => {
                                // (Fx55) - LD [I], Vx - Store V0..VX in memory starting at i
                                let x = get_x!(opcode);
                                if 0xFFFF - self.i < x as u16 {
                                    return Err(CoreError::new(
                                        err_info!(),
                                        InvalidMemoryPtr(self.pc, self.i as usize),
                                    ));
                                }
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
                                if 0xFFFF - self.i < x as u16 {
                                    return Err(CoreError::new(
                                        err_info!(),
                                        InvalidMemoryPtr(self.pc, self.i as usize),
                                    ));
                                }
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
                                    return Err(CoreError::new(
                                        err_info!(),
                                        InvalidOpcode(self.pc, opcode),
                                    ));
                                }
                                self.rpl[x] = self.v[x];
                            }
                            0x85 => {
                                // FX85*    Read V0..VX from RPL user flags (X <= 7)
                                ensure_super_chip!(self.super_chip_enabled);
                                let x = get_x!(opcode);
                                if x > 8 {
                                    return Err(CoreError::new(
                                        err_info!(),
                                        InvalidOpcode(self.pc, opcode),
                                    ));
                                }
                                self.v[x] = self.rpl[x];
                            }
                            _ => {
                                return Err(CoreError::new(
                                    err_info!(),
                                    InvalidOpcode(self.pc, opcode),
                                ));
                            }
                        }
                    }
                }
            }
            _ => {
                return Err(CoreError::new(err_info!(), InvalidOpcode(self.pc, opcode)));
            }
        }
        Ok(1)
    }

    fn clear_layer(&mut self, layer: usize) {
        let mut screen_writer = self.screen.lock().unwrap();
        for row in screen_writer.iter_mut() {
            for cell in row.iter_mut() {
                (*cell)[layer] = false;
            }
        }
    }

    fn scroll_plane_up(&mut self, scroll_distance: usize, layer: usize) {
        let mut screen_writer = self.screen.lock().unwrap();
        if layer >= DISPLAY_LAYERS {
            panic!("invalid layer index: {}", layer);
        }
        for r in 0..DISPLAY_ROWS - scroll_distance {
            for c in 0..DISPLAY_COLS {
                screen_writer[r][c][layer] = screen_writer[r + scroll_distance][c][layer];
            }
        }
        for i in DISPLAY_ROWS - scroll_distance..DISPLAY_ROWS {
            for c in 0..DISPLAY_COLS {
                screen_writer[i][c][layer] = false;
            }
        }
    }

    fn scroll_plane_down(&mut self, scroll_distance: usize, layer: usize) {
        let mut screen_writer = self.screen.lock().unwrap();
        if layer >= DISPLAY_LAYERS {
            panic!("invalid layer index: {}", layer);
        }
        for r in (scroll_distance..DISPLAY_ROWS).rev() {
            for c in 0..DISPLAY_COLS {
                screen_writer[r][c][layer] = screen_writer[r - scroll_distance][c][layer];
            }
        }
        for i in 0..scroll_distance {
            for c in 0..DISPLAY_COLS {
                screen_writer[i][c][layer] = false;
            }
        }
    }

    fn scroll_layer_left(&mut self, layer: usize) {
        let mut screen_writer = self.screen.lock().unwrap();

        let mut scroll_distance = 4;
        if self.hires_mode == false {
            scroll_distance *= 2;
        }

        for row in 0..DISPLAY_ROWS {
            for c in scroll_distance..DISPLAY_COLS {
                screen_writer[row][c - scroll_distance][layer] = screen_writer[row][c][layer];
            }
            for c in DISPLAY_COLS - scroll_distance..DISPLAY_COLS {
                screen_writer[row][c][layer] = false;
            }
        }
    }

    fn scroll_layer_right(&mut self, layer: usize) {
        // QUIRK: Scrolling in superchip lowres 'modern' (incorrectly) requires doubling.
        //        In legacy, it doesn't
        let mut scroll_distance = 4;
        if self.hires_mode == false {
            scroll_distance *= 2;
        }

        let mut screen_writer = self.screen.lock().unwrap();
        for row in 0..DISPLAY_ROWS {
            for c in (scroll_distance..DISPLAY_COLS).rev() {
                screen_writer[row][c][layer] = screen_writer[row][c - scroll_distance][layer];
            }
            for c in 0..scroll_distance {
                screen_writer[row][c][layer] = false;
            }
        }
    }

    fn draw_sprite(
        &mut self,
        col: u8,
        row: u8,
        sprite_rows: u8,
        page_num: usize,
        layer: usize,
    ) -> Result<(), CoreError> {
        let mut screen_writer = self.screen.lock().unwrap();
        let sprite_offset = self.i as usize;
        if sprite_rows == 0 && self.hires_mode {
            // draw a SuperChip 16x16 sprite
            let page_size = 2 * 16;
            for r in 0..16u16 {
                let mem_loc = sprite_offset + (page_num * page_size) + (2 * r as usize);
                if mem_loc > self.memory.len() - 2 {
                    return Err(CoreError::new(
                        err_info!(),
                        InvalidMemoryAccess(self.pc, mem_loc),
                    ));
                }
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
                    screen_x %= DISPLAY_COLS as u8;
                    screen_y %= DISPLAY_ROWS as u8;
                    let curr = &mut screen_writer[screen_y as usize][screen_x as usize][layer];
                    if bit && *curr {
                        self.v[0xF] = 1;
                    }
                    *curr ^= bit;
                }
            }
        } else {
            let page_size = sprite_rows as usize;
            for byte_ind in 0..sprite_rows {
                let sprite_offset = sprite_offset + (page_num * page_size) + byte_ind as usize;
                let sprite_byte = self.memory[sprite_offset];

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

                        let curr = &mut screen_writer[screen_y as usize % DISPLAY_ROWS]
                            [screen_x as usize % DISPLAY_COLS][layer];
                        if bit && (*curr) {
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
                                let curr = &mut screen_writer
                                    [((screen_y + j) % (DISPLAY_ROWS as u8)) as usize]
                                    [((screen_x + i) % (DISPLAY_COLS as u8)) as usize][layer];
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
        Ok(())
    }
}
