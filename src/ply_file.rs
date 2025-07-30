use crate::{
    de::{find_header_end, ChunkedFileParser, ChunkedHeaderParser},
    ElementDef, PlyError, PlyHeader,
};
use serde::Deserialize;
use std::collections::VecDeque;

#[derive(Debug)]
enum ParseState {
    WaitingForHeader(ChunkedHeaderParser),
    ParsingElements {
        parser: ChunkedFileParser,
        current_element_index: usize,
    },
    Finished,
}

pub struct PlyFile {
    state: ParseState,
    data_buffer: VecDeque<u8>,
}

pub struct ElementReader {
    element_name: String,
    element_def: ElementDef,
    finished: bool,
}

pub trait PlyConstruct {
    fn from_ply_file(ply_file: &mut PlyFile) -> Result<Self, PlyError>
    where
        Self: Sized;
}

impl PlyFile {
    pub fn new() -> Self {
        Self {
            state: ParseState::WaitingForHeader(ChunkedHeaderParser::new()),
            data_buffer: VecDeque::new(),
        }
    }

    pub fn feed_data(&mut self, chunk: &[u8]) {
        self.data_buffer.extend(chunk);
        self.try_parse_header();
    }

    pub fn header(&self) -> Option<&PlyHeader> {
        match &self.state {
            ParseState::ParsingElements { parser, .. } => Some(&parser.header),
            _ => None,
        }
    }

    pub fn is_header_ready(&self) -> bool {
        matches!(self.state, ParseState::ParsingElements { .. })
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.state, ParseState::Finished)
    }

    pub fn element_reader(&self, element_name: &str) -> Result<ElementReader, PlyError> {
        let header = self
            .header()
            .ok_or_else(|| PlyError::InvalidHeader("Header not yet parsed".to_string()))?;

        let element_def = header
            .get_element(element_name)
            .ok_or_else(|| PlyError::MissingElement(element_name.to_string()))?
            .clone();

        Ok(ElementReader {
            element_name: element_name.to_string(),
            element_def,
            finished: false,
        })
    }

    pub fn parse_next_chunk<T>(&mut self, element_name: &str) -> Result<Option<Vec<T>>, PlyError>
    where
        T: for<'de> Deserialize<'de>,
    {
        match &mut self.state {
            ParseState::ParsingElements { parser, .. } => {
                // Transfer any buffered data to the parser
                let available_data: Vec<u8> = self.data_buffer.drain(..).collect();
                parser.add_data(&available_data);

                parser.parse_chunk::<T>(element_name)
            }
            _ => Err(PlyError::InvalidHeader(
                "Not in element parsing state".to_string(),
            )),
        }
    }

    pub fn is_element_complete(&self, element_name: &str) -> bool {
        match &self.state {
            ParseState::ParsingElements { parser, .. } => parser.is_element_complete(element_name),
            _ => false,
        }
    }

    pub fn advance_to_next_element(&mut self) -> Result<(), PlyError> {
        match &mut self.state {
            ParseState::ParsingElements {
                parser,
                current_element_index,
            } => {
                parser.advance_to_next_element();
                *current_element_index += 1;

                // Check if we've processed all elements
                if *current_element_index >= parser.header.elements.len() {
                    self.state = ParseState::Finished;
                }
                Ok(())
            }
            _ => Err(PlyError::InvalidHeader(
                "Not in element parsing state".to_string(),
            )),
        }
    }

    fn try_parse_header(&mut self) {
        if let ParseState::WaitingForHeader(_header_parser) = &mut self.state {
            // Transfer data from our buffer to the header parser
            let available_data: Vec<u8> = self.data_buffer.drain(..).collect();

            // Look for header end marker
            if let Some(end_pos) = find_header_end(&available_data) {
                let header_data = available_data[..=end_pos].to_vec();
                let leftover_data = available_data[end_pos + 1..].to_vec();

                // Try to parse the header
                let cursor = std::io::Cursor::new(&header_data);
                let mut reader = std::io::BufReader::new(cursor);

                if let Ok(header) = PlyHeader::parse(&mut reader) {
                    let file_parser = ChunkedFileParser {
                        header,
                        buffer: leftover_data,
                        current_element_index: 0,
                        elements_parsed_in_current: 0,
                    };

                    self.state = ParseState::ParsingElements {
                        parser: file_parser,
                        current_element_index: 0,
                    };
                    return;
                }
            }

            // Put data back if we couldn't parse header yet
            let mut temp_buffer = VecDeque::from(available_data);
            temp_buffer.append(&mut self.data_buffer);
            self.data_buffer = temp_buffer;
        }
    }
}

impl Default for PlyFile {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementReader {
    pub fn next_chunk<T>(&mut self, ply_file: &mut PlyFile) -> Result<Option<Vec<T>>, PlyError>
    where
        T: for<'de> Deserialize<'de>,
    {
        if self.finished {
            return Ok(None);
        }

        let result = ply_file.parse_next_chunk::<T>(&self.element_name)?;

        // Check if this element type is complete
        if ply_file.is_element_complete(&self.element_name) {
            self.finished = true;
        }

        Ok(result)
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn element_def(&self) -> &ElementDef {
        &self.element_def
    }
}

impl<T> PlyConstruct for Vec<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn from_ply_file(ply_file: &mut PlyFile) -> Result<Self, PlyError> {
        // This is a basic implementation that reads the first element type
        let element_name = {
            let header = ply_file
                .header()
                .ok_or_else(|| PlyError::InvalidHeader("Header not ready".to_string()))?;

            let first_element = header
                .elements
                .first()
                .ok_or_else(|| PlyError::MissingElement("No elements defined".to_string()))?;

            first_element.name.clone()
        };

        let mut reader = ply_file.element_reader(&element_name)?;
        let mut result = Vec::new();

        while let Some(chunk) = reader.next_chunk::<T>(ply_file)? {
            result.extend(chunk);
        }

        Ok(result)
    }
}
