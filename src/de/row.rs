use std::marker::PhantomData;

use serde::{
    de::{value::BytesDeserializer, DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserializer,
};

use crate::{de::val_reader::ScalarReader, ElementDef, PlyError, PropertyType, ScalarType};

pub(crate) struct RowDeserializer<E: ScalarReader> {
    pub val_reader: E,
    pub elem_def: ElementDef,
}

impl<E: ScalarReader> RowDeserializer<E> {
    pub fn new(val_reader: E, elem_def: ElementDef) -> Self {
        Self {
            val_reader,
            elem_def,
        }
    }
}

impl<'de, E: ScalarReader> Deserializer<'de> for &mut RowDeserializer<E> {
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

pub struct RowMapAccess<'a, E: ScalarReader> {
    pub parent: &'a mut RowDeserializer<E>,
    pub current_property: usize,
    pub _endian: PhantomData<E>,
}

impl<'de, 'a, E: ScalarReader> MapAccess<'de> for RowMapAccess<'a, E> {
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
            val_reader: &mut self.parent.val_reader,
            prop,
        })
    }
}

struct ValueDeserializer<'a, E: ScalarReader> {
    val_reader: &'a mut E,
    prop: &'a PropertyType,
}

impl<'de, 'a, E: ScalarReader> Deserializer<'de> for ValueDeserializer<'a, E> {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.prop {
            PropertyType::Scalar { data_type } => match data_type {
                ScalarType::I8 => visitor.visit_i8(self.val_reader.read_i8()?),
                ScalarType::U8 => visitor.visit_u8(self.val_reader.read_u8()?),
                ScalarType::I16 => visitor.visit_i16(self.val_reader.read_i16()?),
                ScalarType::U16 => visitor.visit_u16(self.val_reader.read_u16()?),
                ScalarType::I32 => visitor.visit_i32(self.val_reader.read_i32()?),
                ScalarType::U32 => visitor.visit_u32(self.val_reader.read_u32()?),
                ScalarType::F32 => visitor.visit_f32(self.val_reader.read_f32()?),
                ScalarType::F64 => visitor.visit_f64(self.val_reader.read_f64()?),
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
            ScalarType::I8 => self.val_reader.read_i8()? as usize,
            ScalarType::U8 => self.val_reader.read_u8()? as usize,
            ScalarType::I16 => self.val_reader.read_i16()? as usize,
            ScalarType::U16 => self.val_reader.read_u16()? as usize,
            ScalarType::I32 => self.val_reader.read_i32()? as usize,
            ScalarType::U32 => self.val_reader.read_u32()? as usize,
            ScalarType::F32 => self.val_reader.read_f32()? as usize,
            ScalarType::F64 => self.val_reader.read_f64()? as usize,
        };

        visitor.visit_seq(ListSeqAccess {
            val_reader: self.val_reader,
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
    val_reader: &'a mut E,
    data_type: ScalarType,
    remaining: usize,
    _endian: PhantomData<E>,
}

impl<'a, 'de, E: ScalarReader> SeqAccess<'de> for ListSeqAccess<'a, E> {
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
            reader: self.val_reader,
            data_type: self.data_type,
        })
        .map(Some)
    }
}

struct ScalarDeserializer<'a, E> {
    reader: &'a mut E,
    data_type: ScalarType,
}

impl<'a, 'de, E: ScalarReader> Deserializer<'de> for ScalarDeserializer<'a, E> {
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
