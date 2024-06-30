
pub enum Mode {
    Chip8Modern,
    SuperChipModern,
    SuperChipLegacy,
}
pub struct Quirks {
    pub vf_reset: bool,
    pub load_store_index_increase: bool,
    pub display_wait: bool,
    pub clipping: bool,
    pub shifting_vx: bool,
    pub jump_plus_vx: bool
}

impl Quirks {
    pub fn new(mode: Mode) -> Self {
        match mode {
            Mode::Chip8Modern => {
                Quirks {
                    vf_reset: true,
                    load_store_index_increase: true,
                    display_wait: true,
                    clipping: true,
                    shifting_vx: false,
                    jump_plus_vx: false
                }
            }
            Mode::SuperChipModern => {
                Quirks {
                    vf_reset: false,
                    load_store_index_increase: false,
                    display_wait: false,
                    clipping: true,
                    shifting_vx: true,
                    jump_plus_vx: true
                }
            }
            Mode::SuperChipLegacy => {
                Quirks {
                    vf_reset: false,
                    load_store_index_increase: false,
                    display_wait: true,
                    clipping: true,
                    shifting_vx: true,
                    jump_plus_vx: false
                }
            }
        }
    }
}