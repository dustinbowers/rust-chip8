use macroquad::prelude::*;
use std::io::Read;
use std::time::{Duration, Instant};
use std::{fs, io};

mod chip8;
mod display;

use chip8::Chip8;

const WINDOW_HEIGHT: i32 = 256;
const WINDOW_WIDTH: i32 = 512;
const PIXEL_WIDTH: f32 = WINDOW_WIDTH as f32 / 64.0;
const PIXEL_HEIGHT: f32 = WINDOW_HEIGHT as f32 / 32.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Chip 8".to_owned(),
        fullscreen: false,
        window_height: WINDOW_HEIGHT,
        window_width: WINDOW_WIDTH,
        ..Default::default()
    }
}

fn load_rom_file(filename: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(filename)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[allow(dead_code)]
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
    let mut display = display::Display::new(chip.get_screen());

    // Time per step at 700 Hz
    let step_duration = Duration::from_secs_f64(1.0 / 700.0);
    let mut last_step_time = Instant::now();

    loop {
        display.update();
        clear_background(GRAY);
        match DRAW_METHOD {
            DrawMethod::RAW => {
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
                    }
                }
            }
        }

        // Draw debug if enabled
        if debug_draw {
            let debug_x: f32 = 18.0;
            let debug_y: f32 = 18.0;
            let font_size: f32 = 20.0;
            fn render_string(text: &str, x: f32, y: f32, size: f32) {
                text.split("\n").enumerate().for_each(|(ind, line)| {
                    draw_text(line, x, y + ((ind as f32 + 1.0) * size), size, ORANGE);
                });
            }
            render_string(&chip.get_state(), debug_x, debug_y, font_size);
        }

        // Run processor
        // Calculate the number of steps to perform based on elapsed time
        let now = Instant::now();
        let mut elapsed = now - last_step_time;
        while elapsed >= step_duration {
            _ = chip.step();
            elapsed -= step_duration;
            last_step_time += step_duration;
        }

        next_frame().await;
        chip.tick_timers(); // Tick timers at 60Hz
    }
}
