//! PLY file serialization.

use std::io::Write;

use serde::{ser::Error, Serialize};

use crate::{
    ser::{header_collector::HeaderCollector, ply_file::PlyReaderSerializer},
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
    options: SerializeOptions,
    mut writer: impl Write,
) -> Result<(), SerializeError>
where
    T: Serialize,
{
    let format = options.format;
    val.serialize(&mut HeaderCollector::new(options, &mut writer))?;
    val.serialize(&mut PlyReaderSerializer::new(format, &mut writer))?;
    Ok(())
}

/// Serialize PLY data to bytes.
///
/// Returns the complete PLY file as a byte vector in the specified format.
pub fn to_bytes<T>(val: &T, options: SerializeOptions) -> Result<Vec<u8>, SerializeError>
where
    T: Serialize,
{
    let mut buf = vec![];
    to_writer(val, options, &mut buf)?;
    Ok(buf)
}

/// Serialize PLY data to a string.
///
/// This only works with ASCII since binary data cannot be represented as valid UTF-8.
/// Returns the complete PLY file as a string.
pub fn to_string<T>(val: &T, options: SerializeOptions) -> Result<String, SerializeError>
where
    T: Serialize,
{
    if options.format != PlyFormat::Ascii {
        return Err(SerializeError::custom(
            "Cannot serialize binary PLY to string",
        ));
    }
    String::from_utf8(to_bytes(val, options)?).map_err(|e| SerializeError::custom(e.to_string()))
}

/// Options when serializing PLY files.
///
/// This is a builder struct that lets you set the ply format and other metadata.
pub struct SerializeOptions {
    format: PlyFormat,
    comments: Vec<String>,
    obj_info: Vec<String>,
}

impl SerializeOptions {
    /// Create a new [`SerializeOptions`] with the given format.
    pub fn new(format: PlyFormat) -> Self {
        Self {
            format,
            comments: Vec::new(),
            obj_info: Vec::new(),
        }
    }

    /// Default [`SerializeOptions`] for ASCII format.
    pub fn ascii() -> Self {
        Self::new(PlyFormat::Ascii)
    }

    /// Default [`SerializeOptions`] for binary (little-endian) format.
    pub fn binary_le() -> Self {
        Self::new(PlyFormat::BinaryLittleEndian)
    }

    /// Default [`SerializeOptions`] for binary (big-endian) format.
    pub fn binary_be() -> Self {
        Self::new(PlyFormat::BinaryBigEndian)
    }

    /// Add comments to be serialized.
    pub fn with_comments(mut self, comments: Vec<String>) -> Self {
        self.comments.extend(comments);
        self
    }

    /// Add `obj_info` to be serialized.
    ///
    /// These are essentially the same as comments but often recognized seperately
    /// by ply readers.
    pub fn with_obj_info(mut self, obj_info: Vec<String>) -> Self {
        self.obj_info.extend(obj_info);
        self
    }
}
