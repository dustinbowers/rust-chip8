use std::fmt;

#[derive(Debug)]
pub enum CoreErrorType {
    InvalidOpcode(u16, u16),
    StackOverflow(u16, u16),
    InvalidMemoryPtr(u16, usize),
    InvalidMemoryAccess(u16, usize),
    InvalidRom(String),
}
impl fmt::Display for CoreErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CoreErrorType::InvalidOpcode(pc, code) => {
                write!(f, " Invalid Opcode: 0x{:04X}\nPC: 0x{:04X}", code, pc)
            }
            CoreErrorType::StackOverflow(pc, addr) => {
                write!(f, "Stack Overflow - SP:0x{:04X}\nPC: 0x{:04X} ", addr, pc)
            }
            CoreErrorType::InvalidMemoryPtr(pc, ptr) => {
                write!(f, "Invalid Memory Ptr - I:0x{:08X}\nPC: 0x{:04X}", ptr, pc)
            }
            CoreErrorType::InvalidMemoryAccess(pc, addr) => write!(
                f,
                "Invalid Memory Access from: 0x{:08X}\nPC: 0x{:04X}",
                addr, pc
            ),
            CoreErrorType::InvalidRom(ref err_str) => write!(f, "Invalid ROM: {}", err_str),
        }
    }
}

#[derive(Debug)]
pub struct CoreError {
    pub error_type: CoreErrorType,
    pub info: String,
}
impl CoreError {
    pub fn new(info: String, error_type: CoreErrorType) -> Self {
        Self { info, error_type }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = format!("Error Type: {}\nInfo: {}", self.error_type, self.info);
        write!(f, "{}", out)
    }
}
impl std::error::Error for CoreError {}
