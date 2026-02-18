#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    InvalidLength,
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::InvalidLength => write!(f, "invalid length"),
        }
    }
}

impl std::error::Error for CodecError {}
