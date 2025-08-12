use crate::{ser::val_writer::ScalarWriter, PlyError, ScalarType};

use std::marker::PhantomData;

use serde::{
    ser::{Error, SerializeMap, SerializeSeq, SerializeStruct},
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
        Err(PlyError::custom("Invalid ply structure"))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(PlyError::custom("Invalid ply structure"))
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
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Err(PlyError::custom("Invalid ply structure"))
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

impl<W: ScalarWriter> SerializeMap for RowMapSerializer<'_, W> {
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
            count_type: ScalarType::U8,
        })?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.parent.val_writer.write_row_end()?;
        Ok(())
    }
}

impl<W: ScalarWriter> SerializeStruct for RowMapSerializer<'_, W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(PropertySerializer {
            val_writer: &mut self.parent.val_writer,
            count_type: ScalarType::U8,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.parent.val_writer.write_row_end()?;
        Ok(())
    }
}

struct PropertySerializer<'a, W: ScalarWriter> {
    val_writer: &'a mut W,
    count_type: ScalarType,
}

impl<'a, W: ScalarWriter> Serializer for PropertySerializer<'a, W> {
    type Ok = ();
    type Error = PlyError;
    type SerializeSeq = ListValuesSerializer<'a, W>;
    type SerializeTuple = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), PlyError>;
    type SerializeMap = serde::ser::Impossible<(), PlyError>;
    type SerializeStruct = serde::ser::Impossible<(), PlyError>;
    type SerializeStructVariant = serde::ser::Impossible<(), PlyError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Unsupported type: bool"))
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
        Err(PlyError::custom("Unsupported type: i64"))
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
        Err(PlyError::custom("Unsupported type: u64"))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.val_writer.write_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.val_writer.write_f64(v)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Unsupported type: char"))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Unsupported type: str"))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Unsupported type: bytes"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Unsupported type: none"))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Unsupported type: unit"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Unsupported type: unit_struct"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::custom("Unsupported type: unit_variant"))
    }

    fn serialize_newtype_struct<T>(
        mut self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        // Check if this is a ListCount wrapper type
        self.count_type = match name {
            "ListCountU8" => ScalarType::U8,
            "ListCountU16" => ScalarType::U16,
            "ListCountU32" => ScalarType::U32,
            _ => ScalarType::U8,
        };
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
        Err(PlyError::custom("Unsupported type: newtype_variant"))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let count =
            len.ok_or_else(|| PlyError::custom("Unsupported type: sequence without known length"))?;

        // Check if count fits in the specified count type
        let max_count = match self.count_type {
            ScalarType::I8 => i8::MAX as usize,
            ScalarType::U8 => u8::MAX as usize,
            ScalarType::I16 => i16::MAX as usize,
            ScalarType::U16 => u16::MAX as usize,
            ScalarType::I32 => i32::MAX as usize,
            ScalarType::U32 => u32::MAX as usize,
            ScalarType::F32 => u32::MAX as usize,
            ScalarType::F64 => u32::MAX as usize,
        };

        if count > max_count {
            return Err(PlyError::custom(format!(
                "List length {} exceeds maximum for {:?} count type ({})",
                count, self.count_type, max_count
            )));
        }

        // Write the count
        match self.count_type {
            ScalarType::I8 => self.val_writer.write_i8(count as i8)?,
            ScalarType::U8 => self.val_writer.write_u8(count as u8)?,
            ScalarType::I16 => self.val_writer.write_i16(count as i16)?,
            ScalarType::U16 => self.val_writer.write_u16(count as u16)?,
            ScalarType::I32 => self.val_writer.write_i32(count as i32)?,
            ScalarType::U32 => self.val_writer.write_u32(count as u32)?,
            ScalarType::F32 => self.val_writer.write_f32(count as f32)?,
            ScalarType::F64 => self.val_writer.write_f64(count as f64)?,
        }

        Ok(ListValuesSerializer {
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

pub struct ListValuesSerializer<'a, W: ScalarWriter> {
    val_writer: &'a mut W,
}

impl<W: ScalarWriter> SerializeSeq for ListValuesSerializer<'_, W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(PropertySerializer {
            val_writer: self.val_writer,
            count_type: ScalarType::U8,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
