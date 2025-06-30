use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    #[error("invalid state transition")]
    InvalidStateError,
}

pub type Result<T> = std::result::Result<T, ParseError>;
