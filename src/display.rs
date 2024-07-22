use crate::core::error::CoreError;
use crate::core::quirks::Quirks;
use crate::core::types::Screen;
use crate::{PIXEL_HEIGHT, PIXEL_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH};
use js_sys::Math::sin;
use macroquad::color::{Color, BLACK, RED, VIOLET};
use macroquad::prelude::{draw_rectangle, draw_text};
use std::f64::consts::PI;

pub fn draw_splash(last_frame_time: f64) {
    let alpha = sin(last_frame_time % PI) as f32;
    let size = 48.0;
    let str = "Ready";
    let x = WINDOW_WIDTH as f32 / 2.0 - (size / 2.0 * str.len() as f32 / 2.0);
    let y = WINDOW_HEIGHT as f32 / 2.0;
    draw_text(str, x, y, size, Color::new(1.0, 1.0, 1.0, alpha));
}

pub fn draw_screen(screen: &Screen, color_map: &[Color]) {
    for (ri, r) in screen.iter().enumerate() {
        for (ci, c) in r.iter().enumerate() {
            let mut color_ind: u8 = 0;
            for (i, c) in c.iter().enumerate() {
                if *c {
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

pub fn draw_pause() {
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
    draw_text(pause_str, x, y, pause_size, BLACK);
}
pub fn draw_basic_debug_info(quirks: &Quirks, ticks_per_frame: u32, frame_delta: f64) {
    let fps = 1.0 / frame_delta;
    draw_text(
        &format!("FPS: {:?}", fps as u32),
        WINDOW_WIDTH as f32 - 100.0,
        12.0,
        20.0,
        RED,
    );
    draw_text(
        &format!("IPF: {:?}", ticks_per_frame),
        WINDOW_WIDTH as f32 - 100.0,
        24.0,
        20.0,
        RED,
    );
    draw_text(
        &format!("Mode: {}", quirks.mode_label),
        WINDOW_WIDTH as f32 - 200.0,
        WINDOW_HEIGHT as f32 - 4.0,
        20.0,
        RED,
    );
}

pub fn draw_emu_state(state_str: &str) {
    let debug_x: f32 = 12.0;
    let debug_y: f32 = 0.0;
    let font_size: f32 = 20.0;
    draw_string_lines(state_str, debug_x, debug_y, font_size, VIOLET);
}

pub fn draw_string_lines(str: &str, x: f32, y: f32, font_size: f32, text_color: Color) {
    str.split('\n').enumerate().for_each(|(ind, line)| {
        draw_text(
            line,
            x,
            y + ((ind as f32 + 1.0) * font_size),
            font_size,
            text_color,
        );
    });
}

pub fn show_error(err: &CoreError) {
    let debug_x = 30.0;
    let debug_y = 70.0;
    let font_size = 24.0;
    let err_box_color = Color::from_rgba(216, 80, 77, 255);
    let err_box_color2 = Color::from_rgba(177, 60, 57, 255);
    let text_color = Color::from_rgba(255, 255, 255, 255);

    // Main container
    draw_rectangle(
        16.0,
        16.0,
        (WINDOW_WIDTH - 32) as f32,
        (WINDOW_HEIGHT - 32) as f32,
        err_box_color,
    );

    // Header box && Label
    draw_rectangle(24.0, 24.0, (WINDOW_WIDTH - 48) as f32, 42.0, err_box_color2);
    draw_text(
        "ERROR",
        WINDOW_WIDTH as f32 / 2.0 - 36.0,
        54.0,
        32.0,
        text_color,
    );

    // Error info
    let err_text = format!("Type: {}\nInfo: {}", err.error_type, err.info);
    draw_string_lines(&err_text, debug_x, debug_y, font_size, text_color);
}
