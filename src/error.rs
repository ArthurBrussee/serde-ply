use core::fmt;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Missing required element.")]
    MissingElement,

    #[error("Invalid ply structure")]
    InvalidStructure,

    #[error("Unsupported type: {0}")]
    UnsupportedType(String),
}

impl serde::de::Error for PlyError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        PlyError::UnsupportedType(msg.to_string())
    }
}

impl serde::ser::Error for PlyError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        PlyError::UnsupportedType(msg.to_string())
    }
}
