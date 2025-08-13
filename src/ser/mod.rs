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
/// The writer receives the raw PLY bytes.
///
/// # Example
/// ```rust
/// use serde::Serialize;
/// use serde_ply::{to_writer, SerializeOptions};
/// use std::io::Cursor;
///
/// #[derive(Serialize)]
/// struct Vertex { x: f32, y: f32, z: f32 }
///
/// #[derive(Serialize)]
/// struct Mesh { vertex: Vec<Vertex> }
///
/// let mesh = Mesh {
///     vertex: vec![Vertex { x: 0.0, y: 0.0, z: 0.0 }]
/// };
///
/// let mut buffer = Vec::new();
/// to_writer(&mesh, SerializeOptions::ascii(), &mut buffer)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
/// Works with both ASCII and binary formats.
///
/// # Example
/// ```rust
/// use serde::Serialize;
/// use serde_ply::{to_bytes, SerializeOptions};
///
/// #[derive(Serialize)]
/// struct Point { x: f32, y: f32 }
///
/// #[derive(Serialize)]
/// struct Points { vertex: Vec<Point> }
///
/// let points = Points {
///     vertex: vec![Point { x: 1.0, y: 2.0 }]
/// };
///
/// let bytes = to_bytes(&points, SerializeOptions::binary_le())?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
/// This only works with ASCII format since binary data cannot be represented as valid UTF-8.
/// Returns the complete PLY file as a string.
///
/// # Example
/// ```rust
/// use serde::Serialize;
/// use serde_ply::{to_string, SerializeOptions};
///
/// #[derive(Serialize)]
/// struct Vertex { x: f32, y: f32, z: f32 }
///
/// #[derive(Serialize)]
/// struct Mesh { vertex: Vec<Vertex> }
///
/// let mesh = Mesh {
///     vertex: vec![Vertex { x: 0.0, y: 0.0, z: 0.0 }]
/// };
///
/// let ply_string = to_string(&mesh, SerializeOptions::ascii())?;
/// assert!(ply_string.starts_with("ply\n"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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

/// Options for PLY file serialization.
///
/// Builder struct for configuring PLY output format and metadata like comments.
/// Use the convenience methods like [`Self::ascii()`] for common configurations.
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

    /// Add comments to the PLY header.
    ///
    /// Comments appear in the header section and are often used for metadata.
    pub fn with_comments(mut self, comments: Vec<String>) -> Self {
        self.comments.extend(comments);
        self
    }

    /// Add obj_info lines to the PLY header.
    ///
    /// Similar to comments but may be treated differently by some PLY readers.
    /// Often used for application-specific metadata.
    pub fn with_obj_info(mut self, obj_info: Vec<String>) -> Self {
        self.obj_info.extend(obj_info);
        self
    }
}
