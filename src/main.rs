use macroquad::prelude::*;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{fs, io};

mod chip8;

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
        let reader = self.screen.lock().unwrap();

        for (ci, r) in reader.iter().enumerate() {
            for (ri, cell) in r.iter().enumerate() {
                // match cell {
                // true => { print!("#"); },
                // false => { print!(" "); }
                // }
                let mut cur_val: i16 = self.pixels[ci][ri] as i16;
                match *cell {
                    true => cur_val += 200,
                    false => cur_val -= 15,
                }
                self.pixels[ci][ri] = cur_val.clamp(0, 255) as u8;
            }
            // println!();
        }
    }

    pub fn draw(self) {}
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

fn load_rom_file(filename: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(filename)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

enum DrawMethod {
    RAW,
    REAL,
}

#[macroquad::main(window_conf)]
async fn main() {
    const DRAW_METHOD: DrawMethod = DrawMethod::REAL;
    let debug_draw: bool = true;

    // let filename = "./roms/programs/BC_test.ch8";
    // let filename = "./roms/programs/IBM Logo.ch8";
    let filename = "./roms/games/Space Invaders [David Winter].ch8";
    let rom = load_rom_file(filename);

    let rom = match rom {
        Ok(rom) => rom,
        Err(e) => {
            println!("Error loading file: {:#?}", e);
            return;
        }
    };
    println!("Loaded {} bytes from file: {}", rom.len(), filename);

    let mut chip = Chip8::new();
    _ = chip.load_rom(rom, 0x200);
    let mut display = Display::new((640, 480), chip.get_screen());

    display.update();

    // println!("Chip8:\n{:#?}", chip);
    // let mut cycles = 0;
    // loop {
    //     cycles += 1;
    //     if cycles > 0 {
    //         cycles = 0;
    //         if let Ok(cycles) = chip.step() {
    //             // println!("cycles: {}", cycles);
    //         }
    //     }
    // Time per frame at 60 Hz
    let frame_duration = Duration::from_secs_f64(1.0 / 60.0);
    // Time per step at 700 Hz
    let step_duration = Duration::from_secs_f64(1.0 / 700.0);

    // Initialize the last step time
    let mut last_step_time = Instant::now();

    loop {
        display.update();
        // draw screen
        clear_background(GRAY);
        match DRAW_METHOD {
            DrawMethod::RAW => {
                {
                    let reader = display.screen.lock().unwrap();
                    for (ri, r) in reader.iter().enumerate() {
                        for (ci, c) in r.iter().enumerate() {
                            let b = match *c {
                                true => 255,
                                false => 0,
                            };
                            let color = color_u8!(b, b, b, 255);
                            let x = ci as f32 * PIXEL_WIDTH;
                            let y = ri as f32 * PIXEL_HEIGHT;
                            draw_rectangle(x, y, PIXEL_WIDTH, PIXEL_HEIGHT, color);
                            if b > 0 {
                                // println!("drawing - x: {}, y: {}, c: {:?}", x, y, color);
                            }
                        }
                    }
                }
            }
            DrawMethod::REAL => {
                for (ri, c) in display.pixels.iter().enumerate() {
                    for (ci, block) in c.iter().enumerate() {
                        let b = *block;
                        let color = color_u8!(b, b, b, 255);
                        let x = ci as f32 * PIXEL_WIDTH;
                        let y = ri as f32 * PIXEL_HEIGHT;
                        draw_rectangle(x, y, PIXEL_WIDTH, PIXEL_HEIGHT, color);
                        if b > 0 {
                            // println!("drawing - x: {}, y: {}, c: {:?}", x, y, color);
                        }
                    }
                }
            }
        }

        // Draw debug if enabled
        if debug_draw {
            let font_size: f32 = 16.0;
            fn render_string(text: &str, x: f32, y: f32, size: f32) {
                text.split("\n").enumerate().for_each(|(ind, line)| {
                    draw_text(line, x, y + ((ind as f32 + 1.0)*size), size, YELLOW);
                });
            }
            render_string(&chip.get_state(), 18.0, 18.0, font_size);
        }

        // Run processor
        // Calculate the number of steps to perform based on elapsed time
        let now = Instant::now();
        let mut elapsed = now - last_step_time;
        while elapsed >= step_duration {
            chip.step();
            elapsed -= step_duration;
            last_step_time += step_duration;
        }

        next_frame().await;
        chip.tick_timers(); // Tick timers at 60Hz
    }
}
