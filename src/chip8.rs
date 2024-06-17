use std::fmt::Error;
use std::sync::{Arc, Mutex};
use macroquad::rand::rand;

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

#[derive(Debug)]
pub struct Chip8 {
    screen: Arc<Mutex<Screen>>,
    memory: [u8; 4096],
    stack: [u16; 16],
    keyboard: [bool; 16],

    schip_mode: bool,

    v: [u8; 16], // 16 8-bit registers (note VF is a carry-flag register)
    pc: u16,     // Program/Instruction counter
    i: u16,      // Index register
    sp: u16,     // Stack pointer
    dt: u8,          // Delay timer
    st: u8,          // Sound timer
    draw_flag: bool, // Redraw when true
}

impl Chip8 {
    pub fn new() -> Self {
        let mut c = Self {
            screen: Arc::new(Mutex::new([[false; 64]; 32])),
            memory: [0u8; 4096],
            stack: [0u16; 16],
            keyboard: [false; 16],
            schip_mode: false,
            v: [0u8; 16],
            pc: 0,
            i: 0,
            sp: 0,
            dt: 0,
            st: 0,
            draw_flag: false,
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
        self.screen.clone()
    }

    pub fn step(&mut self) -> Result<i32, ()>{
        let x = rand() % 64;
        let y = rand() % 32;
        let r = (rand() % 100) >= 50;

        let mut screen_writer = self.screen.lock().unwrap();
        screen_writer[y as usize][x as usize] = !screen_writer[y as usize][x as usize];

        // println!("x: {}, y: {}, c = {}", x, y, r);
       Ok(1)
    }


}