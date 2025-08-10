use crate::{
    de::{
        val_reader::{AsciiValReader, BinValReader},
        RowDeserializer,
    },
    PlyError, PlyFormat, PlyHeader,
};
use byteorder::{BigEndian, LittleEndian};
use serde::{de::SeqAccess, Deserialize, Deserializer};
use std::io::Cursor;

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
            return Err(PlyError::MissingElement);
        }
        visitor.visit_seq(ChunkPlyFileSeqVisitor { parent: self })
    }

    serde::forward_to_deserialize_any! {
        bool i8 u8 i16 u16 i32 u32 f32 f64 i128 i64 u128 u64 char str string
        bytes byte_buf unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any option
    }
}

struct ChunkPlyFileSeqVisitor<'a> {
    parent: &'a mut ChunkPlyFile,
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

impl<'de> SeqAccess<'de> for ChunkPlyFileSeqVisitor<'_> {
    type Error = PlyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let header = &self.parent.header.as_ref().unwrap();
        let element_def = &header.elements[self.parent.current_element_index];

        // If we've parsed all elements move to the next element.
        if self.parent.rows_parsed >= element_def.row_count {
            self.parent.rows_parsed = 0;
            self.parent.current_element_index += 1;
            return Ok(None);
        }

        let mut cursor = Cursor::new(&self.parent.data_buffer);

        let elem = match header.format {
            PlyFormat::Ascii => {
                let mut val = AsciiValReader::new(&mut cursor);
                seed.deserialize(RowDeserializer::new(&mut val, element_def))
            }
            PlyFormat::BinaryLittleEndian => {
                let mut val = BinValReader::<_, LittleEndian>::new(&mut cursor);
                seed.deserialize(RowDeserializer::new(&mut val, element_def))
            }
            PlyFormat::BinaryBigEndian => {
                let mut val = BinValReader::<_, BigEndian>::new(&mut cursor);
                seed.deserialize(RowDeserializer::new(&mut val, element_def))
            }
        };

        match elem {
            Ok(element) => {
                // Remove consumed bytes from buffer
                self.parent.data_buffer.drain(..cursor.position() as usize);
                self.parent.rows_parsed += 1;
                Ok(Some(element))
            }
            // Not enough data for this element, stop here
            Err(PlyError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl Default for ChunkPlyFile {
    fn default() -> Self {
        Self::new()
    }
}
