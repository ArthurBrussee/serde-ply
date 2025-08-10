use core::fmt;
use serde::de::value::BytesDeserializer;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::io::{BufRead, Read};
use std::marker::PhantomData;

use crate::de::val_reader::{AsciiValReader, BinValReader, ScalarReader};
use crate::de::RowDeserializer;
use crate::{ElementDef, PlyError, PlyFormat, PlyHeader};
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

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 i64 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple
        tuple_struct enum identifier ignored_any seq
    }
}

impl<'de, R: Read> MapAccess<'de> for &mut PlyFileDeserializer<R> {
    type Error = PlyError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.current_element >= self.header.elements.len() {
            return Ok(None);
        }
        let element_name = &self.header.elements[self.current_element].name.as_bytes();
        seed.deserialize(BytesDeserializer::new(element_name))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        // One clone per the whole element is probably fine, simplifies lifetimes a good amount.
        let elem_def = &self.header.elements[self.current_element];
        self.current_element += 1;

        match self.header.format {
            PlyFormat::Ascii => seed.deserialize(ElementSeqDeserializer::new(
                elem_def,
                &mut AsciiValReader::new(&mut self.reader),
                elem_def.row_count,
            )),
            PlyFormat::BinaryLittleEndian => seed.deserialize(ElementSeqDeserializer::new(
                elem_def,
                &mut BinValReader::<_, LittleEndian>::new(&mut self.reader),
                elem_def.row_count,
            )),
            PlyFormat::BinaryBigEndian => seed.deserialize(ElementSeqDeserializer::new(
                elem_def,
                &mut BinValReader::<_, BigEndian>::new(&mut self.reader),
                elem_def.row_count,
            )),
        }
    }
}

struct ElementSeqDeserializer<'a, E: ScalarReader> {
    elem_def: &'a ElementDef,
    val_reader: &'a mut E,
    remaining: usize,
}

impl<'a, E: ScalarReader> ElementSeqDeserializer<'a, E> {
    fn new(elem_def: &'a ElementDef, reader: &'a mut E, row_count: usize) -> Self {
        Self {
            elem_def,
            val_reader: reader,
            remaining: row_count,
        }
    }
}

impl<'de, 'a, R: ScalarReader> Deserializer<'de> for ElementSeqDeserializer<'a, R> {
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

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct map struct tuple
        tuple_struct enum identifier ignored_any i64 u64
    }
}

impl<'de, 'a, R: ScalarReader> SeqAccess<'de> for ElementSeqDeserializer<'a, R> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(RowDeserializer {
            val_reader: self.val_reader,
            elem_def: self.elem_def,
        })
        .map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}
