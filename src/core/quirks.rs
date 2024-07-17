pub enum Mode {
    Chip8Modern,
    SuperChipModern,
    SuperChipLegacy,
    XoChip,
}
pub struct Quirks {
    pub mode: Mode,
    pub mode_label: String,
    pub vf_reset: bool,
    pub load_store_index_increase: bool,
    pub display_wait: bool,
    pub clipping: bool,
    pub shifting_vx: bool,
    pub jump_plus_vx: bool,
}

impl Quirks {
    pub fn new(mode: Mode) -> Self {
        match mode {
            Mode::Chip8Modern => Quirks {
                mode: Mode::Chip8Modern,
                mode_label: "Chip8-Modern".to_string(),
                vf_reset: true,
                load_store_index_increase: true,
                display_wait: true,
                clipping: true,
                shifting_vx: false,
                jump_plus_vx: false,
            },
            Mode::SuperChipModern => Quirks {
                mode: Mode::SuperChipModern,
                mode_label: "SuperChip-Modern".to_string(),
                vf_reset: false,
                load_store_index_increase: false,
                display_wait: false,
                clipping: true,
                shifting_vx: true,
                jump_plus_vx: true,
            },
            Mode::SuperChipLegacy => Quirks {
                mode: Mode::SuperChipLegacy,
                mode_label: "SuperChip-Legacy".to_string(),
                vf_reset: false,
                load_store_index_increase: false,
                display_wait: true,
                clipping: true,
                shifting_vx: true,
                jump_plus_vx: true,
            },
            Mode::XoChip => Quirks {
                mode: Mode::XoChip,
                mode_label: "xo-chip".to_string(),
                vf_reset: false,
                load_store_index_increase: true,
                display_wait: false,
                clipping: false,
                shifting_vx: false,
                jump_plus_vx: false,
            },
        }
    }
}
