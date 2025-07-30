//! PLY deserializer that feeds binary data directly to serde visitors
//!
//! This module provides high-performance deserialization by pre-computing
//! a reading plan to eliminate runtime dispatch and property lookups.

use crate::{PlyError, PlyFormat, PlyHeader, PropertyType, ScalarType};
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use serde::de::{self, DeserializeSeed, MapAccess, Visitor};
use serde::Deserialize;
use std::io::{BufRead, BufReader, Read};

/// Pre-computed plan for reading all fields of an element
#[derive(Debug)]
struct ReadPlan {
    field_names: Vec<String>,
    list_count_types: Vec<Option<ScalarType>>, // None for scalars, Some(type) for lists
}

/// Element deserializer that reads directly into structs
pub struct ElementDeserializer<R> {
    reader: BufReader<R>,
    format: PlyFormat,
    elements_read: usize,
    element_count: usize,
    current_line_tokens: Vec<String>,
    token_index: usize,
    read_plan: ReadPlan,
}

impl<R: Read> ElementDeserializer<R> {
    pub fn new(reader: R, header: &PlyHeader, element_name: &str) -> Result<Self, PlyError> {
        let element_def = header
            .get_element(element_name)
            .ok_or_else(|| PlyError::MissingElement(element_name.to_string()))?;

        // Pre-compute reading plan and validate struct compatibility
        let mut field_names = Vec::new();
        let mut list_count_types = Vec::new();

        for property in &element_def.properties {
            field_names.push(match property {
                PropertyType::Scalar { name, .. } => name.clone(),
                PropertyType::List { name, .. } => name.clone(),
            });

            list_count_types.push(match property {
                PropertyType::Scalar { .. } => None,
                PropertyType::List { count_type, .. } => Some(count_type.clone()),
            });
        }

        let read_plan = ReadPlan {
            field_names,
            list_count_types,
        };

        Ok(Self {
            reader: BufReader::new(reader),
            format: header.format.clone(),
            elements_read: 0,
            element_count: element_def.count,
            current_line_tokens: Vec::new(),
            token_index: 0,
            read_plan,
        })
    }

    /// Deserialize the next element directly into a struct
    pub fn next_element<T>(&mut self) -> Result<Option<T>, PlyError>
    where
        T: for<'de> Deserialize<'de>,
    {
        if self.elements_read >= self.element_count {
            return Ok(None);
        }

        // For ASCII format, read entire line and tokenize for each element
        if self.format == PlyFormat::Ascii {
            let mut line = String::new();
            self.reader.read_line(&mut line)?;
            self.current_line_tokens = line.split_whitespace().map(|s| s.to_string()).collect();
            self.token_index = 0;
        }

        let deserializer = DirectElementDeserializer { parent: self };

        let element = T::deserialize(deserializer)?;
        self.elements_read += 1;
        Ok(Some(element))
    }
}

/// Direct element deserializer that feeds PLY data straight to serde visitors
struct DirectElementDeserializer<'a, R> {
    parent: &'a mut ElementDeserializer<R>,
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for DirectElementDeserializer<'a, R> {
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
        visitor.visit_map(DirectMapAccess {
            parent: self.parent,
            current_property: 0,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}

/// Map access that reads PLY properties directly
struct DirectMapAccess<'a, R> {
    parent: &'a mut ElementDeserializer<R>,
    current_property: usize,
}

impl<'de, 'a, R: Read> MapAccess<'de> for DirectMapAccess<'a, R> {
    type Error = PlyError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.current_property >= self.parent.read_plan.field_names.len() {
            return Ok(None);
        }

        let field_name = &self.parent.read_plan.field_names[self.current_property];
        seed.deserialize(str_to_deserializer(field_name.as_str()))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let list_count_type = self.parent.read_plan.list_count_types[self.current_property].clone();
        self.current_property += 1;

        match list_count_type {
            None => {
                // Scalar property - create format-specific deserializer
                match self.parent.format {
                    PlyFormat::Ascii => {
                        let deserializer = AsciiScalarDeserializer {
                            parent: self.parent,
                        };
                        seed.deserialize(deserializer)
                    }
                    PlyFormat::BinaryLittleEndian => {
                        let deserializer = BinaryScalarDeserializer::<_, LittleEndian> {
                            parent: self.parent,
                            _endian: std::marker::PhantomData,
                        };
                        seed.deserialize(deserializer)
                    }
                    PlyFormat::BinaryBigEndian => {
                        let deserializer = BinaryScalarDeserializer::<_, BigEndian> {
                            parent: self.parent,
                            _endian: std::marker::PhantomData,
                        };
                        seed.deserialize(deserializer)
                    }
                }
            }
            Some(count_type) => {
                // List property - create format-specific deserializer
                match self.parent.format {
                    PlyFormat::Ascii => {
                        let deserializer = AsciiListDeserializer {
                            parent: self.parent,
                            count_type,
                        };
                        seed.deserialize(deserializer)
                    }
                    PlyFormat::BinaryLittleEndian => {
                        let deserializer = BinaryListDeserializer::<_, LittleEndian> {
                            parent: self.parent,
                            count_type,
                            _endian: std::marker::PhantomData,
                        };
                        seed.deserialize(deserializer)
                    }
                    PlyFormat::BinaryBigEndian => {
                        let deserializer = BinaryListDeserializer::<_, BigEndian> {
                            parent: self.parent,
                            count_type,
                            _endian: std::marker::PhantomData,
                        };
                        seed.deserialize(deserializer)
                    }
                }
            }
        }
    }
}

/// ASCII scalar deserializer that trusts serde's type requests
struct AsciiScalarDeserializer<'a, R> {
    parent: &'a mut ElementDeserializer<R>,
}

/// Binary scalar deserializer that trusts serde's type requests
struct BinaryScalarDeserializer<'a, R, E> {
    parent: &'a mut ElementDeserializer<R>,
    _endian: std::marker::PhantomData<E>,
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for AsciiScalarDeserializer<'a, R> {
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
            .map_err(|e| PlyError::Serde(format!("Failed to parse i8: {}", e)))?;
        visitor.visit_i8(value)
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<u8>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse u8: {}", e)))?;
        visitor.visit_u8(value)
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<i16>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse i16: {}", e)))?;
        visitor.visit_i16(value)
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<u16>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse u16: {}", e)))?;
        visitor.visit_u16(value)
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<i32>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse i32: {}", e)))?;
        visitor.visit_i32(value)
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<u32>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse u32: {}", e)))?;
        visitor.visit_u32(value)
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token
            .parse::<f32>()
            .map_err(|e| PlyError::Serde(format!("Failed to parse f32: {}", e)))?;
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.parent.format {
            PlyFormat::Ascii => {
                let token = self.read_ascii_token()?;
                let value = token
                    .parse::<f64>()
                    .map_err(|e| PlyError::Serde(format!("Failed to parse f64: {}", e)))?;
                visitor.visit_f64(value)
            }
            PlyFormat::BinaryLittleEndian => {
                let mut buf = [0u8; 8];
                self.parent.reader.read_exact(&mut buf)?;
                visitor.visit_f64(f64::from_le_bytes(buf))
            }
            PlyFormat::BinaryBigEndian => {
                let mut buf = [0u8; 8];
                self.parent.reader.read_exact(&mut buf)?;
                visitor.visit_f64(f64::from_be_bytes(buf))
            }
        }
    }

    serde::forward_to_deserialize_any! {
        bool i128 i64 u128 u64 char str string bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct map struct enum
        identifier ignored_any
    }
}

impl<'de, 'a, R: Read, E: ByteOrder> de::Deserializer<'de> for BinaryScalarDeserializer<'a, R, E> {
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

    serde::forward_to_deserialize_any! {
        bool i128 i64 u128 u64 char str string bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct map struct enum
        identifier ignored_any
    }
}

impl<'a, R: Read> AsciiScalarDeserializer<'a, R> {
    fn read_ascii_token(&mut self) -> Result<&str, PlyError> {
        // Get next token from pre-parsed line
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

/// ASCII list deserializer
struct AsciiListDeserializer<'a, R> {
    parent: &'a mut ElementDeserializer<R>,
    count_type: ScalarType,
}

/// Binary list deserializer
struct BinaryListDeserializer<'a, R, E> {
    parent: &'a mut ElementDeserializer<R>,
    count_type: ScalarType,
    _endian: std::marker::PhantomData<E>,
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for AsciiListDeserializer<'a, R> {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Read count from ASCII tokens
        if self.parent.token_index >= self.parent.current_line_tokens.len() {
            return Err(PlyError::Serde("No count token available".to_string()));
        }
        let count_str = &self.parent.current_line_tokens[self.parent.token_index];
        self.parent.token_index += 1;
        let count = count_str
            .parse::<usize>()
            .map_err(|_| PlyError::TypeMismatch {
                expected: "count".to_string(),
                found: count_str.clone(),
            })?;

        visitor.visit_seq(AsciiSeqAccess {
            parent: self.parent,
            remaining: count,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, 'a, R: Read, E: ByteOrder> de::Deserializer<'de> for BinaryListDeserializer<'a, R, E> {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Read count using byteorder
        let count = match &self.count_type {
            ScalarType::UChar => self.parent.reader.read_u8()? as usize,
            ScalarType::UShort => self.parent.reader.read_u16::<E>()? as usize,
            ScalarType::UInt => self.parent.reader.read_u32::<E>()? as usize,
            _ => {
                return Err(PlyError::TypeMismatch {
                    expected: "unsigned integer".to_string(),
                    found: format!("{:?}", &self.count_type),
                })
            }
        };

        visitor.visit_seq(BinarySeqAccess::<_, E> {
            parent: self.parent,
            remaining: count,
            _endian: std::marker::PhantomData,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

/// ASCII sequence access for PLY lists
struct AsciiSeqAccess<'a, R> {
    parent: &'a mut ElementDeserializer<R>,
    remaining: usize,
}

/// Binary sequence access for PLY lists
struct BinarySeqAccess<'a, R, E> {
    parent: &'a mut ElementDeserializer<R>,
    remaining: usize,
    _endian: std::marker::PhantomData<E>,
}

impl<'de, 'a, R: Read> de::SeqAccess<'de> for AsciiSeqAccess<'a, R> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        let deserializer = AsciiScalarDeserializer {
            parent: self.parent,
        };
        seed.deserialize(deserializer).map(Some)
    }
}

impl<'de, 'a, R: Read, E: byteorder::ByteOrder> de::SeqAccess<'de> for BinarySeqAccess<'a, R, E> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        let deserializer = BinaryScalarDeserializer::<_, E> {
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
