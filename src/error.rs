use core::fmt;

use thiserror::Error;

#[derive(Error, Debug)]
#[error("Error while deserializing ply: {0}")]
pub struct DeserializeError(#[from] pub std::io::Error);

#[derive(Error, Debug)]
#[error("Error while serializing ply: {0}")]
pub struct SerializeError(#[from] pub std::io::Error);

impl serde::de::Error for DeserializeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        DeserializeError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            msg.to_string(),
        ))
    }
}

impl serde::ser::Error for SerializeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        SerializeError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            msg.to_string(),
        ))
    }
}
