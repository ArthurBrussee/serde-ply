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

    #[error("Error parsing integer: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("Error parsing float: {0}")]
    ParseFloatError(#[from] ParseFloatError),

    #[error("Property type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Missing required element.")]
    MissingElement,

    #[error("Invalid ply structure")]
    InvalidStructure,

    #[error("Property type mismatch: expected list but found scalar")]
    ExpectedListProperty,

    #[error("Failed to read next ASCII property")]
    NoPropertyFound,

    #[error("UTF-8 encoding error: {0}")]
    Utf8Encoding(String),

    #[error("Unsupported type: {0}")]
    UnsupportedType(String),

    #[error("Too many properties provided for element")]
    TooManyProperties,
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
