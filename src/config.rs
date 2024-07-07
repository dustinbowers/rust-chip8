use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub pause_emulation: bool,
    pub debug_draw: bool,
    pub core_mode: String,
    pub ticks_per_frame: u32,
    pub color_map: Vec<u32>,
}

fn rgb_to_int(r: f32, g: f32, b: f32) -> u32 {
    let r = (r * 255.0) as u32;
    let g = (g * 255.0) as u32;
    let b = (b * 255.0) as u32;
    (r << 16) | (g << 8) | (b << 0)
}

impl Config {
    pub fn new() -> Self {
        Self {
            pause_emulation: false,
            debug_draw: true,
            core_mode: "xochip".to_string(),
            ticks_per_frame: 500,
            color_map: vec![
                rgb_to_int(0.0, 0.0, 0.0),
                rgb_to_int(0.78, 0.78, 0.78),
                rgb_to_int(0.51, 0.51, 0.51),
                rgb_to_int(0.32, 0.32, 0.32),
                rgb_to_int(0.0, 1.0, 0.0),
            ],
        }
    }
}
