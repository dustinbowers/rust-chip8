use macroquad::audio::{play_sound, play_sound_once, PlaySoundParams, Sound};
use macroquad::prelude::*;
use macroquad::{audio, Error};
use std::io::Read;
use std::process::exit;
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

const KEY_MAP: &[(KeyCode, chip8::Key)] = &[
    (KeyCode::Key1, chip8::Key::Key1),
    (KeyCode::Key2, chip8::Key::Key2),
    (KeyCode::Key3, chip8::Key::Key3),
    (KeyCode::Key4, chip8::Key::C),
    (KeyCode::Q, chip8::Key::Key4),
    (KeyCode::W, chip8::Key::Key5),
    (KeyCode::E, chip8::Key::Key6),
    (KeyCode::R, chip8::Key::D),
    (KeyCode::A, chip8::Key::Key7),
    (KeyCode::S, chip8::Key::Key8),
    (KeyCode::D, chip8::Key::Key9),
    (KeyCode::F, chip8::Key::E),
    (KeyCode::Z, chip8::Key::A),
    (KeyCode::X, chip8::Key::Key0),
    (KeyCode::C, chip8::Key::B),
    (KeyCode::V, chip8::Key::F),
];

#[macroquad::main(window_conf)]
async fn main() {
    const DRAW_METHOD: DrawMethod = DrawMethod::REAL;
    let mut pause_emulation: bool = false;
    let mut debug_draw: bool = true;

    // let filename = "./roms/programs/BC_test.ch8";
    // let filename = "./roms/programs/IBM Logo.ch8";
    // let filename = "./roms/games/Breakout (Brix hack) [David Winter, 1997].ch8";
    // let filename = "./roms/games/Cave.ch8";
    let filename = "./roms/games/Space Invaders [David Winter].ch8";

    let rom = load_rom_file(filename);

    let mut boop: Sound;
    boop = match audio::load_sound("sine.wav").await {
        Ok(sound) => sound,
        Err(err) => {
            println!("Error loading sine.wav: {}", err);
            exit(0);
        }
    };

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

        if pause_emulation {
            let pause_size = 48.0;
            let pause_str = "[PAUSED]";
            let x = WINDOW_WIDTH as f32 / 2.0 - (pause_size / 2.0 * pause_str.len() as f32 / 2.0);
            let y = WINDOW_HEIGHT as f32 / 2.0;
            draw_rectangle(
                x,
                y - (pause_size * 0.75),
                pause_size * (pause_str.len() - 1) as f32 / 2.0,
                pause_size,
                RED,
            );
            draw_text(&pause_str, x, y, pause_size, BLACK);
        }

        // Draw debug if enabled
        if debug_draw {
            let debug_x: f32 = 18.0;
            let debug_y: f32 = 18.0;
            let font_size: f32 = 20.0;

            chip.get_state()
                .split("\n")
                .enumerate()
                .for_each(|(ind, line)| {
                    draw_text(
                        line,
                        debug_x,
                        debug_y + ((ind as f32 + 1.0) * font_size),
                        font_size,
                        ORANGE,
                    );
                });
        }

        // Handle user input
        chip.reset_key_state();
        let keys_pressed = get_keys_down();
        for (k, v) in KEY_MAP.iter() {
            if keys_pressed.contains(k) {
                chip.set_key_state(*v, true);
            }
        }

        // Toggle debug output
        if is_key_pressed(KeyCode::I) {
            debug_draw = !debug_draw;
        }
        // Pause / Unpause updates
        if is_key_pressed(KeyCode::P) {
            pause_emulation = match pause_emulation {
                true => {
                    // We have to reinitialize the last step time so that the CPU doesn't
                    // try to 'catch up' for all the cycles that should have happened
                    // during the paused period
                    last_step_time = Instant::now();
                    false
                }
                false => true,
            };
        }

        if pause_emulation == false {
            // Run processor
            // Calculate the number of steps to perform based on elapsed time
            let now = Instant::now();
            let mut elapsed = now - last_step_time;
            while elapsed >= step_duration {
                _ = chip.step();
                elapsed -= step_duration;
                last_step_time += step_duration;
            }

            display.update();

            let (st, _) = chip.tick_timers(); // Tick timers at 60Hz

            // Handle audio
            if st == 1 {
                // NOTE: technically the 'beep' should play continuously while ST > 0
                play_sound_once(&boop);
            }
        }

        next_frame().await;
    }
}
