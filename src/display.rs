use crate::core;
use std::sync::{Arc, Mutex};

pub struct Display {
    pub screen: Arc<Mutex<core::types::Screen>>,
    pub pixels: Vec<Vec<u8>>,
}

impl Display {
    pub fn new(screen: Arc<Mutex<core::types::Screen>>, rows: usize, cols: usize) -> Display {
        Display {
            screen,
            pixels: vec![vec![0u8; cols]; rows],
        }
    }

    pub fn update(&mut self) {
        let reader = self.screen.lock().unwrap();

        for (ci, r) in reader.iter().enumerate() {
            for (ri, cell) in r.iter().enumerate() {
                let mut cur_val: i16 = self.pixels[ci][ri] as i16;
                match (*cell)[0] {
                    true => cur_val += 255,
                    false => {
                        let mut new_val = cur_val as f32;
                        new_val -= new_val * 0.15;
                        cur_val = new_val as i16
                    }
                }
                self.pixels[ci][ri] = cur_val.clamp(0, 255) as u8;
            }
        }
    }
}
