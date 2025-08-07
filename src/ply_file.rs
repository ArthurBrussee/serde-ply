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

#[derive(Debug)]
pub(crate) struct PlyFileParser {
    pub header: PlyHeader,
    pub current_element_index: usize,
    pub rows_parsed: usize,
}

pub(crate) enum ParseChunk<T> {
    Chunk(Vec<T>),
    Complete,
}

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

impl PlyFileParser {
    /// Parse elements of a specific type from buffered data. This returns Ok(Vec) as long as there are elements remaining.
    /// When no more elemnts remain, this will return None.
    pub fn parse_chunk<T>(
        &mut self,
        mut cursor: &mut Cursor<&Vec<u8>>,
    ) -> Result<ParseChunk<T>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        // Check if we've moved past all elements
        if self.current_element_index >= self.header.elements.len() {
            return Ok(ParseChunk::Complete);
        }

        // Find the element definition and clone it to avoid borrowing issues
        // TODO: Ideally, we wouldn't need to clone the element def here.
        let mut element_def = self.header.elements[self.current_element_index].clone();

        while self.rows_parsed >= element_def.row_count {
            // Advance to the next element.
            self.current_element_index += 1;
            self.rows_parsed = 0;

            // Check if we've moved past all elements
            if self.current_element_index >= self.header.elements.len() {
                return Ok(ParseChunk::Complete);
            }

            element_def = self.header.elements[self.current_element_index].clone();
        }

        let mut elements = Vec::with_capacity(64);

        // TODO: Seperate loop per format.
        // Parse lines from the buffer.
        loop {
            let start_cursor_pos = cursor.position();

            // TODO: This if should hopefully be moved out of the loop automatically, but maybe
            // double check if that's the case.
            let elem = match self.header.format {
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

        Ok(ParseChunk::Chunk(elements))
    }
}

#[derive(Debug)]
enum ParseState {
    WaitingForHeader,
    ParsingElements { parser: PlyFileParser },
}

pub struct PlyFile {
    state: ParseState,
    data_buffer: Vec<u8>,
}

impl PlyFile {
    pub fn new() -> Self {
        Self {
            state: ParseState::WaitingForHeader,
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
        if let ParseState::WaitingForHeader = &mut self.state {
            let available_data = &self.data_buffer;

            if let Some(end_pos) = find_header_end(available_data) {
                let header_data = available_data[..=end_pos].to_vec();
                let leftover_data = available_data[end_pos + 1..].to_vec();

                let cursor = std::io::Cursor::new(&header_data);
                let mut reader = std::io::BufReader::new(cursor);

                if let Ok(header) = PlyHeader::parse(&mut reader) {
                    let file_parser = PlyFileParser {
                        header,
                        current_element_index: 0,
                        rows_parsed: 0,
                    };

                    self.state = ParseState::ParsingElements {
                        parser: file_parser,
                    };
                    self.data_buffer = leftover_data;
                }
            }
        }
        match &self.state {
            ParseState::ParsingElements { parser } => Some(&parser.header),
            _ => None,
        }
    }

    /// Parse the next chunk of elements from the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_ply::PlyFile;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct Vertex { x: f32, y: f32, z: f32 }
    ///
    /// let mut ply_file = PlyFile::new();
    ///
    /// // Write complete PLY data
    /// let ply_data = b"ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nend_header\n1.0 2.0 3.0\n";
    /// ply_file.buffer_mut().extend_from_slice(ply_data);
    ///
    /// // Parse vertices as they become available
    /// if let Some(vertices) = ply_file.next_chunk::<Vertex>().unwrap() {
    ///     for vertex in vertices {
    ///         println!("Vertex: {:?}", vertex);
    ///     }
    /// }
    /// ```
    pub fn next_chunk<T>(&mut self) -> Result<Option<Vec<T>>, PlyError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Make sure header is parsed
        let _ = self.header();

        let mut cursor = Cursor::new(&self.data_buffer);

        match &mut self.state {
            ParseState::ParsingElements { parser } => {
                match parser.parse_chunk::<T>(&mut cursor)? {
                    ParseChunk::Chunk(elements) => {
                        // Remove consumed bytes from buffer
                        if cursor.position() > 0 {
                            self.data_buffer.drain(..cursor.position() as usize);
                        }
                        if elements.is_empty() {
                            Ok(None)
                        } else {
                            Ok(Some(elements))
                        }
                    }
                    ParseChunk::Complete => Ok(None),
                }
            }
            _ => Err(PlyError::InvalidHeader(
                "Not in element parsing state".into(),
            )),
        }
    }
}

impl Default for PlyFile {
    fn default() -> Self {
        Self::new()
    }
}
