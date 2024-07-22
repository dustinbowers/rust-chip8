use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub pause_emulation: bool,
    pub debug_draw: u8, 
    pub core_mode: String,
    pub ticks_per_frame: u32,
    pub color_map: Vec<u32>,
    pub audio_level: f32,
}

fn rgb_to_int(r: f32, g: f32, b: f32) -> u32 {
    let r = (r * 255.0) as u32;
    let g = (g * 255.0) as u32;
    let b = (b * 255.0) as u32;
    (r << 16) | (g << 8) | b
}

impl Config {
    pub fn new() -> Self {
        Self {
            pause_emulation: false,
            debug_draw: 0,
            core_mode: "xo-chip".to_string(),
            ticks_per_frame: 100000,
            audio_level: 0.1,
            color_map: vec![
                rgb_to_int(0.0, 0.0, 0.0),
                rgb_to_int(0.78, 0.78, 0.78),
                rgb_to_int(0.51, 0.51, 0.51),
                rgb_to_int(0.32, 0.32, 0.32),
                
                rgb_to_int(1.0, 0.0, 0.0),
                rgb_to_int(1.0, 0.5, 0.0),
                rgb_to_int(1.0, 1.0, 0.0),
                rgb_to_int(1.0, 1.0, 0.0),
                rgb_to_int(0.5, 1.0, 0.0),
                rgb_to_int(0.0, 1.0, 0.0),
                rgb_to_int(0.0, 1.0, 0.5),
                rgb_to_int(0.0, 1.0, 1.0),
                rgb_to_int(0.0, 0.5, 1.0),
                rgb_to_int(0.0, 0.0, 1.0),
                rgb_to_int(0.5, 0.0, 1.0),
                rgb_to_int(1.0, 0.0, 1.0),
                rgb_to_int(1.0, 0.0, 0.5),
                rgb_to_int(0.5, 0.5, 0.5),
           ],
        }
    }

    pub fn update(&mut self, other: Self) {
        *self = Self {
            ..other
        };
    }
}
