use core::fmt;
use std::{
    num::{ParseFloatError, ParseIntError},
    string::FromUtf8Error,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid ascii data: {0}")]
    InvalidAscii(#[from] FromUtf8Error),

    #[error("Invalid PLY header: {0}")]
    InvalidHeader(String),

    #[error("Unsupported PLY format: {0}")]
    UnsupportedFormat(String),

    #[error("Parse error: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("Parse error: {0}")]
    ParseFloatError(#[from] ParseFloatError),

    #[error("Property type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Missing required element.")]
    MissingElement,

    #[error("Row deserialization requires struct or map")]
    RowMustBeStructOrMap,

    #[error("Property type mismatch: expected list but found scalar")]
    ExpectedListProperty,

    #[error("Failed to read ASCII token")]
    NoTokenFound,

    #[error("UTF-8 encoding error: {0}")]
    Utf8Encoding(String),
}

impl serde::de::Error for PlyError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        PlyError::Utf8Encoding(msg.to_string())
    }
}

impl serde::ser::Error for PlyError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        PlyError::Utf8Encoding(msg.to_string())
    }
}
