use crate::{
    de::{
        val_reader::{AsciiValReader, BinValReader},
        RowDeserializer,
    },
    PlyError, PlyFormat, PlyHeader,
};
use byteorder::{BigEndian, LittleEndian};
use serde::Deserialize;
use std::io::Cursor;

pub struct ChunkPlyFile {
    header: Option<PlyHeader>,
    current_element_index: usize,
    rows_parsed: usize,
    data_buffer: Vec<u8>,
}

impl ChunkPlyFile {
    pub fn new() -> Self {
        Self {
            header: None,
            current_element_index: 0,
            rows_parsed: 0,
            data_buffer: Vec::new(),
        }
    }

    /// Get mutable access to the internal buffer for zero-copy writing.
    ///
    /// This allows any reader (including async ones) to write directly into
    /// the buffer without copyies.
    pub fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data_buffer
    }

    /// Get the parsed PLY header if available.
    ///
    /// Returns `None` if there isn't enough data to complete header parsing.
    pub fn header(&mut self) -> Option<&PlyHeader> {
        if self.header.is_none() {
            let available_data = &self.data_buffer;
            let mut cursor = Cursor::new(available_data);
            let header = PlyHeader::parse(&mut cursor);
            if let Ok(header) = header {
                self.header = Some(header);
                self.data_buffer.drain(..cursor.position() as usize);
            }
        }
        self.header.as_ref()
    }

    /// Parse the next chunk of elements from the buffer.
    pub fn next_chunk<T>(&mut self) -> Result<Vec<T>, PlyError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Make sure header is parsed
        let _ = self.header();
        let Some(header) = &self.header else {
            return Ok(vec![]);
        };

        // Check if we've moved past all elements, if so error that we've run out of elements.
        if self.current_element_index >= header.elements.len() {
            return Err(PlyError::MissingElement);
        }

        // Find the element definition and clone it to avoid borrowing issues
        let element_def = &header.elements[self.current_element_index];

        // TODO: Maybe size based on last return?
        let mut elements = Vec::with_capacity(64);
        let mut cursor = Cursor::new(&self.data_buffer);

        // TODO: Ideally we'd figure out the format OUTSIDE the loop. The optimizer probably does so anyway
        // but would be nice to be sure.
        // TODO: Maybe this could all be some custom deserialized struct that would support streaming?
        for _ in self.rows_parsed..element_def.row_count {
            let start_cursor_pos = cursor.position();

            let elem = match header.format {
                PlyFormat::Ascii => {
                    let mut val = AsciiValReader::new(&mut cursor);
                    T::deserialize(RowDeserializer::new(&mut val, element_def))
                }
                PlyFormat::BinaryLittleEndian => {
                    let mut val = BinValReader::<_, LittleEndian>::new(&mut cursor);
                    T::deserialize(RowDeserializer::new(&mut val, element_def))
                }
                PlyFormat::BinaryBigEndian => {
                    let mut val = BinValReader::<_, BigEndian>::new(&mut cursor);
                    T::deserialize(RowDeserializer::new(&mut val, element_def))
                }
            };

            match elem {
                Ok(element) => {
                    elements.push(element);
                }
                // Not enough data for this element, stop here
                Err(PlyError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    cursor.set_position(start_cursor_pos);
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        // Remove consumed bytes from buffer
        if cursor.position() > 0 {
            self.data_buffer.drain(..cursor.position() as usize);
        }

        // Break when parsed all elements.
        self.rows_parsed += elements.len();

        // If we've parsed all elements move to the next element.
        if self.rows_parsed >= element_def.row_count {
            self.rows_parsed = 0;
            self.current_element_index += 1;
        }

        Ok(elements)
    }
}

impl Default for ChunkPlyFile {
    fn default() -> Self {
        Self::new()
    }
}
