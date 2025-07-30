//! PLY deserializer implementation

use crate::{ElementDef, PlyError, PlyFormat, PlyHeader, PropertyType};
use byteorder::{ByteOrder, ReadBytesExt};
use serde::de::{self, DeserializeSeed, MapAccess, Visitor};

use std::{io::BufRead, marker::PhantomData};

pub trait FormatDeserializer<R> {
    fn new(reader: R, element_count: usize, properties: Vec<PropertyType>) -> Self;
    fn next_element<T>(&mut self) -> Result<Option<T>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>;
}

pub struct AsciiElementDeserializer<R> {
    reader: R,
    elements_read: usize,
    element_count: usize,
    current_line_tokens: Vec<String>,
    token_index: usize,
    properties: Vec<PropertyType>,
}

pub struct BinaryElementDeserializer<R, E> {
    reader: R,
    elements_read: usize,
    element_count: usize,
    properties: Vec<PropertyType>,
    _endian: PhantomData<E>,
}

/// Chunked header parser for async-compatible header parsing
#[derive(Debug)]
pub struct ChunkedHeaderParser {
    buffer: Vec<u8>,
    header_complete: bool,
    header: Option<PlyHeader>,
    leftover_data: Vec<u8>,
}

/// Multi-element chunked file parser for proper sequential element parsing
#[derive(Debug)]
pub struct ChunkedFileParser {
    pub header: PlyHeader,
    pub buffer: Vec<u8>,
    pub current_element_index: usize,
    pub elements_parsed_in_current: usize,
}

impl<R: BufRead> FormatDeserializer<R> for AsciiElementDeserializer<R> {
    fn new(reader: R, element_count: usize, properties: Vec<PropertyType>) -> Self {
        Self {
            reader,
            elements_read: 0,
            element_count,
            current_line_tokens: Vec::new(),
            token_index: 0,
            properties,
        }
    }

    fn next_element<T>(&mut self) -> Result<Option<T>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        if self.elements_read >= self.element_count {
            return Ok(None);
        }

        self.read_ascii_line()?;
        let element = T::deserialize(AsciiDirectElementDeserializer { parent: self })?;
        self.elements_read += 1;

        Ok(Some(element))
    }
}

impl<R: BufRead, E: ByteOrder> FormatDeserializer<R> for BinaryElementDeserializer<R, E> {
    fn new(reader: R, element_count: usize, properties: Vec<PropertyType>) -> Self {
        Self {
            reader,
            elements_read: 0,
            element_count,
            properties,
            _endian: PhantomData,
        }
    }

    fn next_element<T>(&mut self) -> Result<Option<T>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        if self.elements_read >= self.element_count {
            return Ok(None);
        }

        let element = T::deserialize(BinaryDirectElementDeserializer {
            parent: self,
            _endian: PhantomData,
        })?;
        self.elements_read += 1;

        Ok(Some(element))
    }
}

impl<R: BufRead> AsciiElementDeserializer<R> {
    fn read_ascii_line(&mut self) -> Result<(), PlyError> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;

        self.current_line_tokens = line.split_whitespace().map(|s| s.to_string()).collect();
        self.token_index = 0;
        Ok(())
    }
}

/// Find the position of the last byte in "end_header\n"
pub fn find_header_end(buffer: &[u8]) -> Option<usize> {
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

struct AsciiDirectElementDeserializer<'a, R> {
    parent: &'a mut AsciiElementDeserializer<R>,
}

struct BinaryDirectElementDeserializer<'a, R, E> {
    parent: &'a mut BinaryElementDeserializer<R, E>,
    _endian: PhantomData<E>,
}

impl<'de, 'a, R: BufRead> de::Deserializer<'de> for AsciiDirectElementDeserializer<'a, R> {
    type Error = PlyError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlyError::Serde(
            "deserialize_any not supported - struct fields must have specific types".to_string(),
        ))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let map_access = AsciiDirectMapAccess {
            parent: self.parent,
            current_property: 0,
        };
        visitor.visit_map(map_access)
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}

impl<'de, 'a, R: BufRead, E: ByteOrder> de::Deserializer<'de>
    for BinaryDirectElementDeserializer<'a, R, E>
{
    type Error = PlyError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlyError::Serde(
            "deserialize_any not supported - struct fields must have specific types".to_string(),
        ))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let map_access = BinaryDirectMapAccess {
            parent: self.parent,
            current_property: 0,
            _endian: PhantomData,
        };
        visitor.visit_map(map_access)
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}

struct AsciiDirectMapAccess<'a, R> {
    parent: &'a mut AsciiElementDeserializer<R>,
    current_property: usize,
}

struct BinaryDirectMapAccess<'a, R, E> {
    parent: &'a mut BinaryElementDeserializer<R, E>,
    current_property: usize,
    _endian: PhantomData<E>,
}

impl<'de, 'a, R: BufRead> MapAccess<'de> for AsciiDirectMapAccess<'a, R> {
    type Error = PlyError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.current_property >= self.parent.properties.len() {
            return Ok(None);
        }

        let property = &self.parent.properties[self.current_property];
        let field_name = match property {
            PropertyType::Scalar { name, .. } => name,
            PropertyType::List { name, .. } => name,
        };
        seed.deserialize(str_to_deserializer(field_name)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.current_property += 1;

        // Let Serde decide what it wants - it will call the appropriate deserializer method
        seed.deserialize(AsciiValueDeserializer {
            parent: self.parent,
        })
    }
}

impl<'de, 'a, R: BufRead, E: ByteOrder> MapAccess<'de> for BinaryDirectMapAccess<'a, R, E> {
    type Error = PlyError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.current_property >= self.parent.properties.len() {
            return Ok(None);
        }

        let property = &self.parent.properties[self.current_property];
        let field_name = match property {
            PropertyType::Scalar { name, .. } => name,
            PropertyType::List { name, .. } => name,
        };
        seed.deserialize(str_to_deserializer(field_name)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.current_property += 1;

        // Let Serde decide what it wants - it will call the appropriate deserializer method
        seed.deserialize(BinaryValueDeserializer::<_, E> {
            parent: self.parent,
            _endian: PhantomData,
        })
    }
}

struct AsciiValueDeserializer<'a, R> {
    parent: &'a mut AsciiElementDeserializer<R>,
}

struct BinaryValueDeserializer<'a, R, E> {
    parent: &'a mut BinaryElementDeserializer<R, E>,
    _endian: PhantomData<E>,
}

impl<'de, 'a, R: BufRead> de::Deserializer<'de> for AsciiValueDeserializer<'a, R> {
    type Error = PlyError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlyError::Serde(
            "deserialize_any not supported - struct fields must have specific types".to_string(),
        ))
    }

    fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<i8>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse i8: {e}")))?;
        visitor.visit_i8(value)
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<u8>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse u8: {e}")))?;
        visitor.visit_u8(value)
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<i16>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse i16: {e}")))?;
        visitor.visit_i16(value)
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<u16>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse u16: {e}")))?;
        visitor.visit_u16(value)
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<i32>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse i32: {e}")))?;
        visitor.visit_i32(value)
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<u32>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse u32: {e}")))?;
        visitor.visit_u32(value)
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<f32>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse f32: {e}")))?;
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<f64>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse f64: {e}")))?;
        visitor.visit_f64(value)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Read list count from current token
        let count_token = self.read_ascii_token()?;
        let count = count_token
            .parse::<usize>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse list count: {e}")))?;

        let seq_access = AsciiSeqAccess {
            parent: self.parent,
            remaining: count,
        };
        visitor.visit_seq(seq_access)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // For PLY, if a property exists in the header, it has a value
        // We don't have null/None values in PLY format, so always Some
        visitor.visit_some(self)
    }

    serde::forward_to_deserialize_any! {
        bool i128 i64 u128 u64 char str string bytes byte_buf unit
        unit_struct newtype_struct tuple tuple_struct map struct enum
        identifier ignored_any
    }
}

impl<'de, 'a, R: BufRead, E: ByteOrder> de::Deserializer<'de>
    for BinaryValueDeserializer<'a, R, E>
{
    type Error = PlyError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlyError::Serde(
            "deserialize_any not supported - struct fields must have specific types".to_string(),
        ))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.parent.reader.read_i8()?;
        visitor.visit_i8(value)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.parent.reader.read_u8()?;
        visitor.visit_u8(value)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.parent.reader.read_i16::<E>()?;
        visitor.visit_i16(value)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.parent.reader.read_u16::<E>()?;
        visitor.visit_u16(value)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.parent.reader.read_i32::<E>()?;
        visitor.visit_i32(value)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.parent.reader.read_u32::<E>()?;
        visitor.visit_u32(value)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.parent.reader.read_f32::<E>()?;
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.parent.reader.read_f64::<E>()?;
        visitor.visit_f64(value)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Read list count as u8 (standard PLY list count type)
        let count = self.parent.reader.read_u8()? as usize;

        let seq_access = BinarySeqAccess {
            parent: self.parent,
            remaining: count,
            _endian: PhantomData,
        };
        visitor.visit_seq(seq_access)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // For PLY, if a property exists in the header, it has a value
        // We don't have null/None values in PLY format, so always Some
        visitor.visit_some(self)
    }

    serde::forward_to_deserialize_any! {
        bool i128 i64 u128 u64 char str string bytes byte_buf unit
        unit_struct newtype_struct tuple tuple_struct map struct enum
        identifier ignored_any
    }
}

impl<'a, R: BufRead> AsciiValueDeserializer<'a, R> {
    fn read_ascii_token(&mut self) -> Result<&str, PlyError> {
        if self.parent.token_index >= self.parent.current_line_tokens.len() {
            return Err(PlyError::Serde(
                "Not enough tokens in line for element".to_string(),
            ));
        }

        let token = &self.parent.current_line_tokens[self.parent.token_index];
        self.parent.token_index += 1;
        Ok(token)
    }
}

/// ASCII sequence access for PLY lists
struct AsciiSeqAccess<'a, R> {
    parent: &'a mut AsciiElementDeserializer<R>,
    remaining: usize,
}

/// Binary sequence access for PLY lists
struct BinarySeqAccess<'a, R, E> {
    parent: &'a mut BinaryElementDeserializer<R, E>,
    remaining: usize,
    _endian: PhantomData<E>,
}

impl<'de, 'a, R: BufRead> de::SeqAccess<'de> for AsciiSeqAccess<'a, R> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;

        let deserializer = AsciiValueDeserializer {
            parent: self.parent,
        };
        seed.deserialize(deserializer).map(Some)
    }
}

impl<'de, 'a, R: BufRead, E: ByteOrder> de::SeqAccess<'de> for BinarySeqAccess<'a, R, E> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;

        let deserializer = BinaryValueDeserializer {
            parent: self.parent,
            _endian: PhantomData,
        };
        seed.deserialize(deserializer).map(Some)
    }
}

use serde::de::value::StrDeserializer;

// Use a simple wrapper function instead of conflicting trait impl
fn str_to_deserializer(s: &str) -> StrDeserializer<'_, PlyError> {
    StrDeserializer::new(s)
}

use serde::Deserialize;
use std::io::Cursor;

impl Default for ChunkedHeaderParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkedHeaderParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            header_complete: false,
            header: None,
            leftover_data: Vec::new(),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.header_complete
    }

    /// Parse header from a chunk of bytes, returning the header if complete
    pub fn parse_from_bytes(&mut self, chunk: &[u8]) -> Result<Option<PlyHeader>, PlyError> {
        if self.header_complete {
            return Ok(self.header.clone());
        }

        self.buffer.extend_from_slice(chunk);

        if let Some(end_pos) = find_header_end(&self.buffer) {
            let header_data = self.buffer[..=end_pos].to_vec();
            self.leftover_data = self.buffer[end_pos + 1..].to_vec();

            let cursor = std::io::Cursor::new(&header_data);
            let mut reader = std::io::BufReader::new(cursor);
            let header = PlyHeader::parse(&mut reader)?;

            self.header = Some(header.clone());
            self.header_complete = true;

            Ok(Some(header))
        } else {
            Ok(None)
        }
    }

    /// Create a multi-element file parser that inherits leftover data
    pub fn into_file_parser(self) -> Result<ChunkedFileParser, PlyError> {
        // Parse header from header bytes
        let cursor = std::io::Cursor::new(self.buffer);
        let mut reader = std::io::BufReader::new(cursor);
        let header = PlyHeader::parse(&mut reader)?;

        Ok(ChunkedFileParser {
            header,
            buffer: self.leftover_data,
            current_element_index: 0,
            elements_parsed_in_current: 0,
        })
    }
}

impl ChunkedFileParser {
    /// Parse elements of a specific type from buffered data, managing leftover data between element types
    pub fn parse_chunk<T>(&mut self, element_name: &str) -> Result<Option<Vec<T>>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        // Find the element definition and clone it to avoid borrowing issues
        let element_def = self
            .header
            .get_element(element_name)
            .ok_or_else(|| PlyError::MissingElement(element_name.to_string()))?
            .clone();

        // Check if we've already finished parsing this element type
        if self.elements_parsed_in_current >= element_def.count {
            return Ok(None);
        }

        match self.header.format {
            PlyFormat::Ascii => self.parse_ascii_chunk::<T>(&element_def),
            PlyFormat::BinaryLittleEndian => {
                self.parse_binary_chunk::<T, byteorder::LittleEndian>(&element_def)
            }
            PlyFormat::BinaryBigEndian => {
                self.parse_binary_chunk::<T, byteorder::BigEndian>(&element_def)
            }
        }
    }

    /// Add more data to the parser buffer
    pub fn add_data(&mut self, chunk: &[u8]) {
        self.buffer.extend_from_slice(chunk);
    }

    /// Check if all elements of a specific type have been parsed
    pub fn is_element_complete(&self, element_name: &str) -> bool {
        if let Some(element_def) = self.header.get_element(element_name) {
            self.elements_parsed_in_current >= element_def.count
        } else {
            false
        }
    }

    /// Move to the next element type (call when current element type is complete)
    pub fn advance_to_next_element(&mut self) {
        self.current_element_index += 1;
        self.elements_parsed_in_current = 0;
    }

    fn parse_ascii_chunk<T>(&mut self, element_def: &ElementDef) -> Result<Option<Vec<T>>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let mut elements = Vec::new();

        // Process complete lines from buffer
        while let Some(newline_pos) = self.buffer.iter().position(|&b| b == b'\n') {
            if self.elements_parsed_in_current >= element_def.count {
                break;
            }

            let line_bytes = self.buffer.drain(..=newline_pos).collect::<Vec<u8>>();
            let line = String::from_utf8_lossy(&line_bytes[..line_bytes.len() - 1]); // Remove newline

            if !line.trim().is_empty() {
                let element = self.parse_ascii_line::<T>(&line, &element_def.properties)?;
                elements.push(element);
                self.elements_parsed_in_current += 1;
            }
        }

        if elements.is_empty() {
            Ok(None) // No complete lines available yet
        } else {
            Ok(Some(elements))
        }
    }

    fn parse_binary_chunk<T, E: ByteOrder>(
        &mut self,
        element_def: &ElementDef,
    ) -> Result<Option<Vec<T>>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let element_size = calculate_binary_element_size(&element_def.properties)?;
        let remaining_elements = element_def.count - self.elements_parsed_in_current;
        let available_elements = self.buffer.len() / element_size;
        let elements_to_parse = remaining_elements.min(available_elements);

        if elements_to_parse == 0 {
            return Ok(None); // Not enough data for complete elements
        }

        let bytes_to_consume = elements_to_parse * element_size;
        let element_bytes = self.buffer.drain(..bytes_to_consume).collect::<Vec<u8>>();

        let mut elements = Vec::new();
        let cursor = std::io::Cursor::new(element_bytes);
        let mut deserializer = BinaryElementDeserializer::<_, E>::new(
            cursor,
            elements_to_parse,
            element_def.properties.clone(),
        );

        while let Some(element) = deserializer.next_element::<T>()? {
            elements.push(element);
            self.elements_parsed_in_current += 1;
        }

        Ok(Some(elements))
    }

    fn parse_ascii_line<T>(&self, line: &str, properties: &[PropertyType]) -> Result<T, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let tokens: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

        let mut deserializer = AsciiElementDeserializer {
            reader: std::io::Cursor::new(""),
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
