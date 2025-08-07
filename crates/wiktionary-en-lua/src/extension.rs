use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum ExtensionErrorType {
    UnknownOption,
    UnknownError,
}

impl ExtensionErrorType {
    pub fn from(code: &str) -> Self {
        match code {
            "unknown_option" => Self::UnknownOption,
            _ => Self::UnknownError,
        }
    }
}

impl Display for ExtensionErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::UnknownOption => write!(f, "unknown extension option"),
            Self::UnknownError => write!(f, "unknown extension error"),
        }
    }
}

#[derive(Debug)]
pub struct ExtensionResult {
    pub result: String,
    pub error: Option<ExtensionErrorType>,
}
