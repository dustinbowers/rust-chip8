use bitvec::prelude::BitVec;

pub struct SquareWave {
    pub bit_pattern: BitVec<u8>, // 128 1-bit samples
    pub phase_inc: f64, // (4000*2^((vx-64)/48)) / device_sample_rate
    pub phase_bit: f64, // looping index in bit_pattern
}

impl SquareWave {
    pub fn new() -> Self {
        Self {
            bit_pattern: BitVec::<u8>::from_vec(vec![
                0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
                0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,
            ]),
            phase_inc: Self::pitch_to_ratio(128),
            phase_bit: 0.0,
        }
    }

    pub fn pitch_to_ratio(pitch: u8) -> f64 {
        let base: f64 = 2.0;
        let sr = (4000.0 * base.powf((pitch as f64 - 64.0) / 48.0));
        sr / 44100.0
    }

    pub fn set_pattern(&mut self, pitch: u8, pattern: Vec<u8>)
    {
        self.bit_pattern = BitVec::from_vec(pattern);
        self.phase_inc = Self::pitch_to_ratio(pitch);
        self.phase_bit = 0.0;
    }
}
