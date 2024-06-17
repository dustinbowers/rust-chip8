use std::sync::{Arc, Mutex};
use macroquad::prelude::*;

pub mod chip8;
use chip8::Chip8;

const WINDOW_HEIGHT: i32 = 256;
const WINDOW_WIDTH: i32 = 512;
const PIXEL_WIDTH: f32 = WINDOW_WIDTH as f32 / 64.0;
const PIXEL_HEIGHT: f32 = WINDOW_HEIGHT as f32 / 32.0;

struct Display {
    window_size: (u16, u16),
    // screen: &'a chip8::Screen,
    screen: Arc<Mutex<chip8::Screen>>,
    pub pixels: [[u8; 64]; 32],
}

impl Display {

    pub fn new(window_size: (u16, u16), screen: Arc<Mutex<chip8::Screen>>) -> Display {
        Display {
            window_size,
            screen,
            pixels: [[0u8; 64]; 32],
        }
    }

    pub fn update(&mut self) {
        let mut reader = self.screen.lock().unwrap();

        for (ci, r) in reader.iter().enumerate() {
            for (ri, cell) in r.iter().enumerate() {
                let mut cur_val: i16 = self.pixels[ci][ri] as i16;
                match *cell {
                    true => { cur_val += 200 },
                    false => { cur_val -= 15 },
                }
                self.pixels[ci][ri] = cur_val.clamp(0, 255) as u8;
            }
        }

    }

    pub fn draw(self) {

    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Chip 8".to_owned(),
        fullscreen: false,
        window_width: 512,
        window_height: 256,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut chip = Chip8::new();
    let mut display = Display::new((640, 480), chip.get_screen());

    display.update();

    // println!("Chip8:\n{:#?}", chip);
    let mut cycles = 0;
    loop {
        cycles += 1;
        if cycles > 0 {
            cycles = 0;
            if let Ok(cycles) = chip.step() {
                // println!("cycles: {}", cycles);
            }
        }

        // draw screen
        display.update();
        clear_background(BLACK);
        for (ri, c) in display.pixels.iter().enumerate() {
            for (ci, block) in  c.iter().enumerate() {
                let b = *block;
                let color = color_u8!(b,b,b,255);
                let x = ci as f32 * PIXEL_WIDTH;
                let y = ri as f32 * PIXEL_HEIGHT;
                draw_rectangle(x, y, PIXEL_WIDTH, PIXEL_HEIGHT, color);
                if b > 0 {
                    // println!("drawing - x: {}, y: {}, c: {:?}", x, y, color);
                }
            }
        }

    // TODO:

        next_frame().await
    }
}
