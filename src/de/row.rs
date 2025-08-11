use crate::{de::val_reader::ScalarReader, PlyError, PlyProperty, PropertyType, ScalarType};
use serde::{
    de::{value::BytesDeserializer, DeserializeSeed, Error, MapAccess, SeqAccess, Visitor},
    Deserializer,
};
use std::{io::Read, marker::PhantomData};

pub(crate) struct RowDeserializer<'a, R: Read, S: ScalarReader> {
    pub reader: &'a mut R,
    pub properties: &'a [PlyProperty],
    pub current_property: u32,
    _marker: PhantomData<S>,
}

impl<'a, R: Read, S: ScalarReader> RowDeserializer<'a, R, S> {
    pub fn new(reader: &'a mut R, properties: &'a [PlyProperty]) -> Self {
        Self {
            current_property: 0,
            reader,
            properties,
            _marker: PhantomData,
        }
    }
}

impl<'de, R: Read, S: ScalarReader> Deserializer<'de> for &mut RowDeserializer<'_, R, S> {
    type Error = PlyError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PlyError::custom("Rows must be deserialized as maps"))
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
        self.current_property = 0;
        visitor.visit_map(self)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.current_property = 0;
        visitor.visit_map(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier ignored_any
    }
}

impl<'de, R: Read, S: ScalarReader> MapAccess<'de> for RowDeserializer<'_, R, S> {
    type Error = PlyError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let Some(prop) = self.properties.get(self.current_property as usize) else {
            return Ok(None);
        };
        seed.deserialize(BytesDeserializer::new(prop.name.as_bytes()))
            .map(Some)
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        // TODO: Hope the bounds check here gets optimzed out (next_key_seed already checks).
        match self.properties[self.current_property as usize].property_type {
            PropertyType::Scalar(data_type) => {
                self.current_property += 1;
                seed.deserialize(ScalarDeserializer {
                    reader: &mut self.reader,
                    data_type,
                    _marker: PhantomData::<S>,
                })
            }
            PropertyType::List {
                count_type,
                data_type,
            } => {
                self.current_property += 1;
                seed.deserialize(ListDeserializer {
                    reader: &mut self.reader,
                    count_type,
                    data_type,
                    _marker: PhantomData::<S>,
                })
            }
        }
    }
}

struct ScalarDeserializer<'a, R: Read, S: ScalarReader> {
    reader: &'a mut R,
    data_type: ScalarType,
    _marker: PhantomData<S>,
}

impl<'de, R: Read, S: ScalarReader> Deserializer<'de> for ScalarDeserializer<'_, R, S> {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.data_type {
            ScalarType::I8 => visitor.visit_i8(S::read_i8(self.reader)?),
            ScalarType::U8 => visitor.visit_u8(S::read_u8(self.reader)?),
            ScalarType::I16 => visitor.visit_i16(S::read_i16(self.reader)?),
            ScalarType::U16 => visitor.visit_u16(S::read_u16(self.reader)?),
            ScalarType::I32 => visitor.visit_i32(S::read_i32(self.reader)?),
            ScalarType::U32 => visitor.visit_u32(S::read_u32(self.reader)?),
            ScalarType::F32 => visitor.visit_f32(S::read_f32(self.reader)?),
            ScalarType::F64 => visitor.visit_f64(S::read_f64(self.reader)?),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 f32 f64 i128 i64 u128 u64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ListDeserializer<R: Read, S: ScalarReader> {
    reader: R,
    count_type: ScalarType,
    data_type: ScalarType,
    _marker: PhantomData<S>,
}

impl<'de, R: Read, S: ScalarReader> Deserializer<'de> for ListDeserializer<R, S> {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // PLY properties are always present if defined in header
        visitor.visit_some(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let count = match self.count_type {
            ScalarType::I8 => S::read_i8(&mut self.reader)? as usize,
            ScalarType::U8 => S::read_u8(&mut self.reader)? as usize,
            ScalarType::I16 => S::read_i16(&mut self.reader)? as usize,
            ScalarType::U16 => S::read_u16(&mut self.reader)? as usize,
            ScalarType::I32 => S::read_i32(&mut self.reader)? as usize,
            ScalarType::U32 => S::read_u32(&mut self.reader)? as usize,
            ScalarType::F32 => return Err(PlyError::custom("List count cannot be a float")),
            ScalarType::F64 => return Err(PlyError::custom("List count cannot be a float")),
        };
        visitor.visit_seq(ListSeqAccess {
            reader: &mut self.reader,
            remaining: count,
            data_type: self.data_type,
            _marker: PhantomData::<S>,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 f32 f64 i128 i64 u128 u64 char str string
        bytes byte_buf unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ListSeqAccess<R: Read, S> {
    reader: R,
    data_type: ScalarType,
    remaining: usize,
    _marker: PhantomData<S>,
}

impl<'de, R: Read, S: ScalarReader> SeqAccess<'de> for ListSeqAccess<R, S> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(ScalarDeserializer {
            reader: &mut self.reader,
            data_type: self.data_type,
            _marker: PhantomData::<S>,
        })
        .map(Some)
    }
}
