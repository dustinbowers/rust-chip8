use macroquad::color::Color;

pub struct ColorMap {
    default_map: Vec<Color>,
    custom_map: Vec<Color>,
}

impl ColorMap {
    pub fn new() -> Self {
        Self {
            custom_map: vec![],
            default_map: vec![
                Color::from([0.0, 0.0, 0.0, 0.0]),
                
                Color::from([0.78, 0.78, 0.7, 0.0]),
                Color::from([0.51, 0.51, 0.51, 0.0]),
                Color::from([0.32, 0.32, 0.32, 0.0]),

                // Color::from([1.0, 0.0, 0.0, 0.0]),
                Color::from([1.0, 0.5, 0.0, 0.0]),
                Color::from([1.0, 1.0, 0.0, 0.0]),
                Color::from([1.0, 1.0, 0.0, 0.0]),
                Color::from([0.5, 1.0, 0.0, 0.0]),
                Color::from([0.0, 1.0, 0.0, 0.0]),
                Color::from([0.0, 1.0, 0.5, 0.0]),
                Color::from([0.0, 1.0, 1.0, 0.0]),
                Color::from([0.0, 0.5, 1.0, 0.0]),
                Color::from([0.0, 0.0, 1.0, 0.0]),
                Color::from([0.5, 0.0, 1.0, 0.0]),
                Color::from([1.0, 0.0, 1.0, 0.0]),
                Color::from([1.0, 0.0, 0.5, 0.0]),
                Color::from([0.5, 0.5, 0.5, 0.0]),
            ]
        }
    }
}

impl ColorMap {    
    pub fn set_int_color_map(&mut self, int_color_map: &Vec<u32>) {
        self.custom_map = int_color_map.iter()
            .map(|c| {
                let r = ((c >> 16) & 0xFFu32) as f32 / 255.0;
                let g = ((c >> 8) & 0xFFu32) as f32 / 255.0;
                let b = ((c) & 0xFFu32) as f32 / 255.0;
                Color::new(r, g, b, 1.0)
            })
            .collect();
    }
    
    #[inline]
    pub fn get_color(&self, ind: usize) -> &Color {
        if ind < self.custom_map.len() {
            &self.custom_map[ind]
        } else {
            &self.default_map[ind]
        }
    }
}
