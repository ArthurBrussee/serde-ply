use crate::{
    de::{find_header_end, AsciiDirectElementDeserializer},
    AsciiElementDeserializer, BinaryElementDeserializer, ElementDef, FormatDeserializer, PlyError,
    PlyFormat, PlyHeader, PropertyType,
};
use byteorder::ByteOrder;
use serde::Deserialize;

/// TODO: Instead of doing this, we should track sizes WHILE parsing, as then we know the size of lists.
/// Calculate the size in bytes of a binary element based on its properties
fn calculate_binary_element_size(properties: &[PropertyType]) -> Result<usize, PlyError> {
    let mut size = 0;

    for property in properties {
        match property {
            PropertyType::Scalar { data_type, .. } => {
                size += data_type.size_bytes();
            }
            PropertyType::List { .. } => {
                return Err(PlyError::Serde(
                    "List properties not supported in chunked binary parsing - use sequential parsing instead".to_string()
                ));
            }
        }
    }

    Ok(size)
}

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

    /// Check if all elements of a specific type have been parsed
    pub fn is_element_complete(&self) -> bool {
        self.elements_parsed_in_current >= self.header.elements[self.current_element_index].count
    }

    pub fn all_elements_complete(&self) -> bool {
        self.current_element_index >= self.header.elements.len()
    }

    fn parse_ascii_chunk<T>(
        &mut self,
        element_def: &ElementDef,
        buffer: &[u8],
    ) -> Result<ParseChunk<T>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let mut elements = Vec::new();
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
        // TODO: This doesn't work with lists to calculate a binary size upfront. We should
        // instead properly handle early EOF in deserializer.next_element(). This is
        // important anyway for correctness.
        let element_size = calculate_binary_element_size(&element_def.properties)?;
        let remaining_elements = element_def.count - self.elements_parsed_in_current;
        let available_elements = buffer.len() / element_size;
        let elements_to_parse = remaining_elements.min(available_elements);

        if elements_to_parse == 0 {
            return Ok(ParseChunk::Advance(Vec::new(), 0)); // Not enough data for complete elements
        }

        let bytes_to_consume = elements_to_parse * element_size;

        let mut elements = Vec::with_capacity(elements_to_parse);
        let cursor = std::io::Cursor::new(&buffer[..bytes_to_consume]);
        let mut deserializer = BinaryElementDeserializer::<_, E>::new(
            cursor,
            elements_to_parse,
            element_def.properties.clone(),
        );

        while let Some(element) = deserializer.next_element::<T>()? {
            elements.push(element);
        }

        self.elements_parsed_in_current += elements.len();

        Ok(ParseChunk::Advance(elements, bytes_to_consume))
    }

    fn parse_ascii_line<T>(&self, line: &str, properties: &[PropertyType]) -> Result<T, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let tokens: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

        let mut deserializer = AsciiElementDeserializer {
            reader: std::io::Cursor::new(line),
            elements_read: 0,
            element_count: 1,
            current_line_tokens: tokens,
            token_index: 0,
            properties: properties.to_vec(),
        };

        let direct_deserializer = AsciiDirectElementDeserializer {
            parent: &mut deserializer,
        };

        T::deserialize(direct_deserializer)
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

    pub fn feed_data(&mut self, chunk: &[u8]) {
        self.data_buffer.extend(chunk);
        self.try_parse_header();
    }

    pub fn header(&self) -> Option<&PlyHeader> {
        match &self.state {
            ParseState::ParsingElements { parser } => Some(&parser.header),
            _ => None,
        }
    }

    pub fn is_finished(&self) -> bool {
        match &self.state {
            ParseState::ParsingElements { parser } => parser.all_elements_complete(),
            _ => false,
        }
    }

    pub fn next_chunk<T>(&mut self) -> Result<Option<Vec<T>>, PlyError>
    where
        T: for<'de> Deserialize<'de>,
    {
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
                "Not in element parsing state".to_string(),
            )),
        }
    }

    pub fn is_element_complete(&self) -> bool {
        match &self.state {
            ParseState::ParsingElements { parser } => parser.is_element_complete(),
            _ => false,
        }
    }

    fn try_parse_header(&mut self) {
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
    }
}

impl Default for PlyFile {
    fn default() -> Self {
        Self::new()
    }
}
