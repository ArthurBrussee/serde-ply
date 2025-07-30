//! PLY deserializer with type-level format specialization

use crate::{ElementDef, PlyError, PlyFormat, PlyHeader, PropertyType};
use byteorder::{ByteOrder, ReadBytesExt};
use serde::de::{self, DeserializeSeed, MapAccess, Visitor};

use std::io::BufRead;

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
    _endian: std::marker::PhantomData<E>,
}

/// Chunked header parser for async-compatible header parsing
pub struct ChunkedHeaderParser {
    buffer: Vec<u8>,
    header_complete: bool,
    header: Option<PlyHeader>,
    leftover_data: Vec<u8>,
}

/// Chunked element parser for async-compatible parsing from raw byte chunks
pub struct ChunkedElementParser<T> {
    format: PlyFormat,
    element_def: ElementDef,
    elements_parsed: usize,
    buffer: Vec<u8>,
    element_size: Option<usize>, // For binary formats
    _phantom: std::marker::PhantomData<T>,
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
        let deserializer = AsciiDirectElementDeserializer { parent: self };
        let element = T::deserialize(deserializer)?;
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
            _endian: std::marker::PhantomData,
        }
    }

    fn next_element<T>(&mut self) -> Result<Option<T>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        if self.elements_read >= self.element_count {
            return Ok(None);
        }

        let deserializer = BinaryDirectElementDeserializer {
            parent: self,
            _endian: std::marker::PhantomData,
        };
        let element = T::deserialize(deserializer)?;
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

impl ChunkedHeaderParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            header_complete: false,
            header: None,
            leftover_data: Vec::new(),
        }
    }

    /// Parse header from a raw byte chunk, returning header when complete
    pub fn parse_from_bytes(&mut self, chunk: &[u8]) -> Result<Option<&PlyHeader>, PlyError> {
        if self.header_complete {
            return Ok(self.header.as_ref());
        }

        // Append new chunk to buffer
        self.buffer.extend_from_slice(chunk);

        // Look for "end_header\n"
        if let Some(end_pos) = find_header_end(&self.buffer) {
            self.header_complete = true;

            // Split buffer at header end
            let header_bytes = self.buffer[..=end_pos].to_vec();
            self.leftover_data = self.buffer[end_pos + 1..].to_vec();

            // Parse header from header bytes
            let cursor = std::io::Cursor::new(header_bytes);
            let mut reader = std::io::BufReader::new(cursor);
            let header = PlyHeader::parse(&mut reader)?;

            self.header = Some(header);
            Ok(self.header.as_ref())
        } else {
            Ok(None) // Need more data
        }
    }

    /// Create an element parser that inherits any leftover data from header parsing
    pub fn element_parser<T>(
        &mut self,
        element_name: &str,
    ) -> Result<ChunkedElementParser<T>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let header = self
            .header
            .as_ref()
            .ok_or_else(|| PlyError::Serde("Header not parsed yet".to_string()))?;

        let element_def = header
            .get_element(element_name)
            .ok_or_else(|| PlyError::MissingElement(element_name.to_string()))?;

        let mut parser = ChunkedElementParser::new(header.format.clone(), element_def.clone())?;

        // Transfer leftover data to element parser
        if !self.leftover_data.is_empty() {
            parser.buffer = std::mem::take(&mut self.leftover_data);
        }

        Ok(parser)
    }

    /// Check if header parsing is complete
    pub fn is_complete(&self) -> bool {
        self.header_complete
    }

    /// Get the parsed header if available
    pub fn header(&self) -> Option<&PlyHeader> {
        self.header.as_ref()
    }
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

impl<T> ChunkedElementParser<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    pub fn new(format: PlyFormat, element_def: ElementDef) -> Result<Self, PlyError> {
        let element_size = match format {
            PlyFormat::Ascii => None,
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                Some(calculate_binary_element_size(&element_def.properties)?)
            }
        };

        Ok(Self {
            format,
            element_def,
            elements_parsed: 0,
            buffer: Vec::new(),
            element_size,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Parse elements from a raw byte chunk, returning parsed elements if any complete ones are available
    pub fn parse_from_bytes(&mut self, chunk: &[u8]) -> Result<Option<Vec<T>>, PlyError> {
        if self.elements_parsed >= self.element_def.count {
            return Ok(None);
        }

        // Append new chunk to buffer
        self.buffer.extend_from_slice(chunk);

        match self.format {
            PlyFormat::Ascii => self.parse_ascii_from_buffer(),
            PlyFormat::BinaryLittleEndian => {
                self.parse_binary_from_buffer::<byteorder::LittleEndian>()
            }
            PlyFormat::BinaryBigEndian => self.parse_binary_from_buffer::<byteorder::BigEndian>(),
        }
    }

    fn parse_ascii_from_buffer(&mut self) -> Result<Option<Vec<T>>, PlyError> {
        let mut elements = Vec::new();

        // Process complete lines from buffer
        while let Some(newline_pos) = self.buffer.iter().position(|&b| b == b'\n') {
            if self.elements_parsed >= self.element_def.count {
                break;
            }

            let line_bytes = self.buffer.drain(..=newline_pos).collect::<Vec<u8>>();
            let line = String::from_utf8_lossy(&line_bytes[..line_bytes.len() - 1]); // Remove newline

            if !line.trim().is_empty() {
                let element = self.parse_ascii_line(&line)?;
                elements.push(element);
                self.elements_parsed += 1;
            }
        }

        if elements.is_empty() {
            Ok(None) // No complete lines available yet
        } else {
            Ok(Some(elements))
        }
    }

    fn parse_binary_from_buffer<E: ByteOrder>(&mut self) -> Result<Option<Vec<T>>, PlyError> {
        let element_size = self.element_size.unwrap();
        let remaining_elements = self.element_def.count - self.elements_parsed;
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
            self.element_def.properties.clone(),
        );

        while let Some(element) = deserializer.next_element::<T>()? {
            elements.push(element);
            self.elements_parsed += 1;
        }

        Ok(Some(elements))
    }

    fn parse_ascii_line(&self, line: &str) -> Result<T, PlyError> {
        let tokens: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

        let mut deserializer = AsciiElementDeserializer {
            reader: std::io::Cursor::new(""),
            elements_read: 0,
            element_count: 1,
            current_line_tokens: tokens,
            token_index: 0,
            properties: self.element_def.properties.clone(),
        };

        let direct_deserializer = AsciiDirectElementDeserializer {
            parent: &mut deserializer,
        };

        T::deserialize(direct_deserializer)
    }

    /// Get the total number of elements parsed so far
    pub fn elements_parsed(&self) -> usize {
        self.elements_parsed
    }

    /// Get the total number of elements to parse
    pub fn total_elements(&self) -> usize {
        self.element_def.count
    }

    /// Check if parsing is complete
    pub fn is_complete(&self) -> bool {
        self.elements_parsed >= self.element_def.count
    }
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
    _endian: std::marker::PhantomData<E>,
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
            _endian: std::marker::PhantomData,
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
    _endian: std::marker::PhantomData<E>,
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
        let deserializer = AsciiValueDeserializer {
            parent: self.parent,
        };
        seed.deserialize(deserializer)
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
        let deserializer = BinaryValueDeserializer::<_, E> {
            parent: self.parent,
            _endian: std::marker::PhantomData,
        };
        seed.deserialize(deserializer)
    }
}

struct AsciiValueDeserializer<'a, R> {
    parent: &'a mut AsciiElementDeserializer<R>,
}

struct BinaryValueDeserializer<'a, R, E> {
    parent: &'a mut BinaryElementDeserializer<R, E>,
    _endian: std::marker::PhantomData<E>,
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

    serde::forward_to_deserialize_any! {
        bool i128 i64 u128 u64 char str string bytes byte_buf option unit
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
            _endian: std::marker::PhantomData,
        };
        visitor.visit_seq(seq_access)
    }

    serde::forward_to_deserialize_any! {
        bool i128 i64 u128 u64 char str string bytes byte_buf option unit
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
    _endian: std::marker::PhantomData<E>,
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

        let deserializer = BinaryValueDeserializer::<_, E> {
            parent: self.parent,
            _endian: std::marker::PhantomData,
        };
        seed.deserialize(deserializer).map(Some)
    }
}

use serde::de::value::StrDeserializer;

// Use a simple wrapper function instead of conflicting trait impl
fn str_to_deserializer(s: &str) -> StrDeserializer<'_, PlyError> {
    StrDeserializer::new(s)
}
