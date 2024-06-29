use macroquad::audio;
use macroquad::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use {
    std::io::Read,
    std::{fs, io},
};

#[cfg(feature = "audio")]
use macroquad::audio::{play_sound_once, Sound};

mod chip8;
mod display;
use chip8::Chip8;
use crate::chip8::{ DISPLAY_ROWS, DISPLAY_COLS};

const WINDOW_HEIGHT: i32 = 256;
const WINDOW_WIDTH: i32 = 512;
const PIXEL_WIDTH: f32 = WINDOW_WIDTH as f32 / DISPLAY_COLS as f32;
const PIXEL_HEIGHT: f32 = WINDOW_HEIGHT as f32 / DISPLAY_ROWS as f32;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn get_byte_array() -> js_sys::Uint8Array;
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn fetch_byte_array_from_js() -> Vec<u8> {
    let js_array = crate::get_byte_array();
    js_array.to_vec()
}

#[cfg(target_arch = "wasm32")]
pub fn fetch_rom_bytes() -> Vec<u8> {
    fetch_byte_array_from_js()
}
#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_rom_bytes() -> Vec<u8> {
    // Test CPU
    // include_bytes!("../roms/programs/BC_test.ch8").to_vec()
    // include_bytes!("../roms/programs/Keypad Test [Hap, 2006].ch8").to_vec()
    // include_bytes!("../roms/schip/octopeg.ch8").to_vec()
    // include_bytes!("../roms/schip/gradsim.ch8").to_vec()
    // include_bytes!("../roms/schip/sub8.ch8").to_vec()
    // include_bytes!("../roms/schip/1-chip8-logo.ch8").to_vec()
    // include_bytes!("../roms/schip/3-corax+.ch8").to_vec()
    // include_bytes!("../roms/schip/4-flags.ch8").to_vec()
    // include_bytes!("../roms/schip/5-quirks.ch8").to_vec()
    // include_bytes!("../roms/schip/6-keypad.ch8").to_vec()
    // include_bytes!("../roms/schip/7-beep.ch8").to_vec()
    include_bytes!("../roms/schip/8-scrolling.ch8").to_vec()

    // include_bytes!("../roms/games/Space Invaders [David Winter].ch8").to_vec()
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Chip 8".to_owned(),
        fullscreen: false,
        window_height: WINDOW_HEIGHT,
        window_width: WINDOW_WIDTH,
        ..Default::default()
    }
}

#[allow(dead_code)]
#[cfg(not(target_arch = "wasm32"))]
fn load_rom_file(filename: &str) -> io::Result<Vec<u8>> {
    // TODO: might use this later
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

const KEY_MAP: &[(KeyCode, chip8::types::Key)] = &[
    (KeyCode::Key1, chip8::types::Key::Key1),
    (KeyCode::Key2, chip8::types::Key::Key2),
    (KeyCode::Key3, chip8::types::Key::Key3),
    (KeyCode::Key4, chip8::types::Key::C),
    (KeyCode::Q, chip8::types::Key::Key4),
    (KeyCode::W, chip8::types::Key::Key5),
    (KeyCode::E, chip8::types::Key::Key6),
    (KeyCode::R, chip8::types::Key::D),
    (KeyCode::A, chip8::types::Key::Key7),
    (KeyCode::S, chip8::types::Key::Key8),
    (KeyCode::D, chip8::types::Key::Key9),
    (KeyCode::F, chip8::types::Key::E),
    (KeyCode::Z, chip8::types::Key::A),
    (KeyCode::X, chip8::types::Key::Key0),
    (KeyCode::C, chip8::types::Key::B),
    (KeyCode::V, chip8::types::Key::F),
];

#[macroquad::main(window_conf)]
async fn main() {
    const DRAW_METHOD: DrawMethod = DrawMethod::REAL;
    let mut ticks_per_sec = 700.0;
    // let mut ticks_per_sec = 1400.0;
    let mut pause_emulation: bool = false;
    let mut debug_draw: bool = true;

    let rom = fetch_rom_bytes();

    let boop: Sound;
    #[cfg(feature = "audio")]
    {
        boop = match audio::load_sound_from_bytes(include_bytes!("sine.wav")).await {
            Ok(sound) => sound,
            Err(err) => {
                println!("Error loading sine.wav: {}", err);
                return;
            }
        };
    }

    let mut chip = Chip8::new();
    _ = chip.load_rom(rom, 0x200);
    let mut display = display::Display::new(chip.get_screen(), DISPLAY_ROWS, DISPLAY_COLS);

    // Time per step at 700 Hz
    let mut last_step_time = get_time();

    loop {
        clear_background(GRAY);
        let step_duration = 1.0 / ticks_per_sec;
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
            let debug_x: f32 = 12.0;
            let debug_y: f32 = 0.0;
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
        let keys_pressed = get_keys_down();
        for (k, v) in KEY_MAP.iter() {
            if keys_pressed.contains(k) {
                chip.set_key_state(*v, true);
            } else {
                chip.set_key_state(*v, false);
            }
        }

        // Toggle debug output
        if is_key_pressed(KeyCode::I) {
            debug_draw = !debug_draw;
        }
        // Pause / Unpause updates
        if is_key_pressed(KeyCode::P) {
            if pause_emulation {
                // We have to reinitialize the last step time so that the CPU doesn't
                // try to 'catch up' for all the cycles that should have happened
                // during the paused period
                last_step_time = get_time();
            }
            pause_emulation = !pause_emulation;
        }

        if pause_emulation == false {
            // Run processor
            // Calculate the number of steps to perform based on elapsed time
            let now = get_time();
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
                #[cfg(feature = "audio")]
                {
                    play_sound_once(&boop);
                }
            }
        }

        next_frame().await;
    }
}
