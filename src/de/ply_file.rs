use core::fmt;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::io::{BufRead, Read};
use std::marker::PhantomData;

use crate::de::val_reader::{AsciiValReader, BinValReader, ScalarReader};
use crate::de::RowDeserializer;
use crate::{PlyError, PlyFormat, PlyHeader};
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
        let a = deserialize_single_value(self)?;
        Ok(a)
    }
}

// Deserialize exactly a single value from a map deserializer
fn deserialize_single_value<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
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
    deserializer.deserialize_map(FirstValueVisitor(PhantomData))
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
        // TODO: Remove the cloning for the header here.
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

        let element_name = &self.header.elements[self.current_element].name;
        seed.deserialize(serde::de::value::StrDeserializer::new(element_name))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        // One clone per the whole element is probably fine, simplifies lifetimes a good amount.
        let elem_def = &self.header.elements[self.current_element];

        let result = match self.header.format {
            PlyFormat::Ascii => seed.deserialize(ElementSeqDeserializer::new(
                RowDeserializer::new(AsciiValReader::new(&mut self.reader), elem_def.clone()),
                elem_def.row_count,
            )),
            PlyFormat::BinaryLittleEndian => seed.deserialize(ElementSeqDeserializer::new(
                RowDeserializer::new(
                    BinValReader::<_, LittleEndian>::new(&mut self.reader),
                    elem_def.clone(),
                ),
                elem_def.row_count,
            )),
            PlyFormat::BinaryBigEndian => seed.deserialize(ElementSeqDeserializer::new(
                RowDeserializer::new(
                    BinValReader::<_, BigEndian>::new(&mut self.reader),
                    elem_def.clone(),
                ),
                elem_def.row_count,
            )),
        };

        self.current_element += 1;
        result
    }
}

struct ElementSeqDeserializer<E: ScalarReader> {
    row_deserialize: RowDeserializer<E>,
    remaining: usize,
}

impl<E: ScalarReader> ElementSeqDeserializer<E> {
    fn new(row_deserialize: RowDeserializer<E>, row_count: usize) -> Self {
        Self {
            row_deserialize,
            remaining: row_count,
        }
    }
}

impl<'de, R: ScalarReader> Deserializer<'de> for ElementSeqDeserializer<R> {
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

impl<'de, R: ScalarReader> SeqAccess<'de> for ElementSeqDeserializer<R> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(&mut self.row_deserialize).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}
