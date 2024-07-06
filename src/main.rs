use macroquad::audio;
use macroquad::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(not(target_arch = "wasm32"))]
use {
    std::io::Read,
    std::{fs, io},
    display::Display
};

#[cfg(feature = "audio")]
use macroquad::audio::{play_sound_once, Sound};

mod core;
mod display;
mod config;

use core::Chip8;
use core::{DISPLAY_COLS, DISPLAY_ROWS};
use core::DISPLAY_LAYERS;
use crate::config::Config;

const WINDOW_HEIGHT: i32 = 256;
const WINDOW_WIDTH: i32 = 512;
const PIXEL_WIDTH: f32 = WINDOW_WIDTH as f32 / DISPLAY_COLS as f32;
const PIXEL_HEIGHT: f32 = WINDOW_HEIGHT as f32 / DISPLAY_ROWS as f32;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn get_byte_array() -> js_sys::Uint8Array;
    #[wasm_bindgen(js_namespace = window)]
    fn get_config() -> Config;
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
    // include_bytes!("../roms/tests/1-core-logo.ch8").to_vec()
    // include_bytes!("../roms/tests/3-corax+.ch8").to_vec()
    // include_bytes!("../roms/tests/4-flags.ch8").to_vec()
    // include_bytes!("../roms/tests/5-quirks.ch8").to_vec()
    // include_bytes!("../roms/tests/6-keypad.ch8").to_vec()
    // include_bytes!("../roms/tests/7-beep.ch8").to_vec()
    // include_bytes!("../roms/tests/8-scrolling.ch8").to_vec()
    // include_bytes!("../roms/programs/Keypad Test [Hap, 2006].ch8").to_vec()

    // include_bytes!("../roms/xo-chip/color-scroll-test-xochip.xo8").to_vec()
    // include_bytes!("../roms/xo-chip/anEveningToDieFor.xo8").to_vec()
    include_bytes!("../roms/xo-chip/t8nks.xo8").to_vec()
    // include_bytes!("../roms/xo-chip/chip8e-test.c8e").to_vec()
    // include_bytes!("../roms/xo-chip/superneatboy.ch8").to_vec()
    // include_bytes!("../roms/xo-chip/expedition.ch8").to_vec()

    // include_bytes!("../roms/jaxe-roms/chip8archive/xochip/jub8-1.ch8").to_vec()
    // include_bytes!("../roms/jaxe-roms/chip8archive/xochip/flutterby.ch8").to_vec()
    // include_bytes!("../roms/jaxe-roms/chip8archive/xochip/chickenScratch.ch8").to_vec()

    // include_bytes!("../roms/schip/octogon.ch8").to_vec()
    // include_bytes!("../roms/schip/dodge.ch8").to_vec()
    // include_bytes!("../roms/schip/binding.ch8").to_vec()
    // include_bytes!("../roms/schip/octopeg.ch8").to_vec()
    // include_bytes!("../roms/schip/DVN8.ch8").to_vec()
    // include_bytes!("../roms/schip/oob_test_7.ch8").to_vec()

    // include_bytes!("../roms/games/Space Invaders [David Winter].ch8").to_vec()
}

#[cfg(target_arch = "wasm32")]
pub fn fetch_config() -> Config {
    get_config()
}
#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_config() -> Config {
    Config::new()
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

const KEY_MAP: &[(KeyCode, core::types::Key)] = &[
    (KeyCode::Key1, core::types::Key::Key1),
    (KeyCode::Key2, core::types::Key::Key2),
    (KeyCode::Key3, core::types::Key::Key3),
    (KeyCode::Key4, core::types::Key::C),
    (KeyCode::Q, core::types::Key::Key4),
    (KeyCode::W, core::types::Key::Key5),
    (KeyCode::E, core::types::Key::Key6),
    (KeyCode::R, core::types::Key::D),
    (KeyCode::A, core::types::Key::Key7),
    (KeyCode::S, core::types::Key::Key8),
    (KeyCode::D, core::types::Key::Key9),
    (KeyCode::F, core::types::Key::E),
    (KeyCode::Z, core::types::Key::A),
    (KeyCode::X, core::types::Key::Key0),
    (KeyCode::C, core::types::Key::B),
    (KeyCode::V, core::types::Key::F),
];

// static EMULATOR: Lazy<Mutex<Chip8>> = Lazy::new(|| Mutex::new(Chip8::new()));
// static CONFIG: Lazy<Mutex<config::Config>> = Lazy::new(|| Mutex::new(config::Config::new()));

// macro_rules! chip_lock {
//     () => { EMULATOR.lock().unwrap() };
// }

#[macroquad::main(window_conf)]
async fn main() {

    let color_map = vec![
        BLACK,
        LIGHTGRAY,
        GRAY,
        DARKGRAY,
        RED,
    ];

    const DRAW_METHOD: DrawMethod = DrawMethod::RAW; // DrawMethod::REAL;
    // let mut ticks_per_frame: f64 = 500.0;
    let mut pause_emulation: bool = false;
    let mut debug_draw: bool = true;
    // let config: Config = Config::new();
    let config = fetch_config();

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
    // let mut chip_lock = EMULATOR.lock().unwrap();
    let loaded = chip.load_rom(rom, 0x200);
    match loaded {
        Ok(b) => { println!("Loaded {:?} rom bytes", b); }
        Err(err) => { panic!("{}", err); }
    }

    let mut display = display::Display::new(chip.get_screen(), DISPLAY_ROWS, DISPLAY_COLS);


    // Time per step at 700 Hz
    // let mut last_step_time = get_time();
    let mut last_frame_time = get_time();
    loop {
        chip.v_blank();
        clear_background(GRAY);
        // TODO: Fix the way cycles are executed per frame, maybe? /Technically/ They should be
        //       evenly distributed between frame draws, rather than front-loaded all at once...
        // let step_duration = 1.0 / ticks_per_frame;
        match DRAW_METHOD {
            DrawMethod::RAW => {
                let reader = display.screen.lock().unwrap();
                for (ri, r) in reader.iter().enumerate() {
                    for (ci, c) in r.iter().enumerate() {
                        // let b = match (*c)[0] {
                        //     true => 255,
                        //     false => 0,
                        // };
                        // let color = color_u8!(b, b, b, 255);
                        let mut color_ind : u8 = 0;
                        for i in 0..DISPLAY_LAYERS {
                            if c[i] {
                                color_ind |= 1 << i;
                            }
                        }
                        // println!("color_ind = {} (0b{:04b})", color_ind, color_ind);
                        let color = color_map[color_ind as usize];
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
                        VIOLET,
                    );
                });

            let quirks = chip.get_quirks_mode();
            let s = format!("Mode: {}", quirks.mode_label);
            draw_text(
                &s,
                WINDOW_WIDTH as f32 - 200.0,
                WINDOW_HEIGHT as f32 - 4.0,
                20.0,
                RED,
            );

            let now = get_time();
            let frame_delta = now - last_frame_time;
            let fps = 1.0 / frame_delta;
            draw_text(
                &format!("FPS: {:?}", fps as u32),
                WINDOW_WIDTH as f32 - 64.0,
                12.0,
                20.0,
                RED,
            );
            draw_text(
                &format!("TPF: {:?}", config.ticks_per_frame() as u32),
                WINDOW_WIDTH as f32 - 80.0,
                24.0,
                20.0,
                RED,
            );

            last_frame_time = now;
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

        // Switch modes
        if is_key_pressed(KeyCode::Key7) {
            chip.set_quirks_mode(core::quirks::Quirks::new(core::quirks::Mode::Chip8Modern));
        }
        if is_key_pressed(KeyCode::Key8) {
            chip.set_quirks_mode(core::quirks::Quirks::new(
                core::quirks::Mode::SuperChipModern,
            ));
        }
        if is_key_pressed(KeyCode::Key9) {
            chip.set_quirks_mode(core::quirks::Quirks::new(
                core::quirks::Mode::SuperChipLegacy,
            ));
        }
        if is_key_pressed(KeyCode::Key0) {
            chip.set_quirks_mode(core::quirks::Quirks::new(core::quirks::Mode::XoChip));
        }


        // if is_key_pressed(KeyCode::Minus) {
        //     ticks_per_frame -= 100.0;
        //     ticks_per_frame = ticks_per_frame.clamp(100.0, 20000.0);
        // }
        // if is_key_pressed(KeyCode::Equal) {
        //     ticks_per_frame += 100.0;
        //     ticks_per_frame = ticks_per_frame.clamp(100.0, 20000.0);
        // }

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
                // last_step_time = get_time();
            }
            pause_emulation = !pause_emulation;
        }

        if pause_emulation == false {
            // Run processor
            for _ in 0..config.ticks_per_frame() {
                // TODO: Handle errors gracefully...
                _ = chip.step();
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
