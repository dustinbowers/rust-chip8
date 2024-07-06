#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct Config {
    ticks_per_frame: u32
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Config {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            ticks_per_frame: 500
        }
    }

    #[wasm_bindgen(method, getter)]
    pub fn ticks_per_frame(&self) -> u32 {
        self.ticks_per_frame
    }

    #[wasm_bindgen(method, setter)]
    pub fn set_ticks_per_frame(&mut self, ticks_per_frame: u32) {
        self.ticks_per_frame = ticks_per_frame;
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub struct Config {
    ticks_per_frame: u32
}

#[cfg(not(target_arch = "wasm32"))]
impl Config {
    pub fn new() -> Self {
        Self { ticks_per_frame: 500 }
    }

    pub fn ticks_per_frame(&self) -> u32 {
        self.ticks_per_frame
    }

    pub fn set_ticks_per_frame(&mut self, ticks_per_frame: u32) {
        self.ticks_per_frame = ticks_per_frame;
    }
}
