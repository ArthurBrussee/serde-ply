use core::fmt;

use thiserror::Error;

#[derive(Error, Debug)]
#[error("IO error: {0}")]
pub struct PlyError(#[from] pub std::io::Error);

impl serde::de::Error for PlyError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        PlyError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            msg.to_string(),
        ))
    }
}

impl serde::ser::Error for PlyError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        PlyError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            msg.to_string(),
        ))
    }
}
