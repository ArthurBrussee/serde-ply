//! PLY file deserialization.

pub(crate) mod ply_file;
pub(crate) use row::*;
pub(crate) mod chunked;
mod row;

pub mod val_reader;
use std::io::{BufRead, BufReader, Cursor};

pub use ply_file::PlyReader;
use serde::Deserialize;

use crate::DeserializeError;

/// Deserialize PLY data from a reader.
///
/// This is the primary entry point for deserializing complete PLY files.
/// The reader should contain the full PLY file including header.
///
/// # Example
/// ```rust
/// use serde::Deserialize;
/// use std::io::{BufReader, Cursor};
///
/// #[derive(Deserialize)]
/// struct Vertex { x: f32, y: f32, z: f32 }
///
/// #[derive(Deserialize)]
/// struct Mesh { vertex: Vec<Vertex> }
///
/// let ply_data = b"ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nend_header\n1.0 2.0 3.0\n";
/// let reader = BufReader::new(Cursor::new(ply_data));
/// let mesh: Mesh = serde_ply::from_reader(reader)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn from_reader<'a, T>(reader: impl BufRead) -> Result<T, DeserializeError>
where
    T: Deserialize<'a>,
{
    let mut deserializer = PlyReader::from_reader(reader)?;
    let t: T = T::deserialize(&mut deserializer)?;
    Ok(t)
}

/// Deserialize PLY data from bytes.
///
/// Convenience function for parsing PLY data from a byte slice.
/// Works with both ASCII and binary format PLY files.
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
/// Only works for ASCII format PLY files since binary data is not valid UTF-8.
///
/// # Example
/// ```rust
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Point { x: f32, y: f32 }
///
/// #[derive(Deserialize)]
/// struct Points { vertex: Vec<Point> }
///
/// let ply = "ply\nformat ascii 1.0\nelement vertex 2\nproperty float x\nproperty float y\nend_header\n0.0 0.0\n1.0 1.0\n";
/// let points: Points = serde_ply::from_str(ply)?;
/// assert_eq!(points.vertex.len(), 2);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn from_str<'a, T>(str: &str) -> Result<T, DeserializeError>
where
    T: Deserialize<'a>,
{
    from_bytes(str.as_bytes())
}
