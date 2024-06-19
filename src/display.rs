use crate::chip8;
use std::sync::{Arc, Mutex};

pub struct Display {
    pub screen: Arc<Mutex<chip8::Screen>>,
    pub pixels: [[u8; 64]; 32],
}

impl Display {
    pub fn new(screen: Arc<Mutex<chip8::Screen>>) -> Display {
        Display {
            screen,
            pixels: [[0u8; 64]; 32],
        }
    }

    pub fn update(&mut self) {
        let reader = self.screen.lock().unwrap();

        for (ci, r) in reader.iter().enumerate() {
            for (ri, cell) in r.iter().enumerate() {
                let mut cur_val: i16 = self.pixels[ci][ri] as i16;
                match *cell {
                    true => cur_val += 200,
                    false => cur_val -= 25,
                }
                self.pixels[ci][ri] = cur_val.clamp(0, 255) as u8;
            }
        }
    }
}
