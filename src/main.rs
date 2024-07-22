use macroquad::prelude::*;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex, RwLock};
use tinyaudio::{run_output_device, BaseAudioOutputDevice, OutputDeviceParameters};

#[cfg(not(target_arch = "wasm32"))]
use {
    std::io::Read,
    std::{fs, io},
};

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

mod config;
mod core;
mod square_wave;
mod display;

use crate::config::Config;
use crate::core::error::CoreError;
use crate::core::quirks::Mode;
use crate::square_wave::SquareWave;
use core::Chip8;
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
    // include_bytes!("../roms/xo-chip/scroll_edge_test_b.ch8").to_vec()
    // include_bytes!("../roms/xo-chip/anEveningToDieFor.xo8").to_vec()
    // include_bytes!("../roms/xo-chip/t8nks.xo8").to_vec()
    // include_bytes!("../roms/xo-chip/chip8e-test.c8e").to_vec()
    // include_bytes!("../roms/xo-chip/superneatboy.ch8").to_vec()
    include_bytes!("../roms/xo-chip/nyancat.ch8").to_vec()
    // include_bytes!("../roms/xo-chip/NYAN.xo8").to_vec()
    // include_bytes!("../roms/xo-chip/expedition.ch8").to_vec()
    // include_bytes!("../roms/xo-chip/alien-inv8sion.ch8").to_vec()

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

#[wasm_bindgen]
pub fn send_new_config_to_js() -> JsValue {
    let new_conf = Config::new();
    serde_wasm_bindgen::to_value(&new_conf).unwrap()
}

#[wasm_bindgen]
pub fn reset_core() {
    let mut state = STATE.write().unwrap();
    *state = EmuState::Load;
}

#[derive(Clone, Copy, PartialEq)]
enum EmuState {
    Preload,
    Load,
    Run,
    Error,
}

static STATE: Lazy<Arc<RwLock<EmuState>>> = Lazy::new(|| Arc::new(RwLock::new(EmuState::Preload)));

#[macroquad::main(window_conf)]
async fn main() {
    let global_square_wave = Arc::new(Mutex::new(SquareWave::new()));
    let audio_volume = 0.1f32;

    let mut chip: Chip8 = Chip8::new();
    let mut core_error: Option<CoreError> = None;
    let global_config: Arc<Mutex<Config>> = Arc::new(Mutex::new(Config::new()));
    let mut color_map: Vec<Color>;
    let mut rom: Vec<u8>;

    let mut audio_device: Option<Box<dyn BaseAudioOutputDevice>> = None;

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut s = STATE.write().unwrap();
        *s = EmuState::Load;
        drop(s);
    }

    // TODO: Potentially move color_map into this its own struct?
    color_map = global_config
        .lock()
        .unwrap()
        .color_map
        .iter()
        .map(|c| {
            let r = ((c >> 16) & 0xFFu32) as f32 / 255.0;
            let g = ((c >> 8) & 0xFFu32) as f32 / 255.0;
            let b = ((c) & 0xFFu32) as f32 / 255.0;
            Color::new(r, g, b, 1.0)
        })
        .collect();

    let key_map: &[(Vec<KeyCode>, core::types::Key)] = &[
        (vec![KeyCode::Key1], core::types::Key::Key1),
        (vec![KeyCode::Key2], core::types::Key::Key2),
        (vec![KeyCode::Key3], core::types::Key::Key3),
        (vec![KeyCode::Key4], core::types::Key::C),
        (vec![KeyCode::Q], core::types::Key::Key4),
        (vec![KeyCode::W], core::types::Key::Key5),
        (vec![KeyCode::E, KeyCode::Space], core::types::Key::Key6),
        (vec![KeyCode::R], core::types::Key::D),
        (vec![KeyCode::A], core::types::Key::Key7),
        (vec![KeyCode::S], core::types::Key::Key8),
        (vec![KeyCode::D], core::types::Key::Key9),
        (vec![KeyCode::F], core::types::Key::E),
        (vec![KeyCode::Z], core::types::Key::A),
        (vec![KeyCode::X], core::types::Key::Key0),
        (vec![KeyCode::C], core::types::Key::B),
        (vec![KeyCode::V], core::types::Key::F),
    ];

    let mut last_frame_time = get_time();
    loop {
        let config_handle = Arc::clone(&global_config);

        // Handle user input
        let keys_pressed = get_keys_down();
        for (keys, v) in key_map.iter() {
            let mut pressed = false;
            for k in keys.iter() {
                if keys_pressed.contains(k) {
                    pressed = true;
                }
            }
            chip.set_key_state(*v, pressed);
        }

        // Switch modes
        if is_key_pressed(KeyCode::Key7) {
            chip.set_quirks_mode(core::quirks::Quirks::new(Mode::Chip8Modern));
        }
        if is_key_pressed(KeyCode::Key8) {
            chip.set_quirks_mode(core::quirks::Quirks::new(Mode::SuperChipModern));
        }
        if is_key_pressed(KeyCode::Key9) {
            chip.set_quirks_mode(core::quirks::Quirks::new(Mode::SuperChipLegacy));
        }
        if is_key_pressed(KeyCode::Key0) {
            chip.set_quirks_mode(core::quirks::Quirks::new(Mode::XoChip));
        }

        if is_key_pressed(KeyCode::Minus) {
            let mut config = config_handle.lock().unwrap();
            let increment = get_ipf_increment(config.ticks_per_frame);
            config.ticks_per_frame -= increment;
            config.ticks_per_frame = config.ticks_per_frame.clamp(1, 200000);
        }

        if is_key_pressed(KeyCode::Equal) {
            let mut config = config_handle.lock().unwrap();
            let increment = get_ipf_increment(config.ticks_per_frame);
            config.ticks_per_frame += increment;
            config.ticks_per_frame = config.ticks_per_frame.clamp(1, 200000);
        }

        // Toggle debug output
        if is_key_pressed(KeyCode::I) {
            let mut config = config_handle.lock().unwrap();
            config.debug_draw += 1;
            config.debug_draw %= 3;
        }
        // Pause / Unpause updates
        if is_key_pressed(KeyCode::P) {
            let mut config = config_handle.lock().unwrap();
            config.pause_emulation = !config.pause_emulation;
        }

        // TODO: Remove this
        // BLOW UP THE CORE - just for fun
        if is_key_pressed(KeyCode::F5) {
            chip.chaos();
        }

        // Draw the screen
        chip.v_blank();
        
        display::draw_screen(&(chip.get_screen().lock().unwrap()), &color_map);

        let current_state = {
            let state_read = STATE.read().unwrap();
            *state_read
        };
        match current_state {
            EmuState::Preload => {
                display::draw_splash(last_frame_time);
            }
            EmuState::Load => {
                #[cfg(feature = "chip-audio")]
                if audio_device.is_none() {
                    audio_device = init_audio(&global_square_wave, &global_config, audio_volume);
                }

                chip.reset();
                rom = fetch_rom_bytes();
                let new_config = fetch_config();
                let mut config_handle = global_config.lock().unwrap();
                config_handle.update(new_config);
                chip.set_core_mode(&config_handle.core_mode);

                color_map = config_handle
                    .color_map
                    .iter()
                    .map(|c| {
                        let r = ((c >> 16) & 0xFFu32) as f32 / 255.0;
                        let g = ((c >> 8) & 0xFFu32) as f32 / 255.0;
                        let b = ((c) & 0xFFu32) as f32 / 255.0;
                        Color::new(r, g, b, 1.0)
                    })
                    .collect();

                let loaded = chip.load_rom(rom, 0x200);
                match loaded {
                    Ok(b) => {
                        println!("Loaded {:?} ROM bytes", b);
                    }
                    Err(err) => {
                        panic!("Error loading ROM bytes: {}", err);
                    }
                }

                let mut state_writer = STATE.write().unwrap();
                *state_writer = EmuState::Run;
                drop(config_handle);
            }
            EmuState::Run => {
                // Run processor
                let config_handle = Arc::clone(&global_config);
                let config = config_handle.lock().unwrap();

                if !config.pause_emulation {
                    for _ in 0..config.ticks_per_frame {
                        if let Err(e) = chip.step() {
                            println!("Error: {:#?}", e);
                            core_error = Some(e);
                            let mut state_writer = STATE.write().unwrap();
                            *state_writer = EmuState::Error;
                        }
                    }
                    let (st, _) = chip.tick_timers(); // Tick timers at 60Hz

                    #[cfg(feature = "chip-audio")]
                    if audio_device.is_some() {
                        let sw_handle = Arc::clone(&global_square_wave);
                        if st > 0 {
                            if let Mode::XoChip = chip.quirks_mode().mode {
                                if let Some(snd) = chip.get_sound() {
                                    sw_handle
                                        .lock()
                                        .unwrap()
                                        .set_pattern(snd.pitch, snd.pattern.clone());
                                }
                            } else {
                                sw_handle.lock().unwrap().set_pattern(
                                    128,
                                    vec![
                                        0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF,
                                        0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00,
                                    ],
                                );
                            }
                        } else {
                            sw_handle.lock().unwrap().set_pattern(64, vec![0u8; 16]);
                        }
                    }
                }
            }
            EmuState::Error => {
                if let Some(err) = &core_error {
                    display::show_error(err);
                }
                if is_key_pressed(KeyCode::Enter) {
                    let mut state_writer = STATE.write().unwrap();
                    *state_writer = EmuState::Run; 
                    core_error = None;
                }
            }
        };
        let config = global_config.lock().unwrap();

        if config.pause_emulation {
            display::draw_pause();
        }

        let now = get_time();
        if config.debug_draw > 0 {
            display::draw_basic_debug_info(chip.quirks_mode(), config.ticks_per_frame, now-last_frame_time);
        }

        if config.debug_draw > 1 {
            display::draw_emu_state(&chip.get_state());
        }
        
        last_frame_time = now;
        drop(config);
        next_frame().await;
    }
}

fn get_ipf_increment(val: u32) -> u32 {
    match val {
        0..=9 => 1,
        10..=99 => 10,
        100..=999 => 50,
        1000..=9999 => 500,
        10_000..=99_999 => 5_000,
        100_000..=999_999 => 50_000,
        _ => 100,
    }
}

#[cfg(feature = "chip-audio")]
fn init_audio(
    global_square_wave: &Arc<Mutex<SquareWave>>,
    global_config: &Arc<Mutex<Config>>,
    audio_volume: f32,
) -> Option<Box<dyn BaseAudioOutputDevice>> {
    let sw_handle = Arc::clone(&global_square_wave);
    let audio_config_handle = Arc::clone(&global_config);
    let params = OutputDeviceParameters {
        channels_count: 1,
        sample_rate: 44100,
        channel_sample_count: 735,
    };

    let device = run_output_device(params, {
        move |data| {
            let c = audio_config_handle.lock().unwrap();
            let paused = c.pause_emulation;
            drop(c);
            if paused {
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
    });

    match device {
        Ok(d) => Some(d),
        Err(_) => None,
    }
}
