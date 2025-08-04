use crate::{
    de::{
        find_header_end, AsciiElementDeserializer, AsciiRowMapDeserializer,
        BinaryElementDeserializer,
    },
    ElementDef, PlyError, PlyFormat, PlyHeader, PlyProperty,
};
use byteorder::ByteOrder;
use serde::Deserialize;
use std::io::BufRead;

#[derive(Debug)]
pub(crate) struct ChunkedFileParser {
    pub header: PlyHeader,
    pub current_element_index: usize,
    pub elements_parsed_in_current: usize,
}

pub(crate) enum ParseChunk<T> {
    Advance(Vec<T>, usize),
    Done,
}

impl ChunkedFileParser {
    /// Parse elements of a specific type from buffered data. This returns Ok(Vec) as long as there are elements remaining.
    /// When no more elemnts remain, this will return None.
    pub fn parse_chunk<T>(&mut self, buffer: &[u8]) -> Result<ParseChunk<T>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        // Check if we've moved past all elements
        if self.current_element_index >= self.header.elements.len() {
            return Ok(ParseChunk::Done);
        }

        // Find the element definition and clone it to avoid borrowing issues
        // TODO: Ideally, we wouldn't need to clone the element def here.
        let element_def = self.header.elements[self.current_element_index].clone();

        let ret = match self.header.format {
            PlyFormat::Ascii => self.parse_ascii_chunk::<T>(&element_def, buffer),
            PlyFormat::BinaryLittleEndian => {
                self.parse_binary_chunk::<T, byteorder::LittleEndian>(&element_def, buffer)
            }
            PlyFormat::BinaryBigEndian => {
                self.parse_binary_chunk::<T, byteorder::BigEndian>(&element_def, buffer)
            }
        };

        // Check if we've already finished parsing this element type
        if self.elements_parsed_in_current >= element_def.count {
            // Advance to the next element.
            self.current_element_index += 1;
            self.elements_parsed_in_current = 0;
        }

        ret
    }

    fn parse_ascii_chunk<T>(
        &mut self,
        element_def: &ElementDef,
        buffer: &[u8],
    ) -> Result<ParseChunk<T>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let remaining_elements = element_def.count - self.elements_parsed_in_current;
        let mut elements = Vec::with_capacity(remaining_elements);
        let mut bytes_consumed = 0;
        let mut cursor = 0;

        // Parse lines from the buffer
        while cursor < buffer.len() && self.elements_parsed_in_current < element_def.count {
            // Find the next newline
            if let Some(newline_pos) = buffer[cursor..].iter().position(|&b| b == b'\n') {
                let line_end = cursor + newline_pos;
                let line_bytes = &buffer[cursor..line_end];

                // Convert to string, skipping invalid UTF-8
                if let Ok(line) = std::str::from_utf8(line_bytes) {
                    let line = line.trim();
                    if !line.is_empty() {
                        let element = self.parse_ascii_line::<T>(line, &element_def.properties)?;
                        elements.push(element);
                        self.elements_parsed_in_current += 1;
                    }
                }

                cursor = line_end + 1; // Move past the newline
                bytes_consumed = cursor;
            } else {
                // No complete line found, need more data
                break;
            }
        }

        Ok(ParseChunk::Advance(elements, bytes_consumed))
    }

    fn parse_binary_chunk<T, E: ByteOrder>(
        &mut self,
        element_def: &ElementDef,
        buffer: &[u8],
    ) -> Result<ParseChunk<T>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let cursor = std::io::Cursor::new(buffer);
        let mut total_bytes_consumed = 0;
        let remaining_elements = element_def.count - self.elements_parsed_in_current;
        let mut elements = Vec::with_capacity(remaining_elements);

        // Create deserializer once and reuse it
        let mut deserializer = BinaryElementDeserializer::<_, E>::new(cursor, element_def.clone());

        // Parse elements reusing the same deserializer
        for _ in 0..remaining_elements {
            let position_before = deserializer.reader.position();

            match deserializer.next_element::<T>() {
                Ok(Some(element)) => {
                    elements.push(element);
                    let position_after = deserializer.reader.position();
                    let bytes_consumed = position_after - position_before;
                    total_bytes_consumed += bytes_consumed as usize;
                }
                Ok(None) => break,
                Err(PlyError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Not enough data for this element, stop here
                    break;
                }
                Err(PlyError::NotEnoughData) => {
                    // Not enough data for this element, stop here
                    break;
                }
                Err(e) => return Err(e), // Other errors
            }
        }

        self.elements_parsed_in_current += elements.len();
        Ok(ParseChunk::Advance(elements, total_bytes_consumed))
    }

    fn parse_ascii_line<T>(&self, line: &str, properties: &[PlyProperty]) -> Result<T, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let elem_def = crate::ElementDef {
            name: "line".to_string(),
            count: 1,
            properties: properties.to_vec(),
        };
        let mut deserializer =
            AsciiElementDeserializer::new(std::io::Cursor::new(format!("{line}\n")), elem_def);

        // Read the line into the deserializer's state
        deserializer.read_ascii_line()?;

        T::deserialize(AsciiRowMapDeserializer {
            parent: &mut deserializer,
        })
    }
}

#[derive(Debug)]
enum ParseState {
    WaitingForHeader,
    ParsingElements { parser: ChunkedFileParser },
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
                    let file_parser = ChunkedFileParser {
                        header,
                        current_element_index: 0,
                        elements_parsed_in_current: 0,
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

        match &mut self.state {
            ParseState::ParsingElements { parser } => {
                match parser.parse_chunk::<T>(&self.data_buffer)? {
                    ParseChunk::Advance(elements, bytes_consumed) => {
                        // Remove consumed bytes from buffer
                        if bytes_consumed > 0 {
                            self.data_buffer.drain(..bytes_consumed);
                        }
                        if elements.is_empty() {
                            Ok(None)
                        } else {
                            Ok(Some(elements))
                        }
                    }
                    ParseChunk::Done => Ok(None),
                }
            }
            _ => Err(PlyError::InvalidHeader(
                "Not in element parsing state".into(),
            )),
        }
    }
}

impl PlyFile {
    /// Serialize elements to a PLY format string
    pub fn to_string<T>(header: &PlyHeader, elements: &[T]) -> Result<String, PlyError>
    where
        T: serde::Serialize,
    {
        if !matches!(header.format, PlyFormat::Ascii) {
            return Err(PlyError::UnsupportedFormat(
                "to_string only supports ASCII format - use to_bytes for binary formats"
                    .to_string(),
            ));
        }

        let mut buffer = Vec::new();
        crate::ser::elements_to_writer(&mut buffer, header, elements)?;
        String::from_utf8(buffer).map_err(|e| PlyError::Serde(format!("UTF-8 encoding error: {e}")))
    }

    /// Serialize elements to a PLY format byte vector
    pub fn to_bytes<T>(header: &PlyHeader, elements: &[T]) -> Result<Vec<u8>, PlyError>
    where
        T: serde::Serialize,
    {
        crate::ser::elements_to_bytes(header, elements)
    }

    /// Serialize elements to a writer
    pub fn to_writer<W, T>(writer: W, header: &PlyHeader, elements: &[T]) -> Result<(), PlyError>
    where
        W: std::io::Write,
        T: serde::Serialize,
    {
        crate::ser::elements_to_writer(writer, header, elements)
    }

    pub fn parse_elements<R, T>(
        reader: R,
        header: &PlyHeader,
        element_name: &str,
    ) -> Result<Vec<T>, PlyError>
    where
        R: BufRead,
        T: for<'de> serde::Deserialize<'de>,
    {
        let element_def = header
            .get_element(element_name)
            .ok_or_else(|| PlyError::MissingElement(element_name.into()))?;
        let mut results = Vec::with_capacity(element_def.count);

        match header.format {
            PlyFormat::Ascii => {
                let mut deserializer = AsciiElementDeserializer::new(reader, element_def.clone());
                while let Some(element) = deserializer.next_element::<T>()? {
                    results.push(element);
                }
            }
            PlyFormat::BinaryLittleEndian => {
                let mut deserializer = BinaryElementDeserializer::<_, byteorder::LittleEndian>::new(
                    reader,
                    element_def.clone(),
                );
                while let Some(element) = deserializer.next_element::<T>()? {
                    results.push(element);
                }
            }
            PlyFormat::BinaryBigEndian => {
                let mut deserializer = BinaryElementDeserializer::<_, byteorder::BigEndian>::new(
                    reader,
                    element_def.clone(),
                );
                while let Some(element) = deserializer.next_element::<T>()? {
                    results.push(element);
                }
            }
        }

        Ok(results)
    }

    pub fn elements_to_writer<W, T>(
        writer: W,
        header: &PlyHeader,
        elements: &[T],
    ) -> Result<(), PlyError>
    where
        W: std::io::Write,
        T: serde::Serialize,
    {
        crate::ser::elements_to_writer(writer, header, elements)
    }

    pub fn elements_to_bytes<T>(header: &PlyHeader, elements: &[T]) -> Result<Vec<u8>, PlyError>
    where
        T: serde::Serialize,
    {
        crate::ser::elements_to_bytes(header, elements)
    }
}

impl Default for PlyFile {
    fn default() -> Self {
        Self::new()
    }
}
