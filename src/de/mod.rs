mod ply_file;

pub mod val_reader;

use std::marker::PhantomData;

use serde::{
    de::{value::BytesDeserializer, DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserializer,
};

use crate::{de::val_reader::ValReader, ElementDef, PlyError, PropertyType, ScalarType};

pub(crate) struct RowDeserializer<'e, E: ValReader> {
    pub val_read: E,
    pub elem_def: &'e ElementDef,
}

impl<'e, E: ValReader> RowDeserializer<'e, E> {
    pub fn new(val_read: E, elem_def: &'e ElementDef) -> Self {
        Self { val_read, elem_def }
    }
}

impl<'de, 'e: 'de, E: ValReader> Deserializer<'de> for &mut RowDeserializer<'e, E> {
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
        visitor.visit_map(RowMapAccess {
            parent: self,
            current_property: 0,
            _endian: PhantomData,
        })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(RowMapAccess {
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

struct RowMapAccess<'a, 'e, E: ValReader> {
    parent: &'a mut RowDeserializer<'e, E>,
    current_property: usize,
    _endian: PhantomData<E>,
}

impl<'de, 'a, 'e, E: ValReader> MapAccess<'de> for RowMapAccess<'a, 'e, E> {
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

        seed.deserialize(ValueDeserializer {
            parent: self.parent,
            prop,
        })
    }
}

struct ValueDeserializer<'a, 'e, E: ValReader> {
    parent: &'a mut RowDeserializer<'e, E>,
    prop: &'a PropertyType,
}

impl<'de, 'a, 'e, E: ValReader> Deserializer<'de> for ValueDeserializer<'a, 'e, E> {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let reader = &mut self.parent.val_read;
        match self.prop {
            PropertyType::Scalar { data_type } => match data_type {
                ScalarType::I8 => visitor.visit_i8(reader.read_i8()?),
                ScalarType::U8 => visitor.visit_u8(reader.read_u8()?),
                ScalarType::I16 => visitor.visit_i16(reader.read_i16()?),
                ScalarType::U16 => visitor.visit_u16(reader.read_u16()?),
                ScalarType::I32 => visitor.visit_i32(reader.read_i32()?),
                ScalarType::U32 => visitor.visit_u32(reader.read_u32()?),
                ScalarType::F32 => visitor.visit_f32(reader.read_f32()?),
                ScalarType::F64 => visitor.visit_f64(reader.read_f64()?),
            },
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

        let count = match count_type {
            ScalarType::I8 => self.parent.val_read.read_i8()? as usize,
            ScalarType::U8 => self.parent.val_read.read_u8()? as usize,
            ScalarType::I16 => self.parent.val_read.read_i16()? as usize,
            ScalarType::U16 => self.parent.val_read.read_u16()? as usize,
            ScalarType::I32 => self.parent.val_read.read_i32()? as usize,
            ScalarType::U32 => self.parent.val_read.read_u32()? as usize,
            ScalarType::F32 => self.parent.val_read.read_f32()? as usize,
            ScalarType::F64 => self.parent.val_read.read_f64()? as usize,
        };

        visitor.visit_seq(ListSeqAccess {
            reader: &mut self.parent.val_read,
            remaining: count,
            data_type: *data_type,
            _endian: PhantomData::<E>,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 f32 f64 i128 i64 u128 u64 char str string
        bytes byte_buf unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ListSeqAccess<'a, E> {
    reader: &'a mut E,
    data_type: ScalarType,
    remaining: usize,
    _endian: PhantomData<E>,
}

impl<'a, 'de, E: ValReader> SeqAccess<'de> for ListSeqAccess<'a, E> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;

        seed.deserialize(ScalarDeserializer::<E> {
            reader: self.reader,
            data_type: self.data_type,
        })
        .map(Some)
    }
}

struct ScalarDeserializer<'a, E> {
    reader: &'a mut E,
    data_type: ScalarType,
}

impl<'a, 'de, E: ValReader> Deserializer<'de> for ScalarDeserializer<'a, E> {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.data_type {
            ScalarType::I8 => visitor.visit_i8(self.reader.read_i8()?),
            ScalarType::U8 => visitor.visit_u8(self.reader.read_u8()?),
            ScalarType::I16 => visitor.visit_i16(self.reader.read_i16()?),
            ScalarType::U16 => visitor.visit_u16(self.reader.read_u16()?),
            ScalarType::I32 => visitor.visit_i32(self.reader.read_i32()?),
            ScalarType::U32 => visitor.visit_u32(self.reader.read_u32()?),
            ScalarType::F32 => visitor.visit_f32(self.reader.read_f32()?),
            ScalarType::F64 => visitor.visit_f64(self.reader.read_f64()?),
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
