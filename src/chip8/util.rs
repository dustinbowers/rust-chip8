/*
   CHIP8 opcodes are always exactly 2 bytes.
   These macros make it easy to examine specific parts of the opcode.
   Below is a diagram that shows what each macro is extracting

   byte1     byte2
   0000 0000 0000 0000
        x--- y--- n---
        nnn-----------
             kk-------
*/

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
macro_rules! get_n {
    ($opcode:expr) => {
        $opcode & 0xF
    };
}

#[macro_export]
macro_rules! get_nnn {
    ($opcode:expr) => {
        $opcode & 0x0FFF
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
    };
}

#[macro_export]
macro_rules! ensure_super_chip {
    ($enabled:expr) => {
        if !$enabled {
            panic!("Invalid opcode - Super-Chip only");
        }
    };
}
