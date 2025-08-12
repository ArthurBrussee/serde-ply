use core::fmt;
use serde::de::value::BytesDeserializer;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::io::{BufRead, Read};
use std::marker::PhantomData;

use crate::de::val_reader::{AsciiValReader, BinValReader, ScalarReader};
use crate::de::RowDeserializer;
use crate::{PlyError, PlyFormat, PlyHeader, PlyProperty};
use byteorder::{BigEndian, LittleEndian};

pub struct PlyFileDeserializer<R> {
    reader: R,
    header: PlyHeader,
    current_element: usize,
}

impl<R: BufRead> PlyFileDeserializer<R> {
    pub fn from_reader(mut reader: R) -> Result<Self, PlyError> {
        let header = PlyHeader::parse(&mut reader)?;
        Ok(Self {
            reader,
            header,
            current_element: 0,
        })
    }

    pub fn header(&self) -> &PlyHeader {
        &self.header
    }

    pub fn next_element<'a, T>(&mut self) -> Result<T, PlyError>
    where
        T: Deserialize<'a>,
    {
        // Deserialize exactly a single value from a map deserializer.
        struct FirstValueVisitor<T>(PhantomData<T>);
        impl<'de, T> Visitor<'de> for FirstValueVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = T;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with at least one entry")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                // Use next value directly. This is NOT ok in general as next_key might
                // increment things instead but it's fine we're in charge.
                map.next_value::<T>()
            }
        }
        self.deserialize_map(FirstValueVisitor(PhantomData))
    }
}

impl<'de, R: BufRead> Deserializer<'de> for &mut PlyFileDeserializer<R> {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
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
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct tuple
        tuple_struct enum identifier ignored_any seq
    }
}

impl<'de, R: Read> MapAccess<'de> for &mut PlyFileDeserializer<R> {
    type Error = PlyError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.current_element >= self.header.elem_defs.len() {
            return Ok(None);
        }
        let element_name = &self.header.elem_defs[self.current_element].name.as_bytes();
        seed.deserialize(BytesDeserializer::new(element_name))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let elem_def = &self.header.elem_defs[self.current_element];
        self.current_element += 1;

        match self.header.format {
            PlyFormat::Ascii => seed.deserialize(ElementSeqDeserializer::<_, AsciiValReader>::new(
                &elem_def.properties,
                &mut self.reader,
                elem_def.count,
            )),
            PlyFormat::BinaryLittleEndian => seed.deserialize(ElementSeqDeserializer::<
                _,
                BinValReader<LittleEndian>,
            >::new(
                &elem_def.properties,
                &mut self.reader,
                elem_def.count,
            )),
            PlyFormat::BinaryBigEndian => {
                seed.deserialize(ElementSeqDeserializer::<_, BinValReader<BigEndian>>::new(
                    &elem_def.properties,
                    &mut self.reader,
                    elem_def.count,
                ))
            }
        }
    }
}

pub(crate) struct ElementSeqDeserializer<'a, R: Read, S: ScalarReader> {
    row: RowDeserializer<'a, R, S>,
    remaining: usize,
}

impl<'a, R: Read, S: ScalarReader> ElementSeqDeserializer<'a, R, S> {
    pub(crate) fn new(properties: &'a [PlyProperty], reader: &'a mut R, row_count: usize) -> Self {
        Self {
            row: RowDeserializer::new(reader, properties),
            remaining: row_count,
        }
    }
}

impl<'de, R: Read, S: ScalarReader> Deserializer<'de> for ElementSeqDeserializer<'_, R, S> {
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
        visitor.visit_seq(self)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 f32 f64 char str string
        bytes byte_buf unit unit_struct map struct tuple
        tuple_struct enum identifier ignored_any i64 u64
    }
}

impl<'de, R: Read, S: ScalarReader> SeqAccess<'de> for ElementSeqDeserializer<'_, R, S> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(&mut self.row).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}
