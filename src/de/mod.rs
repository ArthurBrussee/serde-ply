mod ply_file;
mod row;

pub mod val_reader;
use std::io::{BufRead, BufReader, Cursor};

pub use ply_file::PlyFileDeserializer;
pub use row::*;
use serde::Deserialize;

use crate::PlyError;

// TODO: Make this compatible with :Read instead of BufRead?
// This is just for a read_line when parsing the header but really
// don't need BufRead otherwise.
pub fn from_reader<'a, T>(reader: impl BufRead) -> Result<T, PlyError>
where
    T: Deserialize<'a>,
{
    let mut deserializer = PlyFileDeserializer::from_reader(reader)?;
    let t: T = T::deserialize(&mut deserializer)?;
    Ok(t)
}

pub fn from_str<'a, T>(str: &str) -> Result<T, PlyError>
where
    T: Deserialize<'a>,
{
    let cursor = Cursor::new(str);
    let buf_read = BufReader::new(cursor);
    from_reader(buf_read)
}
