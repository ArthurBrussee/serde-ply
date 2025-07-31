//! Chunked PLY file loading with interleaved data feeding and element parsing

use crate::{
    de::{find_header_end, ChunkedFileParser, ChunkedHeaderParser},
    PlyError, PlyHeader,
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

    pub fn element_reader(&self) -> Result<ElementReader, PlyError> {
        if !self.is_header_ready() {
            return Err(PlyError::InvalidHeader("Header not yet parsed".to_string()));
        }
        Ok(ElementReader { finished: false })
    }

    pub fn next_chunk<T>(&mut self) -> Result<Option<Vec<T>>, PlyError>
    where
        T: for<'de> Deserialize<'de>,
    {
        match &mut self.state {
            ParseState::ParsingElements {
                parser,
                current_element_index,
            } => {
                let available_data: Vec<u8> = self.data_buffer.drain(..).collect();
                parser.add_data(&available_data);

                let element_name = parser.header.elements[*current_element_index].name.clone();
                parser.parse_chunk::<T>(&element_name)
            }
            _ => Err(PlyError::InvalidHeader(
                "Not in element parsing state".to_string(),
            )),
        }
    }

    pub fn is_element_complete(&self) -> bool {
        match &self.state {
            ParseState::ParsingElements {
                parser,
                current_element_index,
            } => {
                let element_name = parser.header.elements[*current_element_index].name.clone();
                parser.is_element_complete(&element_name)
            }
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
            let available_data: Vec<u8> = self.data_buffer.drain(..).collect();

            if let Some(end_pos) = find_header_end(&available_data) {
                let header_data = available_data[..=end_pos].to_vec();
                let leftover_data = available_data[end_pos + 1..].to_vec();

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

        let result = ply_file.next_chunk::<T>()?;

        if ply_file.is_element_complete() {
            self.finished = true;
        }

        Ok(result)
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

impl<T> PlyConstruct for Vec<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn from_ply_file(ply_file: &mut PlyFile) -> Result<Self, PlyError> {
        if !ply_file.is_header_ready() {
            return Err(PlyError::InvalidHeader("Header not ready".to_string()));
        }

        let mut reader = ply_file.element_reader()?;
        let mut result = Vec::new();

        while let Some(chunk) = reader.next_chunk::<T>(ply_file)? {
            result.extend(chunk);
        }

        Ok(result)
    }
}
