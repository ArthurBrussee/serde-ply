//! PLY file deserialization.

pub(crate) mod ply_file;
pub(crate) use row::*;
pub(crate) mod chunked;
mod row;

pub mod val_reader;
use std::io::{BufRead, BufReader, Cursor};

pub use ply_file::PlyFileDeserializer;
use serde::Deserialize;

use crate::DeserializeError;

/// Deserialize PLY data from a reader.
///
/// This is the primary entry point for deserializing complete PLY files.
pub fn from_reader<'a, T>(reader: impl BufRead) -> Result<T, DeserializeError>
where
    T: Deserialize<'a>,
{
    let mut deserializer = PlyFileDeserializer::from_reader(reader)?;
    let t: T = T::deserialize(&mut deserializer)?;
    Ok(t)
}

/// Deserialize PLY data from a string.
///
/// Convenience function for parsing PLY data from bytes.
pub fn from_bytes<'a, T>(bytes: &[u8]) -> Result<T, DeserializeError>
where
    T: Deserialize<'a>,
{
    let cursor = Cursor::new(bytes);
    let buf_read = BufReader::new(cursor);
    from_reader(buf_read)
}

/// Deserialize PLY data from a string.
///
/// Convenience function for parsing PLY data from strings.
/// Only works for ASCII format PLY files.
pub fn from_str<'a, T>(str: &str) -> Result<T, DeserializeError>
where
    T: Deserialize<'a>,
{
    from_bytes(str.as_bytes())
}
