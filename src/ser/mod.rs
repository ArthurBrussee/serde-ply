use std::io::Write;

use serde::Serialize;

use crate::{
    ser::{header_collector::HeaderCollector, ply_file::PlyFileSerializer},
    PlyError, PlyFormat,
};

// mod ply_file;
mod header_collector;
mod ply_file;
mod row;

pub mod val_writer;

pub fn to_writer<T>(
    val: &T,
    format: PlyFormat,
    mut writer: impl Write,
    comments: Vec<String>,
) -> Result<(), PlyError>
where
    T: Serialize,
{
    val.serialize(&mut HeaderCollector::new(format, &mut writer, comments))?;
    val.serialize(&mut PlyFileSerializer::new(format, &mut writer))?;
    Ok(())
}

/// Serializes
pub fn to_bytes<T>(val: &T, format: PlyFormat, comments: Vec<String>) -> Result<Vec<u8>, PlyError>
where
    T: Serialize,
{
    let mut buf = vec![];
    val.serialize(&mut HeaderCollector::new(format, &mut buf, comments))?;
    val.serialize(&mut PlyFileSerializer::new(format, &mut buf))?;
    Ok(buf)
}
