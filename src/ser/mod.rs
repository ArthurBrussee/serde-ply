use std::io::Write;

use serde::{Serialize, Serializer};

use crate::{
    ser::{header_collector::HeaderCollector, ply_file::PlyFileSerializer},
    PlyError, PlyFormat,
};

// mod ply_file;
mod header_collector;
mod ply_file;
mod row;

pub mod val_writer;

// Helper function to extract string from serde key
pub(crate) fn extract_string_key<T: Serialize + ?Sized>(key: &T) -> Result<String, PlyError> {
    struct StringExtractor(String);

    impl Serializer for &mut StringExtractor {
        type Ok = ();
        type Error = PlyError;
        type SerializeSeq = serde::ser::Impossible<(), PlyError>;
        type SerializeTuple = serde::ser::Impossible<(), PlyError>;
        type SerializeTupleStruct = serde::ser::Impossible<(), PlyError>;
        type SerializeTupleVariant = serde::ser::Impossible<(), PlyError>;
        type SerializeMap = serde::ser::Impossible<(), PlyError>;
        type SerializeStruct = serde::ser::Impossible<(), PlyError>;
        type SerializeStructVariant = serde::ser::Impossible<(), PlyError>;

        fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
            self.0 = v.to_string();
            Ok(())
        }

        fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
        where
            T: Serialize + ?Sized,
        {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_newtype_struct<T>(
            self,
            _name: &'static str,
            _value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: Serialize + ?Sized,
        {
            Err(serde::ser::Error::custom("keys must be strings"))
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
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }

        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            Err(serde::ser::Error::custom("keys must be strings"))
        }
    }

    let mut extractor = StringExtractor(String::new());
    key.serialize(&mut extractor)?;
    Ok(extractor.0)
}

pub fn to_writer<T>(val: &T, format: PlyFormat, mut writer: impl Write) -> Result<(), PlyError>
where
    T: Serialize,
{
    val.serialize(&mut HeaderCollector::new(format, &mut writer))?;
    val.serialize(&mut PlyFileSerializer::new(format, &mut writer))?;
    Ok(())
}

/// Serializes
pub fn to_bytes<T>(val: &T, format: PlyFormat) -> Result<Vec<u8>, PlyError>
where
    T: Serialize,
{
    let mut buf = vec![];
    val.serialize(&mut HeaderCollector::new(format, &mut buf))?;
    val.serialize(&mut PlyFileSerializer::new(format, &mut buf))?;
    Ok(buf)
}
