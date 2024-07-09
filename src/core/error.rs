use std::fmt;

#[derive(Debug)]
pub enum CoreErrorType {
    InvalidOpcode(u16),
    InvalidRom(String),
}

#[derive(Debug)]
pub struct CoreError {
    pub error_type: CoreErrorType,
    pub file: String,
}
impl CoreError {
    pub fn new(file: &str, error_type: CoreErrorType) -> Self {
        Self {
            file: file.to_string(),
            error_type,
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = format!("Error Type: {:#?}\nFile: {}", self.error_type, self.file);
        write!(f, "{}", out)
    }
}
impl std::error::Error for CoreError {}
