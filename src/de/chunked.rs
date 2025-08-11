use crate::{
    de::{
        val_reader::{AsciiValReader, BinValReader, ScalarReader},
        RowDeserializer,
    },
    ElementDef, PlyError, PlyFormat, PlyHeader,
};
use byteorder::{BigEndian, LittleEndian};
use serde::{
    de::{DeserializeSeed, Error, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{io::Cursor, marker::PhantomData};

pub struct ChunkPlyFile {
    header: Option<PlyHeader>,
    current_element_index: usize,
    rows_parsed: usize,
    data_buffer: Vec<u8>,
}

impl ChunkPlyFile {
    pub fn new() -> Self {
        Self {
            header: None,
            current_element_index: 0,
            rows_parsed: 0,
            data_buffer: Vec::new(),
        }
    }

    /// Get mutable access to the internal buffer for zero-copy writing.
    ///
    /// This allows any reader (including async ones) to write directly into
    /// the buffer without copyies.
    pub fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data_buffer
    }

    /// Get the parsed PLY header if available.
    ///
    /// Returns `None` if there isn't enough data to complete header parsing.
    pub fn header(&mut self) -> Option<&PlyHeader> {
        if self.header.is_none() {
            let available_data = &self.data_buffer;
            let mut cursor = Cursor::new(available_data);
            let header = PlyHeader::parse(&mut cursor);
            if let Ok(header) = header {
                self.header = Some(header);
                self.data_buffer.drain(..cursor.position() as usize);
            }
        }
        self.header.as_ref()
    }

    /// Parse the next chunk of elements from the buffer.
    pub fn next_chunk<T>(&mut self) -> Result<T, PlyError>
    where
        T: for<'de> Deserialize<'de>,
    {
        T::deserialize(self)
    }

    pub fn current_element(&mut self) -> Option<&ElementDef> {
        let ind = self.current_element_index;
        self.header().map(|e| &e.elements[ind])
    }

    pub fn rows_done(&self) -> usize {
        self.rows_parsed
    }
}

impl<'de> Deserializer<'de> for &'_ mut ChunkPlyFile {
    type Error = PlyError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let _ = self.header();
        // Make sure header is parsed
        let Some(header) = &self.header else {
            return visitor.visit_seq(EmptySeq);
        };

        // Check if we've moved past all elements, if so error that we've run out of elements.
        if self.current_element_index >= header.elements.len() {
            return Err(PlyError::custom("Ran out of elements"));
        }

        let elem_def = &header.elements[self.current_element_index];

        let mut cursor = Cursor::new(&self.data_buffer);
        let remaining = elem_def.row_count - self.rows_parsed;

        let (res, rows_remaining) = match header.format {
            PlyFormat::Ascii => {
                let mut seq = ChunkPlyFileSeqVisitor {
                    remaining,
                    row: RowDeserializer::<_, AsciiValReader>::new(
                        &mut cursor,
                        &elem_def.properties,
                    ),
                };
                let res = visitor.visit_seq(&mut seq)?;
                (res, seq.remaining)
            }
            PlyFormat::BinaryLittleEndian => {
                let mut seq = ChunkPlyFileSeqVisitor {
                    remaining,
                    row: RowDeserializer::<_, BinValReader<LittleEndian>>::new(
                        &mut cursor,
                        &elem_def.properties,
                    ),
                };
                let res = visitor.visit_seq(&mut seq)?;
                (res, seq.remaining)
            }
            PlyFormat::BinaryBigEndian => {
                let mut seq = ChunkPlyFileSeqVisitor {
                    remaining,
                    row: RowDeserializer::<_, BinValReader<BigEndian>>::new(
                        &mut cursor,
                        &elem_def.properties,
                    ),
                };
                let res = visitor.visit_seq(&mut seq)?;
                (res, seq.remaining)
            }
        };

        self.rows_parsed = elem_def.row_count - rows_remaining;
        self.data_buffer.drain(..cursor.position() as usize);

        // If we've parsed all elements move to the next element.
        if self.rows_parsed >= elem_def.row_count {
            self.rows_parsed = 0;
            self.current_element_index += 1;
        }

        Ok(res)
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 f32 f64 i128 i64 u128 u64 char str string
        bytes byte_buf unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any option
    }
}

struct EmptySeq;

impl<'de> SeqAccess<'de> for EmptySeq {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, _seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        Ok(None)
    }
}

struct ChunkPlyFileSeqVisitor<'a, D: AsRef<[u8]>, S: ScalarReader> {
    remaining: usize,
    row: RowDeserializer<'a, Cursor<D>, S>,
}

impl<'de, D: AsRef<[u8]>, S: ScalarReader> SeqAccess<'de>
    for &mut ChunkPlyFileSeqVisitor<'_, D, S>
{
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        let last_pos = self.row.reader.position();
        match seed.deserialize(&mut self.row) {
            Ok(element) => {
                self.remaining -= 1;
                Ok(Some(element))
            }
            // Not enough data for this element, stop here
            Err(e) if e.0.kind() == std::io::ErrorKind::UnexpectedEof => {
                self.row.reader.set_position(last_pos);
                Ok(None)
            }
            Err(e) => Err(e)?,
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

impl Default for ChunkPlyFile {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RowVisitor<T, F: FnMut(T)> {
    row_callback: F,
    _row: PhantomData<T>,
}

impl<T, F: FnMut(T)> RowVisitor<T, F> {
    pub fn new(row_callback: F) -> Self {
        Self {
            row_callback,
            _row: PhantomData,
        }
    }
}

impl<'de, T: Deserialize<'de>, F: FnMut(T)> DeserializeSeed<'de> for &mut RowVisitor<T, F> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, T: Deserialize<'de>, F: FnMut(T)> Visitor<'de> for &mut RowVisitor<T, F> {
    type Value = ();
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence of rows")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        while let Some(row) = seq.next_element()? {
            (self.row_callback)(row);
        }
        Ok(())
    }
}
