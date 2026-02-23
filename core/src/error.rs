#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid length, got {}, expected {}", got, expected)]
    InvalidLength { got: usize, expected: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
