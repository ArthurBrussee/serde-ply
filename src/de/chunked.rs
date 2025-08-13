use crate::{
    de::{
        val_reader::{AsciiValReader, BinValReader, ScalarReader},
        RowDeserializer,
    },
    DeserializeError, ElementDef, PlyFormat, PlyHeader,
};
use byteorder::{BigEndian, LittleEndian};
use serde::{
    de::{DeserializeSeed, Error, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{io::Cursor, marker::PhantomData};

/// Streaming PLY file parser for chunked data processing.
///
/// Processes PLY files incrementally as data becomes available, useful for
/// streaming data or async readers. Unlike [`crate::PlyReader`], this parser
/// can handle incomplete data and resume parsing when more bytes arrive.
///
/// # Example
/// ```rust
/// use serde::{Deserialize, de::DeserializeSeed};
/// use serde_ply::{PlyChunkedReader, RowVisitor};
///
/// #[derive(Deserialize)]
/// struct Vertex { x: f32, y: f32, z: f32 }
///
/// let mut file = PlyChunkedReader::new();
/// let mut vertices = Vec::new();
///
/// // Feed data in chunks
/// let data = br#"ply
/// format ascii 1.0
/// element vertex 2
/// property float x
/// property float y
/// property float z
/// end_header
/// 1.0 2.0 3.0
/// 4.0 5.0 6.0
/// "#;
///
/// for chunk in data.chunks(15) {
///     file.buffer_mut().extend_from_slice(chunk);
///
///     if file.current_element().is_some() {
///         RowVisitor::new(|v: Vertex| vertices.push(v)).deserialize(&mut file)?;
///     }
/// }
/// assert_eq!(vertices.len(), 2);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct PlyChunkedReader {
    header: Option<PlyHeader>,
    current_element_index: usize,
    rows_parsed: usize,
    data_buffer: Vec<u8>,
}

impl PlyChunkedReader {
    /// Create a new chunked PLY file parser.
    pub fn new() -> Self {
        Self {
            header: None,
            current_element_index: 0,
            rows_parsed: 0,
            data_buffer: Vec::new(),
        }
    }

    /// Get mutable access to the internal buffer.
    ///
    /// Allows writing data directly into the parser's buffer without copies.
    /// Useful for async readers or when feeding data in chunks.
    pub fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data_buffer
    }

    /// Get the parsed PLY header if available.
    ///
    /// Returns `None` if there isn't enough buffered data to parse the complete header.
    /// The header is parsed lazily when first accessed.
    ///
    /// # Example
    /// ```rust
    /// use serde_ply::PlyChunkedReader;
    ///
    /// let mut file = PlyChunkedReader::new();
    /// assert!(file.header().is_none());
    ///
    /// file.buffer_mut().extend_from_slice(
    ///     b"ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nend_header\n"
    /// );
    /// assert!(file.header().is_some());
    /// ```
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

    /// Deserialize as many complete elements as possible from the current buffer.
    ///
    /// Stops when the buffer is exhausted or an element boundary is reached.
    /// Use this for batch processing of available data.
    pub fn next_chunk<T>(&mut self) -> Result<T, DeserializeError>
    where
        T: for<'de> Deserialize<'de>,
    {
        T::deserialize(self)
    }

    /// Get the current element definition being processed.
    ///
    /// Returns `None` when the header isn't parsed yet, or when all elements
    /// have been processed. Use this to inspect element structure before parsing.
    ///
    /// # Example
    /// ```rust
    /// use serde_ply::PlyChunkedReader;
    ///
    /// let mut file = PlyChunkedReader::new();
    /// file.buffer_mut().extend_from_slice(
    ///     b"ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nend_header\n"
    /// );
    ///
    /// if let Some(element) = file.current_element() {
    ///     assert_eq!(element.name, "vertex");
    ///     assert_eq!(element.count, 1);
    /// }
    /// ```
    pub fn current_element(&mut self) -> Option<&ElementDef> {
        let ind = self.current_element_index;
        self.header().and_then(|e| e.elem_defs.get(ind))
    }

    /// Number of rows parsed so far in the current element.
    pub fn rows_done(&self) -> usize {
        self.rows_parsed
    }
}

impl<'de> Deserializer<'de> for &'_ mut PlyChunkedReader {
    type Error = DeserializeError;

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
        if self.current_element_index >= header.elem_defs.len() {
            return Err(DeserializeError::custom("Ran out of elements"));
        }

        let elem_def = &header.elem_defs[self.current_element_index];

        let mut cursor = Cursor::new(&self.data_buffer);
        let remaining = elem_def.count - self.rows_parsed;

        let (res, rows_remaining) = match header.format {
            PlyFormat::Ascii => {
                let mut seq = ChunkPlyReaderSeqVisitor {
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
                let mut seq = ChunkPlyReaderSeqVisitor {
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
                let mut seq = ChunkPlyReaderSeqVisitor {
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

        self.rows_parsed = elem_def.count - rows_remaining;
        self.data_buffer.drain(..cursor.position() as usize);

        // If we've parsed all elements move to the next element.
        if self.rows_parsed >= elem_def.count {
            self.rows_parsed = 0;
            self.current_element_index += 1;
        }

        Ok(res)
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
        bool i8 u8 i16 u16 i32 u32 f32 f64 i128 i64 u128 u64 char str string
        bytes byte_buf unit unit_struct tuple
        tuple_struct map struct enum identifier ignored_any option
    }
}

struct EmptySeq;

impl<'de> SeqAccess<'de> for EmptySeq {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, _seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        Ok(None)
    }
}

struct ChunkPlyReaderSeqVisitor<'a, D: AsRef<[u8]>, S: ScalarReader> {
    remaining: usize,
    row: RowDeserializer<'a, Cursor<D>, S>,
}

impl<'de, D: AsRef<[u8]>, S: ScalarReader> SeqAccess<'de>
    for &mut ChunkPlyReaderSeqVisitor<'_, D, S>
{
    type Error = DeserializeError;

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

impl Default for PlyChunkedReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Visitor for processing PLY rows one at a time.
///
/// Provides a callback-based interface for processing PLY elements
/// without collecting them into intermediate collections. Useful for
/// streaming processing or when memory usage is a concern.
///
/// # Example
/// ```rust
/// use serde::{Deserialize, de::DeserializeSeed};
/// use serde_ply::{PlyChunkedReader, RowVisitor};
///
/// #[derive(Deserialize)]
/// struct Vertex { x: f32, y: f32, z: f32 }
///
/// let mut file = PlyChunkedReader::new();
/// file.buffer_mut().extend_from_slice(
///     b"ply\nformat ascii 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nend_header\n1.0 2.0 3.0\n"
/// );
///
/// let mut count = 0;
/// RowVisitor::new(|_vertex: Vertex| {
///     count += 1;
/// }).deserialize(&mut file)?;
/// assert_eq!(count, 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct RowVisitor<T, F: FnMut(T)> {
    row_callback: F,
    _row: PhantomData<T>,
}

impl<T, F: FnMut(T)> RowVisitor<T, F> {
    /// Create a new row visitor with the given callback.
    ///
    /// The callback is invoked for each successfully parsed row.
    /// Call [`serde::de::DeserializeSeed::deserialize`] to start processing.
    #[must_use = "Please call deserialize(&mut file) to actually deserialize data"]
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
