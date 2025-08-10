use crate::{ser::val_writer::ScalarWriter, PlyError, ScalarType};

use std::marker::PhantomData;

use serde::{
    ser::{SerializeMap, SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};

pub(crate) struct RowSerializer<W: ScalarWriter> {
    pub val_writer: W,
}

impl<W: ScalarWriter> RowSerializer<W> {
    pub fn new(val_writer: W) -> Self {
        Self { val_writer }
    }
}

impl<'a, W: ScalarWriter> Serializer for &'a mut RowSerializer<W> {
    type Ok = ();
    type Error = PlyError;

    // Only support struct and map serialization for PLY rows
    type SerializeSeq = serde::ser::Impossible<(), PlyError>;
    type SerializeTuple = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), PlyError>;
    type SerializeMap = RowMapSerializer<'a, W>;
    type SerializeStruct = RowMapSerializer<'a, W>;
    type SerializeStructVariant = serde::ser::Impossible<(), PlyError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(PlyError::InvalidStructure)
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
        Err(PlyError::InvalidStructure)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(RowMapSerializer {
            parent: self,
            _marker: PhantomData,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(RowMapSerializer {
            parent: self,
            _marker: PhantomData,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(serde::ser::Error::custom(
            "PLY rows must be structs or maps",
        ))
    }
}

pub struct RowMapSerializer<'a, W: ScalarWriter> {
    parent: &'a mut RowSerializer<W>,
    _marker: PhantomData<W>,
}

impl<'a, W: ScalarWriter> SerializeMap for RowMapSerializer<'a, W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(PropertySerializer {
            val_writer: &mut self.parent.val_writer,
        })?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.parent.val_writer.write_row_end()?;
        Ok(())
    }
}

impl<'a, W: ScalarWriter> SerializeStruct for RowMapSerializer<'a, W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(PropertySerializer {
            val_writer: &mut self.parent.val_writer,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.parent.val_writer.write_row_end()?;
        Ok(())
    }
}

struct PropertySerializer<'a, W: ScalarWriter> {
    val_writer: &'a mut W,
}

impl<'a, W: ScalarWriter> Serializer for PropertySerializer<'a, W> {
    type Ok = ();
    type Error = PlyError;
    type SerializeSeq = ListSerializer<'a, W>;
    type SerializeTuple = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), PlyError>;
    type SerializeMap = serde::ser::Impossible<(), PlyError>;
    type SerializeStruct = serde::ser::Impossible<(), PlyError>;
    type SerializeStructVariant = serde::ser::Impossible<(), PlyError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("bool".to_string()))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.val_writer.write_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.val_writer.write_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.val_writer.write_i32(v)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("i64".to_string()))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.val_writer.write_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.val_writer.write_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.val_writer.write_u32(v)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("u64".to_string()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.val_writer.write_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.val_writer.write_f64(v)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("char".to_string()))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("str".to_string()))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("bytes".to_string()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("none".to_string()))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("unit".to_string()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("unit_struct".to_string()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("unit_variant".to_string()))
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
        Err(PlyError::UnsupportedType("newtype_variant".to_string()))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let count = len.ok_or_else(|| {
            PlyError::UnsupportedType("sequence without known length".to_string())
        })?;

        if count > 255 {
            return Err(PlyError::UnsupportedType(
                "sequence length exceeds maximum".to_string(),
            ));
        }

        // TODO: How to support this properly?
        let count_type = ScalarType::U8;

        // Write the count
        match count_type {
            ScalarType::I8 => self.val_writer.write_i8(count as i8)?,
            ScalarType::U8 => self.val_writer.write_u8(count as u8)?,
            ScalarType::I16 => self.val_writer.write_i16(count as i16)?,
            ScalarType::U16 => self.val_writer.write_u16(count as u16)?,
            ScalarType::I32 => self.val_writer.write_i32(count as i32)?,
            ScalarType::U32 => self.val_writer.write_u32(count as u32)?,
            ScalarType::F32 => self.val_writer.write_f32(count as f32)?,
            ScalarType::F64 => self.val_writer.write_f64(count as f64)?,
        }

        Ok(ListSerializer {
            val_writer: self.val_writer,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(serde::ser::Error::custom("tuples not supported"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(serde::ser::Error::custom("tuple structs not supported"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(serde::ser::Error::custom("tuple variants not supported"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(serde::ser::Error::custom("maps not supported"))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(serde::ser::Error::custom("structs not supported"))
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

pub struct ListSerializer<'a, W: ScalarWriter> {
    val_writer: &'a mut W,
}

impl<'a, W: ScalarWriter> SerializeSeq for ListSerializer<'a, W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(PropertySerializer {
            val_writer: self.val_writer,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
