use crate::{PlyError, PlyFormat, ScalarType};
use serde::{
    ser::{Impossible, SerializeMap, SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};
use std::io::Write;

#[derive(Copy, Clone, Eq, PartialEq)]
enum Recursion {
    Header,
    Element,
    Row,
}

impl Recursion {
    fn next(self) -> Result<Recursion, PlyError> {
        match self {
            Recursion::Header => Ok(Recursion::Element),
            Recursion::Element => Ok(Recursion::Row),
            // TODO: More specific error.
            Recursion::Row => Err(PlyError::InvalidStructure),
        }
    }
}

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

pub(crate) struct HeaderCollector<W: Write> {
    writer: W,
    format: PlyFormat,
    recursion: Recursion,
}

impl<W: Write> HeaderCollector<W> {
    pub fn new(format: PlyFormat, writer: W) -> Self {
        Self {
            writer,
            format,
            recursion: Recursion::Header,
        }
    }
}

impl<'a, W: Write> Serializer for &'a mut HeaderCollector<W> {
    type Ok = ();
    type Error = PlyError;

    type SerializeMap = HeaderMapCollector<'a, W>;
    type SerializeStruct = HeaderStructCollector<'a, W>;

    type SerializeTuple = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), PlyError>;
    type SerializeStructVariant = serde::ser::Impossible<(), PlyError>;
    type SerializeSeq = serde::ser::Impossible<(), PlyError>;

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

    fn serialize_some<T: Serialize + ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
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

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
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
        if self.recursion == Recursion::Header {
            writeln!(self.writer, "ply\nformat {}", self.format)?;
        }
        Ok(HeaderMapCollector {
            recursion: self.recursion,
            parent: self,
            cur_key: "".to_string(),
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if self.recursion == Recursion::Header {
            writeln!(self.writer, "ply\nformat {} 1.0", self.format)?;
        }

        Ok(HeaderStructCollector {
            recursion: self.recursion,
            parent: self,
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

pub struct HeaderMapCollector<'a, W: Write> {
    cur_key: String,
    parent: &'a mut HeaderCollector<W>,
    recursion: Recursion,
}

impl<W: Write> SerializeMap for HeaderMapCollector<'_, W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.cur_key = extract_string_key(key)?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(PropertyCollector {
            parent: self.parent,
            property_name: &self.cur_key,
            recursion: self.recursion.next()?,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.recursion == Recursion::Header {
            writeln!(self.parent.writer, "end_header")?;
        }
        Ok(())
    }
}

pub struct HeaderStructCollector<'a, W: Write> {
    parent: &'a mut HeaderCollector<W>,
    recursion: Recursion,
}

impl<W: Write> SerializeStruct for HeaderStructCollector<'_, W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(PropertyCollector {
            parent: self.parent,
            property_name: key,
            recursion: self.recursion.next()?,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.recursion == Recursion::Header {
            writeln!(self.parent.writer, "end_header")?;
        }
        Ok(())
    }
}

struct PropertyCollector<'a, W: Write> {
    parent: &'a mut HeaderCollector<W>,
    property_name: &'a str,
    recursion: Recursion,
}

impl<'a, W: Write> Serializer for PropertyCollector<'a, W> {
    type Ok = ();
    type Error = PlyError;
    type SerializeSeq = ListPropertyCollector<'a, W>;
    type SerializeTuple = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), PlyError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), PlyError>;
    type SerializeMap = HeaderMapCollector<'a, W>;
    type SerializeStruct = HeaderStructCollector<'a, W>;
    type SerializeStructVariant = serde::ser::Impossible<(), PlyError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("bool".to_string()))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        writeln!(self.parent.writer, "property char {}", self.property_name)?;
        Ok(())
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        writeln!(self.parent.writer, "property short {}", self.property_name)?;
        Ok(())
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        writeln!(self.parent.writer, "property int {}", self.property_name)?;
        Ok(())
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("i64".to_string()))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        writeln!(self.parent.writer, "property uchar {}", self.property_name)?;
        Ok(())
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        writeln!(self.parent.writer, "property ushort {}", self.property_name)?;
        Ok(())
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        writeln!(self.parent.writer, "property uint {}", self.property_name)?;
        Ok(())
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("u64".to_string()))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        writeln!(self.parent.writer, "property float {}", self.property_name)?;
        Ok(())
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        writeln!(self.parent.writer, "property double {}", self.property_name)?;
        Ok(())
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
        if self.recursion == Recursion::Element {
            // For elements, this is a list of rows.
            let Some(len) = len else {
                return Err(PlyError::InvalidStructure);
            };
            writeln!(self.parent.writer, "element {} {}", self.property_name, len)?;
        }

        // Now visit this list. This is needed to write the properties of the struct.
        // We really only want to visit the first one though.
        Ok(ListPropertyCollector {
            writer: &mut self.parent.writer,
            recursion: self.recursion,
            prop_name: self.property_name,
            active: true,
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
        Ok(HeaderMapCollector {
            parent: self.parent,
            cur_key: "".to_string(),
            recursion: self.recursion,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(HeaderStructCollector {
            parent: self.parent,
            recursion: self.recursion,
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

struct ListPropertyCollector<'a, W: Write> {
    writer: &'a mut W,
    prop_name: &'a str,
    recursion: Recursion,
    active: bool,
}

impl<W: Write> SerializeSeq for ListPropertyCollector<'_, W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        if self.active {
            self.active = false;

            if self.recursion == Recursion::Element {
                value.serialize(&mut HeaderCollector {
                    writer: &mut self.writer,
                    format: PlyFormat::Ascii, // unused
                    recursion: self.recursion,
                })?
            } else if self.recursion == Recursion::Row {
                value.serialize(self)?
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: Write> ListPropertyCollector<'_, W> {
    fn write_list_prop(&mut self, t: ScalarType) -> Result<(), PlyError> {
        Ok(writeln!(
            self.writer,
            "property list uchar {} {}",
            t, self.prop_name
        )?)
    }
}

// For rows, this is a list property, and we're trying to get the element type.
// For now, assume u8 count type and f32 data type for lists as it's most common,
// but we really should give users control here.
impl<W: Write> Serializer for &mut ListPropertyCollector<'_, W> {
    type Ok = ();
    type Error = PlyError;

    type SerializeSeq = Impossible<(), PlyError>;
    type SerializeTuple = Impossible<(), PlyError>;
    type SerializeTupleStruct = Impossible<(), PlyError>;
    type SerializeTupleVariant = Impossible<(), PlyError>;
    type SerializeMap = Impossible<(), PlyError>;
    type SerializeStruct = Impossible<(), PlyError>;
    type SerializeStructVariant = Impossible<(), PlyError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::InvalidStructure)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        self.write_list_prop(ScalarType::I8)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        self.write_list_prop(ScalarType::I16)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        self.write_list_prop(ScalarType::I32)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("i64".to_string()))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        self.write_list_prop(ScalarType::U8)
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        self.write_list_prop(ScalarType::U16)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        self.write_list_prop(ScalarType::U32)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("u64".to_string()))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        self.write_list_prop(ScalarType::F32)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedType("f64".to_string()))
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
        Err(PlyError::UnsupportedType("option".to_string()))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(PlyError::UnsupportedType("option".to_string()))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Default, Serialize)]
    struct TestVertex {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Serialize)]
    struct TestPly {
        vertices: Vec<TestVertex>,
    }

    #[test]
    fn test_header_collector() {
        let vertex = TestPly {
            vertices: vec![TestVertex::default(), TestVertex::default()],
        };

        let mut output = Vec::new();
        vertex
            .serialize(&mut HeaderCollector::new(PlyFormat::Ascii, &mut output))
            .unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(
            result,
            r"ply
format ascii 1.0
element vertices 2
property float x
property float y
property float z
end_header
"
        );
    }
}
