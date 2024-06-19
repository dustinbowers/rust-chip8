
#[macro_export]
macro_rules! get_nnn {
    ($opcode:expr) => {
        $opcode & 0x0FFF
    };
}

#[macro_export]
macro_rules! get_n {
    ($opcode:expr) => {
        $opcode & 0xF
    };
}

#[macro_export]
macro_rules! get_x {
    ($opcode:expr) => {
        (($opcode >> 8) & 0x0F) as usize
    };
}

#[macro_export]
macro_rules! get_y {
    ($opcode:expr) => {
        (($opcode >> 4) & 0x0F) as usize
    };
}

#[macro_export]
macro_rules! get_kk {
    ($opcode:expr) => {
        ($opcode & 0xFF) as u8
    };
}

#[macro_export]
macro_rules! invalid_opcode {
    ($opcode:expr) => {
        panic!("Invalid opcode: {:#X}", $opcode)
    }
}