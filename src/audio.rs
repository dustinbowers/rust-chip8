use std::sync::{Arc, Mutex};
use crate::config::Config;
use tinyaudio::{run_output_device, BaseAudioOutputDevice, OutputDeviceParameters};

use bitvec::prelude::BitVec;
pub struct SquareWave {
    pub bit_pattern: BitVec<u8>, // 128 1-bit samples
    pub phase_inc: f64,          // (4000*2^((vx-64)/48)) / device_sample_rate
    pub phase_bit: f64,          // looping index in bit_pattern
}

impl SquareWave {
    pub fn new() -> Self {
        Self {
            bit_pattern: BitVec::<u8>::from_vec(vec![0u8; 16]),
            phase_inc: Self::pitch_to_ratio(128),
            phase_bit: 0.0,
        }
    }

    pub fn pitch_to_ratio(pitch: u8) -> f64 {
        let base: f64 = 2.0;
        let sr = 4000.0 * base.powf((pitch as f64 - 64.0) / 48.0);
        sr / 44100.0
    }

    pub fn set_pattern(&mut self, pitch: u8, pattern: Vec<u8>) {
        self.bit_pattern = BitVec::from_vec(pattern);
        self.phase_inc = Self::pitch_to_ratio(pitch);
    }
}

pub fn init_audio(
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
