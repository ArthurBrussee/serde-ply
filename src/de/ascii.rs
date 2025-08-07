use std::io::Read;

use serde::{
    de::{value::BytesDeserializer, DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserializer,
};

use crate::{ElementDef, PlyError, PropertyType, ScalarType};

pub(crate) struct AsciiRowDeserializer<'e, R> {
    pub reader: R,
    pub elem_def: &'e ElementDef,
}

impl<'e, R: Read> AsciiRowDeserializer<'e, R> {
    pub fn new(reader: R, elem_def: &'e ElementDef) -> Self {
        Self { reader, elem_def }
    }
}

impl<'de, 'e: 'de, R: Read> Deserializer<'de> for &mut AsciiRowDeserializer<'e, R> {
    type Error = PlyError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlyError::RowMustBeStructOrMap)
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
        visitor.visit_map(AsciiRowMap {
            parent: self,
            current_property: 0,
        })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(AsciiRowMap {
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

struct AsciiRowMap<'a, 'e, R> {
    parent: &'a mut AsciiRowDeserializer<'e, R>,
    current_property: usize,
}

impl<'de, 'a, 'e, R: Read> MapAccess<'de> for AsciiRowMap<'a, 'e, R> {
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

        seed.deserialize(AsciiValueDeserializer {
            parent: self.parent,
            prop,
        })
    }
}

struct AsciiValueDeserializer<'a, 'e, R> {
    parent: &'a mut AsciiRowDeserializer<'e, R>,
    prop: &'a PropertyType,
}

impl<'de, 'a, 'e, R: Read> Deserializer<'de> for AsciiValueDeserializer<'a, 'e, R> {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.prop {
            PropertyType::Scalar { data_type } => {
                let token = read_ascii_token(&mut self.parent.reader)?;
                match data_type {
                    ScalarType::I8 => visitor.visit_i8(token.parse::<i8>()?),
                    ScalarType::U8 => visitor.visit_u8(token.parse::<u8>()?),
                    ScalarType::I16 => visitor.visit_i16(token.parse::<i16>()?),
                    ScalarType::U16 => visitor.visit_u16(token.parse::<u16>()?),
                    ScalarType::I32 => visitor.visit_i32(token.parse::<i32>()?),
                    ScalarType::U32 => visitor.visit_u32(token.parse::<u32>()?),
                    ScalarType::F32 => visitor.visit_f32(token.parse::<f32>()?),
                    ScalarType::F64 => visitor.visit_f64(token.parse::<f64>()?),
                }
            }
            PropertyType::List { .. } => self.deserialize_seq(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // PLY properties are always present if defined in header
        visitor.visit_some(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let PropertyType::List {
            count_type,
            data_type,
        } = self.prop
        else {
            return Err(PlyError::ExpectedListProperty);
        };

        let count_token = read_ascii_token(&mut self.parent.reader)?;
        let count = match count_type {
            ScalarType::I8 => count_token.parse::<i8>()? as usize,
            ScalarType::U8 => count_token.parse::<u8>()? as usize,
            ScalarType::I16 => count_token.parse::<i16>()? as usize,
            ScalarType::U16 => count_token.parse::<u16>()? as usize,
            ScalarType::I32 => count_token.parse::<i32>()? as usize,
            ScalarType::U32 => count_token.parse::<u32>()? as usize,
            ScalarType::F32 => count_token.parse::<f32>()? as usize,
            ScalarType::F64 => count_token.parse::<f64>()? as usize,
        };

        visitor.visit_seq(AsciiSeqAccess {
            reader: &mut self.parent.reader,
            remaining: count,
            data_type: *data_type,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct AsciiScalarDeserializer {
    token: String,
    data_type: ScalarType,
}

impl<'de> Deserializer<'de> for AsciiScalarDeserializer {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.data_type {
            ScalarType::I8 => visitor.visit_i8(self.token.parse::<i8>()?),
            ScalarType::U8 => visitor.visit_u8(self.token.parse::<u8>()?),
            ScalarType::I16 => visitor.visit_i16(self.token.parse::<i16>()?),
            ScalarType::U16 => visitor.visit_u16(self.token.parse::<u16>()?),
            ScalarType::I32 => visitor.visit_i32(self.token.parse::<i32>()?),
            ScalarType::U32 => visitor.visit_u32(self.token.parse::<u32>()?),
            ScalarType::F32 => visitor.visit_f32(self.token.parse::<f32>()?),
            ScalarType::F64 => visitor.visit_f64(self.token.parse::<f64>()?),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct AsciiSeqAccess<'a, R> {
    reader: &'a mut R,
    data_type: ScalarType,
    remaining: usize,
}

impl<'a, 'de, R: Read> SeqAccess<'de> for AsciiSeqAccess<'a, R> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;

        let token = read_ascii_token(self.reader)?;
        seed.deserialize(AsciiScalarDeserializer {
            token,
            data_type: self.data_type,
        })
        .map(Some)
    }
}

fn read_ascii_token<R: Read>(reader: &mut R) -> Result<String, PlyError> {
    let mut token = String::new();
    let mut in_token = false;

    loop {
        let mut byte = [0u8; 1];
        match reader.read_exact(&mut byte) {
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
        return Err(PlyError::NoTokenFound);
    }

    Ok(token)
}
