use macroquad::prelude::*;
use std::sync::{Arc, Mutex};
use tinyaudio::{run_output_device, OutputDeviceParameters};

#[cfg(not(target_arch = "wasm32"))]
use {
    std::io::Read,
    std::{fs, io},
};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

mod config;
mod core;
mod display;
mod square_wave;

use crate::config::Config;
use crate::core::quirks::Mode;
use crate::square_wave::SquareWave;
use core::Chip8;
use core::DISPLAY_LAYERS;
use core::{DISPLAY_COLS, DISPLAY_ROWS};

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
    fn get_config() -> JsValue;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn fetch_byte_array_from_js() -> Vec<u8> {
    let js_array = crate::get_byte_array();
    js_array.to_vec()
}

#[cfg(target_arch = "wasm32")]
pub fn fetch_config() -> Config {
    let val = get_config();
    let new_conf: Config = serde_wasm_bindgen::from_value(val).unwrap();
    new_conf
}
#[cfg(not(target_arch = "wasm32"))]
fn fetch_config() -> Config {
    Config::new()
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
    // include_bytes!("../roms/xo-chip/t8nks.xo8").to_vec()
    // include_bytes!("../roms/xo-chip/chip8e-test.c8e").to_vec()
    include_bytes!("../roms/xo-chip/superneatboy.ch8").to_vec()
    // include_bytes!("../roms/xo-chip/nyancat.ch8").to_vec()
    // include_bytes!("../roms/xo-chip/NYAN.xo8").to_vec()
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

#[wasm_bindgen]
pub fn send_new_config_to_js() -> JsValue {
    let new_conf = Config::new();
    serde_wasm_bindgen::to_value(&new_conf).unwrap()
}

#[macroquad::main(window_conf)]
async fn main() {
    const DRAW_METHOD: DrawMethod = DrawMethod::RAW; // DrawMethod::REAL;

    let global_config: Arc<Mutex<Config>> = Arc::new(Mutex::new(fetch_config()));
    let rom = fetch_rom_bytes();

    let color_map: Vec<Color> = global_config
        .lock()
        .unwrap()
        .color_map
        .iter()
        .map(|c| {
            let r = ((c >> 16) & 0xFFu32) as f32 / 255.0;
            let g = ((c >> 8) & 0xFFu32) as f32 / 255.0;
            let b = ((c >> 0) & 0xFFu32) as f32 / 255.0;
            Color::new(r, g, b, 1.0)
        })
        .collect();

    let square_wave = Arc::new(Mutex::new(SquareWave::new()));
    let audio_volume = 0.1f32;
    // let device: Box<dyn BaseAudioOutputDevice>;
    #[cfg(feature = "xo-audio")]
    {
        let params = OutputDeviceParameters {
            channels_count: 1,
            sample_rate: 44100,
            channel_sample_count: 1024,
        };

        let sw_handle = Arc::clone(&square_wave);
        let config_handle = Arc::clone(&global_config);
        _ = run_output_device(params, {
            move |data| {
                if config_handle.lock().unwrap().pause_emulation {
                    for d in data {
                        *d = 0.0;
                    }
                    return;
                }
                for samples in data.chunks_mut(params.channels_count) {
                    for sample in samples {
                        let mut sw = sw_handle.lock().unwrap();
                        *sample = if sw.bit_pattern[(sw.phase_bit + 0.5) as usize] {
                            audio_volume
                        } else {
                            -audio_volume
                        };
                        sw.phase_bit += sw.phase_inc;
                        if (sw.phase_bit + 0.5) as usize >= 128 {
                            sw.phase_bit = 0.0;
                        }
                    }
                }
            }
        })
        .unwrap();
    }

    let mut chip = Chip8::new();
    chip.set_core_mode(&global_config.lock().unwrap().core_mode);

    let loaded = chip.load_rom(rom, 0x200);
    match loaded {
        Ok(b) => {
            println!("Loaded {:?} ROM bytes", b);
        }
        Err(err) => {
            panic!("Error loading ROM bytes: {}", err);
        }
    }

    let mut display = display::Display::new(chip.get_screen(), DISPLAY_ROWS, DISPLAY_COLS);

    let mut last_frame_time = get_time();
    loop {
        let config_handle = Arc::clone(&global_config);
        let mut config = config_handle.lock().unwrap();
        chip.v_blank();
        match DRAW_METHOD {
            DrawMethod::RAW => {
                let reader = display.screen.lock().unwrap();
                for (ri, r) in reader.iter().enumerate() {
                    for (ci, c) in r.iter().enumerate() {
                        let mut color_ind: u8 = 0;
                        for i in 0..DISPLAY_LAYERS {
                            if c[i] {
                                color_ind |= 1 << i;
                            }
                        }
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

        if config.pause_emulation {
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
        if config.debug_draw {
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

            let quirks = chip.quirks_mode();
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
                &format!("TPF: {:?}", config.ticks_per_frame as u32),
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

        if is_key_pressed(KeyCode::Minus) {
            config.ticks_per_frame -= 100;
            config.ticks_per_frame = config.ticks_per_frame.clamp(100, 20000);
        }
        if is_key_pressed(KeyCode::Equal) {
            config.ticks_per_frame += 100;
            config.ticks_per_frame = config.ticks_per_frame.clamp(100, 20000);
        }

        // Toggle debug output
        if is_key_pressed(KeyCode::I) {
            config.debug_draw = !config.debug_draw;
        }
        // Pause / Unpause updates
        if is_key_pressed(KeyCode::P) {
            config.pause_emulation = !config.pause_emulation;
        }

        // TODO: Remove this
        // BLOW UP THE CORE - just for fun
        if is_key_pressed(KeyCode::F5) {
            chip.chaos();
        }

        if !config.pause_emulation {
            // Run processor
            for _ in 0..config.ticks_per_frame {
                if let Err(e) = chip.step() {
                    println!("Error: {}", e);
                    show_error(e).await;
                }
            }
            display.update();

            let (st, _) = chip.tick_timers(); // Tick timers at 60Hz

            // Handle audio
            #[cfg(feature = "xo-audio")]
            {
                let sw_handle = Arc::clone(&square_wave);
                if st > 0 {
                    if let Mode::XoChip = chip.quirks_mode().mode {
                        if let Some(snd) = chip.get_sound() {
                            sw_handle
                                .lock()
                                .unwrap()
                                .set_pattern(snd.pitch, snd.pattern.clone());
                        };
                    } else {
                        sw_handle.lock().unwrap().set_pattern(
                            128,
                            vec![
                                0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00,
                                0x00, 0xFF, 0xFF, 0x00, 0x00,
                            ],
                        );
                    }
                } else {
                    sw_handle.lock().unwrap().set_pattern(64, vec![0u8; 16]);
                }
            }
        }
        drop(config);
        next_frame().await;
    }
}

async fn show_error(err: core::error::CoreError) {
    println!("show_error - Error: {:#?}", err);
    let debug_x = 30.0;
    let debug_y = 70.0;
    let font_size = 24.0;
    let err_box_color = Color::from_rgba(216, 80, 77, 255);
    let err_box_color2 = Color::from_rgba(177, 60, 57, 255);
    let text_color = Color::from_rgba(255, 255, 255, 255);

    loop {
        draw_rectangle(
            16.0,
            16.0,
            (WINDOW_WIDTH - 32) as f32,
            (WINDOW_HEIGHT - 32) as f32,
            err_box_color,
        );
        draw_rectangle(24.0, 24.0, (WINDOW_WIDTH - 48) as f32, 42.0, err_box_color2);
        draw_text(
            "ERROR",
            WINDOW_WIDTH as f32 / 2.0 - 36.0,
            54.0,
            32.0,
            text_color,
        );
        let err_text = format!("Type: {}\nInfo: {}", err.error_type, err.info);
        err_text.split("\n").enumerate().for_each(|(ind, line)| {
            draw_text(
                line,
                debug_x,
                debug_y + ((ind as f32 + 1.0) * font_size),
                font_size,
                text_color,
            );
        });
        if is_key_pressed(KeyCode::Enter) {
            break;
        }
        next_frame().await;
    }
}
