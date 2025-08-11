use crate::{
    ser::{
        row::RowSerializer,
        val_writer::{AsciiValWriter, BinValWriter},
    },
    PlyError, PlyFormat,
};
use std::{io::Write, marker::PhantomData};

use byteorder::{BigEndian, LittleEndian};
use serde::{
    de::Error,
    ser::{SerializeMap, SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};

pub struct PlyFileSerializer<W: Write> {
    format: PlyFormat,
    writer: W,
}

impl<W: Write> PlyFileSerializer<W> {
    pub fn new(format: PlyFormat, writer: W) -> Self {
        Self { format, writer }
    }
}

impl<'a, W: Write> Serializer for &'a mut PlyFileSerializer<W> {
    type Ok = ();
    type Error = PlyError;

    type SerializeMap = PlyMapSerializer<&'a mut W>;
    type SerializeStruct = PlyMapSerializer<&'a mut W>;

    type SerializeSeq = serde::ser::Impossible<(), PlyError>;
    type SerializeTuple = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), PlyError>;
    type SerializeStructVariant = serde::ser::Impossible<(), PlyError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(PlyMapSerializer {
            format: self.format,
            writer: &mut self.writer,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(PlyMapSerializer {
            format: self.format,
            writer: &mut self.writer,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(serde::ser::Error::custom("struct variants not supported"))
    }
}

pub struct PlyMapSerializer<W: Write> {
    format: PlyFormat,
    writer: W,
}

impl<W: Write> SerializeMap for PlyMapSerializer<W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, _key: &T) -> Result<(), Self::Error> {
        // Capture the element name
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        // Each value should be a Vec<Row> representing an element
        value.serialize(ElementSerializer {
            format: self.format,
            writer: &mut self.writer,
            _ph: PhantomData,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: Write> SerializeStruct for PlyMapSerializer<W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        // Each field represents an element (e.g., "vertex", "face")
        // The value should be a Vec<Row>
        value.serialize(ElementSerializer {
            format: self.format,
            writer: &mut self.writer,
            _ph: PhantomData,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

struct ElementSerializer<'a, W: Write> {
    format: PlyFormat,
    writer: &'a mut W,
    _ph: PhantomData<&'a W>,
}

impl<'a, W: Write> Serializer for ElementSerializer<'a, W> {
    type Ok = ();
    type Error = PlyError;
    type SerializeSeq = ElementSeqSerializer<'a, W>;
    type SerializeTuple = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), PlyError>;
    type SerializeMap = serde::ser::Impossible<(), PlyError>;
    type SerializeStruct = serde::ser::Impossible<(), PlyError>;
    type SerializeStructVariant = serde::ser::Impossible<(), PlyError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(serde::ser::Error::custom("variants not supported"))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(serde::ser::Error::custom("variants not supported"))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let count = len.unwrap_or(0);

        Ok(ElementSeqSerializer {
            format: self.format,
            count,
            current: 0,
            writer: self.writer,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(serde::ser::Error::custom("elements must be sequences"))
    }
}

pub struct ElementSeqSerializer<'a, W: Write> {
    format: PlyFormat,
    count: usize,
    current: usize,
    writer: &'a mut W,
}

impl<W: Write> SerializeSeq for ElementSeqSerializer<'_, W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        if self.current >= self.count {
            return Err(serde::ser::Error::custom("too many elements"));
        }

        match self.format {
            PlyFormat::Ascii => {
                value.serialize(&mut RowSerializer::new(AsciiValWriter::new(
                    &mut self.writer,
                )))?;
            }
            PlyFormat::BinaryBigEndian => {
                value.serialize(&mut RowSerializer::new(BinValWriter::<_, BigEndian>::new(
                    &mut self.writer,
                )))?;
            }
            PlyFormat::BinaryLittleEndian => {
                value.serialize(&mut RowSerializer::new(
                    BinValWriter::<_, LittleEndian>::new(&mut self.writer),
                ))?;
            }
        }
        self.current += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.current != self.count {
            return Err(serde::ser::Error::custom("element count mismatch"));
        }
        Ok(())
    }
}
