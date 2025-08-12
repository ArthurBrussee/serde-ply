//! PLY file serialization.

use std::io::Write;

use serde::{ser::Error, Serialize};

use crate::{
    ser::{header_collector::HeaderCollector, ply_file::PlyFileSerializer},
    PlyFormat, SerializeError,
};

mod header_collector;
mod ply_file;
mod row;

pub mod val_writer;

/// Serialize PLY data to a writer.
///
/// Writes the complete PLY file including header and data in the specified format.
pub fn to_writer<T>(
    val: &T,
    format: PlyFormat,
    mut writer: impl Write,
    comments: Vec<String>,
) -> Result<(), SerializeError>
where
    T: Serialize,
{
    val.serialize(&mut HeaderCollector::new(format, &mut writer, comments))?;
    val.serialize(&mut PlyFileSerializer::new(format, &mut writer))?;
    Ok(())
}

/// Serialize PLY data to bytes.
///
/// Returns the complete PLY file as a byte vector in the specified format.
pub fn to_bytes<T>(
    val: &T,
    format: PlyFormat,
    comments: Vec<String>,
) -> Result<Vec<u8>, SerializeError>
where
    T: Serialize,
{
    let mut buf = vec![];
    val.serialize(&mut HeaderCollector::new(format, &mut buf, comments))?;
    val.serialize(&mut PlyFileSerializer::new(format, &mut buf))?;
    Ok(buf)
}

/// Serialize PLY data to a string.
///
/// This always uses the ASCII format since binary data cannot be represented as valid UTF-8.
/// Returns the complete PLY file as a string.
pub fn to_string<T>(val: &T, comments: Vec<String>) -> Result<String, SerializeError>
where
    T: Serialize,
{
    String::from_utf8(to_bytes(val, PlyFormat::Ascii, comments)?)
        .map_err(|e| SerializeError::custom(e.to_string()))
}
