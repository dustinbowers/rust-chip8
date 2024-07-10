use std::fmt;

#[derive(Debug)]
pub enum CoreErrorType {
    InvalidOpcode(u16),
    StackOverflow(u16),
    InvalidMemoryPtr(usize),
    InvalidMemoryAccess(usize),
    InvalidRom(String),
}

#[derive(Debug)]
pub struct CoreError {
    pub error_type: CoreErrorType,
    pub info: String,
}
impl CoreError {
    pub fn new(info: String, error_type: CoreErrorType) -> Self {
        Self {
            info,
            error_type,
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = format!("Error Type: {:?}\nInfo: {}", self.error_type, self.info);
        write!(f, "{}", out)
    }
}
impl std::error::Error for CoreError {}
