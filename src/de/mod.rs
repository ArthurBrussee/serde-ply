mod ply_file;

use crate::{ElementDef, PlyError, PropertyType, ScalarType};
use byteorder::{ByteOrder, ReadBytesExt};
use serde::de::{self, DeserializeSeed, Deserializer, IntoDeserializer, MapAccess, Visitor};

use serde::de::value::BytesDeserializer;
use std::io::Read;
use std::marker::PhantomData;

pub(crate) struct AsciiRowDeserializer<R> {
    pub reader: R,
    pub elem_def: ElementDef,
}

impl<R: Read> AsciiRowDeserializer<R> {
    pub fn new(reader: R, elem_def: ElementDef) -> Self {
        Self { reader, elem_def }
    }
}

impl<'de, R: Read> Deserializer<'de> for &mut AsciiRowDeserializer<R> {
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
        visitor.visit_map(AsciiRowMapAccess {
            parent: self,
            current_property: 0,
        })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(AsciiRowMapAccess {
            parent: self,
            current_property: 0,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier ignored_any
    }
}

struct AsciiRowMapAccess<'a, R> {
    parent: &'a mut AsciiRowDeserializer<R>,
    current_property: usize,
}

impl<'de, 'a, R: Read> MapAccess<'de> for AsciiRowMapAccess<'a, R> {
    type Error = PlyError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let Some(prop) = self.parent.elem_def.properties.get(self.current_property) else {
            return Ok(None);
        };
        let result = seed
            .deserialize(BytesDeserializer::new(prop.name.as_bytes()))
            .map(Some);
        result
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let property_index = self.current_property;
        self.current_property += 1;
        seed.deserialize(AsciiValueDeserializer {
            parent: self.parent,
            property_index,
        })
    }
}

/// ASCII sequence access for PLY lists
struct AsciiListAccess<'a, R> {
    parent: &'a mut AsciiRowDeserializer<R>,
    remaining: usize,
    property_index: usize,
}

struct AsciiValueDeserializer<'a, R> {
    parent: &'a mut AsciiRowDeserializer<R>,
    property_index: usize,
}

impl<'de, 'a, R: Read> Deserializer<'de> for AsciiValueDeserializer<'a, R> {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let Some(property) = self.parent.elem_def.properties.get(self.property_index) else {
            return Err(PlyError::Serde("Property index out of bounds".to_string()));
        };
        match &property.property_type {
            PropertyType::Scalar { data_type, .. } => {
                use crate::ScalarType;
                match data_type {
                    ScalarType::I8 => self.deserialize_i8(visitor),
                    ScalarType::U8 => self.deserialize_u8(visitor),
                    ScalarType::I16 => self.deserialize_i16(visitor),
                    ScalarType::U16 => self.deserialize_u16(visitor),
                    ScalarType::I32 => self.deserialize_i32(visitor),
                    ScalarType::U32 => self.deserialize_u32(visitor),
                    ScalarType::F32 => self.deserialize_f32(visitor),
                    ScalarType::F64 => self.deserialize_f64(visitor),
                }
            }
            PropertyType::List { .. } => self.deserialize_seq(visitor),
        }
    }

    fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let token = self.read_ascii_token()?;
        let value = token.parse::<i8>()?;
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

        let seq_access = AsciiListAccess {
            parent: self.parent,
            remaining: count,
            property_index: self.property_index,
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

impl<'a, R: Read> AsciiValueDeserializer<'a, R> {
    fn read_ascii_token(&mut self) -> Result<String, PlyError> {
        let mut token = String::new();
        let mut in_token = false;

        loop {
            let mut byte = [0u8; 1];
            match self.parent.reader.read(&mut byte) {
                Ok(0) => break,
                Ok(_) => {
                    let ch = byte[0] as char;
                    if ch.is_ascii_whitespace() {
                        if in_token || ch == '\n' {
                            break;
                        }
                    } else {
                        in_token = true;
                        token.push(ch);
                    }
                }
                Err(e) => return Err(PlyError::Io(e)),
            }
        }

        if !in_token {
            return Err(PlyError::Serde("No token found".to_string()));
        }

        Ok(token)
    }
}

// Binary deserialization functions.
// Largely similair to ASCII but to keep performance we want to these to be specialized, rather than each
// row having to decide on the ply format.
//
pub(crate) struct BinaryRowDeserializer<R, E> {
    pub reader: R,
    pub elem_def: ElementDef,
    pub _endian: PhantomData<E>,
}

impl<R: Read, E: ByteOrder> BinaryRowDeserializer<R, E> {
    pub fn new(reader: R, elem_def: ElementDef) -> Self {
        Self {
            reader,
            elem_def,
            _endian: PhantomData,
        }
    }
}

impl<'de, R: Read, E: ByteOrder> Deserializer<'de> for &mut BinaryRowDeserializer<R, E> {
    type Error = PlyError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlyError::Serde(
            "Ply row must be a struct or map.".to_string(),
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
        visitor.visit_map(BinaryRowMap {
            parent: self,
            current_property: 0,
            _endian: PhantomData,
        })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(BinaryRowMap {
            parent: self,
            current_property: 0,
            _endian: PhantomData,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier ignored_any
    }
}

struct BinaryRowMap<'a, R, E> {
    parent: &'a mut BinaryRowDeserializer<R, E>,
    current_property: usize,
    _endian: PhantomData<E>,
}

impl<'de, 'a, R: Read, E: ByteOrder> MapAccess<'de> for BinaryRowMap<'a, R, E> {
    type Error = PlyError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let Some(prop) = &self.parent.elem_def.properties.get(self.current_property) else {
            return Ok(None);
        };
        seed.deserialize(BytesDeserializer::<PlyError>::new(prop.name.as_bytes()))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        // SAFETY: Bounds check already has happened in next_key_seed.
        let prop = unsafe {
            &self
                .parent
                .elem_def
                .properties
                .get_unchecked(self.current_property)
                .property_type
        };
        self.current_property += 1;

        let reader = &mut self.parent.reader;
        // Deserialize next value here straight away, no real need to wrap this in something else first.
        match prop {
            PropertyType::Scalar { data_type } => match data_type {
                ScalarType::I8 => seed.deserialize(reader.read_i8()?.into_deserializer()),
                ScalarType::U8 => seed.deserialize(reader.read_u8()?.into_deserializer()),
                ScalarType::I16 => seed.deserialize(reader.read_i16::<E>()?.into_deserializer()),
                ScalarType::U16 => seed.deserialize(reader.read_u16::<E>()?.into_deserializer()),
                ScalarType::I32 => seed.deserialize(reader.read_i32::<E>()?.into_deserializer()),
                ScalarType::U32 => seed.deserialize(reader.read_u32::<E>()?.into_deserializer()),
                ScalarType::F32 => seed.deserialize(reader.read_f32::<E>()?.into_deserializer()),
                ScalarType::F64 => seed.deserialize(reader.read_f64::<E>()?.into_deserializer()),
            },
            // TODO:
            PropertyType::List {
                count_type,
                data_type,
            } => seed.deserialize(BinaryListDeserializer::<_, E> {
                count_type: *count_type,
                data_type: *data_type,
                reader: &mut self.parent.reader,
                _endian: PhantomData,
            }),
        }
    }
}

struct BinaryListDeserializer<'a, R, E> {
    reader: &'a mut R,
    count_type: ScalarType,
    data_type: ScalarType,
    _endian: PhantomData<E>,
}

impl<'a, 'de, R: Read, E: ByteOrder> Deserializer<'de> for BinaryListDeserializer<'a, R, E> {
    type Error = PlyError;

    // Have to use deserialize any as we can't parse a number of something like that, so we have to check the header
    // for what value to produce.
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
        let count = match self.count_type {
            ScalarType::I8 => self.reader.read_i8()? as usize,
            ScalarType::U8 => self.reader.read_u8()? as usize,
            ScalarType::I16 => self.reader.read_i16::<E>()? as usize,
            ScalarType::U16 => self.reader.read_u16::<E>()? as usize,
            ScalarType::I32 => self.reader.read_i32::<E>()? as usize,
            ScalarType::U32 => self.reader.read_u32::<E>()? as usize,
            ScalarType::F32 => self.reader.read_f32::<E>()? as usize,
            ScalarType::F64 => self.reader.read_f64::<E>()? as usize,
        };

        visitor.visit_seq(BinarySeqAccess {
            reader: self.reader,
            remaining: count,
            data_type: self.data_type,
            _endian: PhantomData::<E>,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 f32 f64 i128 i64 u128 u64 char str string bytes byte_buf unit
        option identifier unit_struct newtype_struct tuple tuple_struct map struct enum ignored_any
    }
}

/// Binary sequence access for PLY lists
struct BinarySeqAccess<'a, R, E> {
    reader: &'a mut R,
    data_type: ScalarType,
    remaining: usize,
    _endian: PhantomData<E>,
}

impl<'de, 'a, R: Read> de::SeqAccess<'de> for AsciiListAccess<'a, R> {
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
            property_index: self.property_index,
        };
        seed.deserialize(deserializer).map(Some)
    }
}

impl<'a, 'de, R: Read, E: ByteOrder> de::SeqAccess<'de> for BinarySeqAccess<'a, R, E> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;

        let res = match self.data_type {
            ScalarType::I8 => seed.deserialize(self.reader.read_i8()?.into_deserializer()),
            ScalarType::U8 => seed.deserialize(self.reader.read_u8()?.into_deserializer()),
            ScalarType::I16 => seed.deserialize(self.reader.read_i16::<E>()?.into_deserializer()),
            ScalarType::U16 => seed.deserialize(self.reader.read_u16::<E>()?.into_deserializer()),
            ScalarType::I32 => seed.deserialize(self.reader.read_i32::<E>()?.into_deserializer()),
            ScalarType::U32 => seed.deserialize(self.reader.read_u32::<E>()?.into_deserializer()),
            ScalarType::F32 => seed.deserialize(self.reader.read_f32::<E>()?.into_deserializer()),
            ScalarType::F64 => seed.deserialize(self.reader.read_f64::<E>()?.into_deserializer()),
        };

        res.map(Some)
    }
}
