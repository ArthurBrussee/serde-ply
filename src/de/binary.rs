use std::{io::Read, marker::PhantomData};

use byteorder::ByteOrder;
use serde::{
    de::{
        value::BytesDeserializer, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor,
    },
    Deserializer,
};

use crate::{ElementDef, PlyError, PropertyType, ScalarType};
use byteorder::ReadBytesExt;

pub(crate) struct BinaryRowDeserializer<'e, R, E> {
    pub reader: R,
    pub elem_def: &'e ElementDef,
    pub _endian: PhantomData<E>,
}

impl<'e, R: Read, E: ByteOrder> BinaryRowDeserializer<'e, R, E> {
    pub fn new(reader: R, elem_def: &'e ElementDef) -> Self {
        Self {
            reader,
            elem_def,
            _endian: PhantomData,
        }
    }
}

impl<'de, 'e: 'de, R: Read, E: ByteOrder> Deserializer<'de>
    for &mut BinaryRowDeserializer<'e, R, E>
{
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

struct BinaryRowMap<'a, 'e, R, E> {
    parent: &'a mut BinaryRowDeserializer<'e, R, E>,
    current_property: usize,
    _endian: PhantomData<E>,
}

impl<'de, 'a, 'e, R: Read, E: ByteOrder> MapAccess<'de> for BinaryRowMap<'a, 'e, R, E> {
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

impl<'a, 'de, R: Read, E: ByteOrder> SeqAccess<'de> for BinarySeqAccess<'a, R, E> {
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
