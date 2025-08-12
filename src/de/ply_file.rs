use core::fmt;
use serde::de::value::BytesDeserializer;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::io::{BufRead, Read};
use std::marker::PhantomData;

use crate::de::val_reader::{AsciiValReader, BinValReader, ScalarReader};
use crate::de::RowDeserializer;
use crate::{DeserializeError, ElementDef, PlyFormat, PlyHeader, PlyProperty};
use byteorder::{BigEndian, LittleEndian};

/// PLY file deserializer.
///
/// Also provides fine-grained control over PLY file parsing by allowing you to
/// deserialize individual elements sequentially. This is useful when you need
/// to handle different element types separately or process large files incrementally.
///
/// # Examples
///
/// ```rust
/// use serde::Deserialize;
/// use serde_ply::PlyReader;
/// use std::io::{BufReader, Cursor};
///
/// #[derive(Deserialize)]
/// struct Vertex { x: f32, y: f32, z: f32 }
///
/// #[derive(Deserialize)]
/// struct Face { vertex_indices: Vec<u32> }
///
/// let ply_data = "ply\nformat ascii 1.0\nelement vertex 1\n\
///                 property float x\nproperty float y\nproperty float z\n\
///                 element face 1\nproperty list uchar uint vertex_indices\n\
///                 end_header\n1.0 2.0 3.0\n3 0 1 2\n";
///
/// let cursor = Cursor::new(ply_data);
/// let mut deserializer = PlyReader::from_reader(BufReader::new(cursor))?;
///
/// let vertices: Vec<Vertex> = deserializer.next_element()?;
/// let faces: Vec<Face> = deserializer.next_element()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct PlyReader<R> {
    reader: R,
    header: PlyHeader,
    current_element: usize,
}

impl<R: BufRead> PlyReader<R> {
    /// Create a new PLY file deserializer from a buffered reader.
    ///
    /// Parses the PLY header immediately. Use [`Self::next_element`] to
    /// deserialize individual elements sequentially.
    pub fn from_reader(mut reader: R) -> Result<Self, DeserializeError> {
        let header = PlyHeader::parse(&mut reader)?;
        Ok(Self {
            reader,
            header,
            current_element: 0,
        })
    }

    /// Get the parsed PLY header.
    pub fn header(&self) -> &PlyHeader {
        &self.header
    }

    /// Get the current element definition.
    ///
    /// Returns the element that will be deserialized by the next call to
    /// [`Self::next_element`]. Returns `None` if all elements have been processed.
    pub fn current_element(&self) -> Option<&ElementDef> {
        self.header.elem_defs.get(self.current_element)
    }

    /// Deserialize the next element.
    ///
    /// The type `T` should typically be a sequen of rows eg. `Vec<RowType>` where `RowType` matches
    /// the properties of the current element. Use [`Self::current_element`] to
    /// inspect the element definition.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde::Deserialize;
    /// # use serde_ply::PlyReader;
    /// # use std::io::{BufReader, Cursor};
    ///
    /// #[derive(Deserialize)]
    /// struct Vertex { x: f32, y: f32, z: f32 }
    ///
    /// # let ply_data = "ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nend_header\n1.0 2.0 3.0\n";
    /// # let cursor = Cursor::new(ply_data);
    /// # let mut deserializer = PlyReader::from_reader(BufReader::new(cursor))?;
    /// let vertices: Vec<Vertex> = deserializer.next_element()?;
    /// assert_eq!(vertices.len(), 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn next_element<'a, T>(&mut self) -> Result<T, DeserializeError>
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

impl<'de, R: BufRead> Deserializer<'de> for &mut PlyReader<R> {
    type Error = DeserializeError;

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

impl<'de, R: Read> MapAccess<'de> for &mut PlyReader<R> {
    type Error = DeserializeError;

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
    type Error = DeserializeError;

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
    type Error = DeserializeError;

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
