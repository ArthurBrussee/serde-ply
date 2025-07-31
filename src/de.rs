use crate::{PlyError, PropertyType};
use byteorder::{ByteOrder, ReadBytesExt};
use serde::de::{self, DeserializeSeed, MapAccess, Visitor};

use serde::de::value::StrDeserializer;
use std::{io::BufRead, marker::PhantomData};

pub trait FormatDeserializer<R> {
    fn new(reader: R, element_count: usize, properties: Vec<PropertyType>) -> Self;
    fn next_element<T>(&mut self) -> Result<Option<T>, PlyError>
    where
        T: for<'de> serde::Deserialize<'de>;
}

pub(crate) struct AsciiElementDeserializer<R> {
    pub reader: R,
    pub elements_read: usize,
    pub element_count: usize,
    // TODO: This current line tokens stuff is awful. We should just use the reader for actual parsing.
    // That is, advance until whitespace get number etc. Ply files don't need anything complicated.
    // When hitting a newline we should make sure we've read all the ply properties.
    pub current_line_tokens: Vec<String>,
    pub token_index: usize,
    pub properties: Vec<PropertyType>,
}

pub(crate) struct BinaryElementDeserializer<R, E> {
    pub reader: R,
    pub elements_read: usize,
    pub element_count: usize,
    pub properties: Vec<PropertyType>,
    pub _endian: PhantomData<E>,
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

        self.elements_read += 1;
        self.read_ascii_line()?;
        let element = T::deserialize(AsciiDirectElementDeserializer { parent: self })?;
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

        self.elements_read += 1;
        let element = T::deserialize(BinaryDirectElementDeserializer {
            parent: self,
            _endian: PhantomData,
        })?;
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

pub(crate) struct AsciiDirectElementDeserializer<'a, R> {
    pub parent: &'a mut AsciiElementDeserializer<R>,
}

pub(crate) struct BinaryDirectElementDeserializer<'a, R, E> {
    pub parent: &'a mut BinaryElementDeserializer<R, E>,
    pub _endian: PhantomData<E>,
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
        seed.deserialize(StrDeserializer::new(field_name)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.current_property += 1;
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
        seed.deserialize(StrDeserializer::new(field_name)).map(Some)
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
