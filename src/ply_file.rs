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

/// Find the position of the last byte in "end_header\n"
fn find_header_end(buffer: &[u8]) -> Option<usize> {
    let pattern = b"end_header\n";
    if buffer.len() < pattern.len() {
        return None;
    }
    for i in 0..=(buffer.len() - pattern.len()) {
        if &buffer[i..i + pattern.len()] == pattern {
            return Some(i + pattern.len() - 1);
        }
    }
    None
}

pub struct PlyFile {
    header: Option<PlyHeader>,
    current_element_index: usize,
    rows_parsed: usize,
    data_buffer: Vec<u8>,
}

impl PlyFile {
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

            if let Some(end_pos) = find_header_end(available_data) {
                let header_data = available_data[..=end_pos].to_vec();
                let leftover_data = available_data[end_pos + 1..].to_vec();

                let cursor = std::io::Cursor::new(&header_data);
                let mut reader = std::io::BufReader::new(cursor);

                if let Ok(header) = PlyHeader::parse(&mut reader) {
                    self.header = Some(header);
                    self.data_buffer = leftover_data;
                }
            }
        }
        self.header.as_ref()
    }

    /// Parse the next chunk of elements from the buffer.
    pub fn next_chunk<T>(&mut self) -> Result<Vec<T>, PlyError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let _ = self.header();

        // Make sure header is parsed
        let Some(header) = &self.header else {
            return Err(PlyError::InvalidHeader(
                "Not in element parsing state".into(),
            ));
        };

        let mut cursor = Cursor::new(&self.data_buffer);

        // Check if we've moved past all elements, if so error that we've run out of elements.
        if self.current_element_index >= header.elements.len() {
            return Err(PlyError::MissingElement);
        }

        // Find the element definition and clone it to avoid borrowing issues
        // TODO: Ideally, we wouldn't need to clone the element def here.
        let element_def = header.elements[self.current_element_index].clone();
        let mut elements = Vec::with_capacity(64);

        // TODO: Seperate loop per format.
        // Parse lines from the buffer. Try the maximum rows we have remaining.
        for _ in self.rows_parsed..element_def.row_count {
            let start_cursor_pos = cursor.position();

            // TODO: This if should hopefully be moved out of the loop automatically, but maybe
            // double check if that's the case.
            let elem = match header.format {
                PlyFormat::Ascii => T::deserialize(&mut RowDeserializer::new(
                    AsciiValReader::new(&mut cursor),
                    &element_def,
                )),
                PlyFormat::BinaryLittleEndian => T::deserialize(&mut RowDeserializer::new(
                    BinValReader::<_, LittleEndian>::new(&mut cursor),
                    &element_def,
                )),
                PlyFormat::BinaryBigEndian => T::deserialize(&mut RowDeserializer::new(
                    BinValReader::<_, BigEndian>::new(&mut cursor),
                    &element_def,
                )),
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
                Err(e) => return Err(e), // Other errors
            }

            // Break when parsed all elements.
            self.rows_parsed += 1;
            if self.rows_parsed >= element_def.row_count {
                break;
            }
        }

        // If we've parsed all elements move to the next element.
        if self.rows_parsed >= element_def.row_count {
            self.rows_parsed = 0;
            self.current_element_index += 1;
        }

        // Remove consumed bytes from buffer
        if cursor.position() > 0 {
            self.data_buffer.drain(..cursor.position() as usize);
        }

        Ok(elements)
    }
}

impl Default for PlyFile {
    fn default() -> Self {
        Self::new()
    }
}
