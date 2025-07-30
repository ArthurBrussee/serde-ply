//! PLY serializer implementation using Serde's custom data format.
//!
//! This module provides a serializer that can write PLY files by generating
//! the appropriate header and serializing data according to the PLY format.
//! Supports both ASCII and binary formats.

use crate::{PlyError, PlyFormat, PlyHeader, PropertyType, ScalarType};
use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use std::io::Write;

/// PLY serializer that writes data in PLY format
pub struct PlySerializer<W> {
    writer: W,
    header: Option<PlyHeader>,
    format: PlyFormat,
    element_count: usize,
}

impl<W: Write> PlySerializer<W> {
    /// Create a new PLY serializer
    pub fn new(writer: W, format: PlyFormat) -> Self {
        Self {
            writer,
            header: None,
            format,
            element_count: 0,
        }
    }

    /// Create a PLY serializer with a pre-defined header
    pub fn with_header(writer: W, header: PlyHeader) -> Self {
        let format = header.format.clone();
        Self {
            writer,
            header: Some(header),
            format,
            element_count: 0,
        }
    }

    /// Set the header for this serializer
    pub fn set_header(&mut self, header: PlyHeader) {
        self.format = header.format.clone();
        self.header = Some(header);
    }

    /// Write the PLY header to the output
    fn write_header(&mut self) -> Result<(), PlyError> {
        if let Some(ref header) = self.header {
            writeln!(self.writer, "ply")?;
            writeln!(self.writer, "format {} {}", header.format, header.version)?;

            // Write comments
            for comment in &header.comments {
                writeln!(self.writer, "comment {comment}")?;
            }

            // Write obj_info
            for obj_info in &header.obj_info {
                writeln!(self.writer, "obj_info {obj_info}")?;
            }

            // Write elements
            for element in &header.elements {
                writeln!(self.writer, "element {} {}", element.name, element.count)?;

                for property in &element.properties {
                    match property {
                        PropertyType::Scalar { data_type, name } => {
                            writeln!(
                                self.writer,
                                "property {} {}",
                                scalar_type_to_string(data_type),
                                name
                            )?;
                        }
                        PropertyType::List {
                            count_type,
                            data_type,
                            name,
                        } => {
                            writeln!(
                                self.writer,
                                "property list {} {} {}",
                                scalar_type_to_string(count_type),
                                scalar_type_to_string(data_type),
                                name
                            )?;
                        }
                    }
                }
            }

            writeln!(self.writer, "end_header")?;
        }
        Ok(())
    }

    /// Write a scalar value in the appropriate format
    fn write_scalar(&mut self, value: &PlyScalarValue) -> Result<(), PlyError> {
        match self.format {
            PlyFormat::Ascii => {
                write!(self.writer, "{}", value)?;
                Ok(())
            }
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                self.write_binary_scalar(value)
            }
        }
    }

    /// Write a scalar value in binary format with proper endianness
    fn write_binary_scalar(&mut self, value: &PlyScalarValue) -> Result<(), PlyError> {
        match value {
            PlyScalarValue::Char(v) => {
                self.writer.write_all(&[*v as u8])?;
            }
            PlyScalarValue::UChar(v) => {
                self.writer.write_all(&[*v])?;
            }
            PlyScalarValue::Short(v) => {
                let bytes = match self.format {
                    PlyFormat::BinaryLittleEndian => v.to_le_bytes(),
                    PlyFormat::BinaryBigEndian => v.to_be_bytes(),
                    _ => unreachable!(),
                };
                self.writer.write_all(&bytes)?;
            }
            PlyScalarValue::UShort(v) => {
                let bytes = match self.format {
                    PlyFormat::BinaryLittleEndian => v.to_le_bytes(),
                    PlyFormat::BinaryBigEndian => v.to_be_bytes(),
                    _ => unreachable!(),
                };
                self.writer.write_all(&bytes)?;
            }
            PlyScalarValue::Int(v) => {
                let bytes = match self.format {
                    PlyFormat::BinaryLittleEndian => v.to_le_bytes(),
                    PlyFormat::BinaryBigEndian => v.to_be_bytes(),
                    _ => unreachable!(),
                };
                self.writer.write_all(&bytes)?;
            }
            PlyScalarValue::UInt(v) => {
                let bytes = match self.format {
                    PlyFormat::BinaryLittleEndian => v.to_le_bytes(),
                    PlyFormat::BinaryBigEndian => v.to_be_bytes(),
                    _ => unreachable!(),
                };
                self.writer.write_all(&bytes)?;
            }
            PlyScalarValue::Float(v) => {
                let bytes = match self.format {
                    PlyFormat::BinaryLittleEndian => v.to_le_bytes(),
                    PlyFormat::BinaryBigEndian => v.to_be_bytes(),
                    _ => unreachable!(),
                };
                self.writer.write_all(&bytes)?;
            }
            PlyScalarValue::Double(v) => {
                let bytes = match self.format {
                    PlyFormat::BinaryLittleEndian => v.to_le_bytes(),
                    PlyFormat::BinaryBigEndian => v.to_be_bytes(),
                    _ => unreachable!(),
                };
                self.writer.write_all(&bytes)?;
            }
        }
        Ok(())
    }
}

/// Represents a PLY scalar value for serialization
#[derive(Debug, Clone, PartialEq)]
pub enum PlyScalarValue {
    Char(i8),
    UChar(u8),
    Short(i16),
    UShort(u16),
    Int(i32),
    UInt(u32),
    Float(f32),
    Double(f64),
}

impl std::fmt::Display for PlyScalarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlyScalarValue::Char(v) => write!(f, "{v}"),
            PlyScalarValue::UChar(v) => write!(f, "{v}"),
            PlyScalarValue::Short(v) => write!(f, "{v}"),
            PlyScalarValue::UShort(v) => write!(f, "{v}"),
            PlyScalarValue::Int(v) => write!(f, "{v}"),
            PlyScalarValue::UInt(v) => write!(f, "{v}"),
            PlyScalarValue::Float(v) => write!(f, "{v}"),
            PlyScalarValue::Double(v) => write!(f, "{v}"),
        }
    }
}

/// Convert ScalarType to string representation
fn scalar_type_to_string(scalar_type: &ScalarType) -> &'static str {
    match scalar_type {
        ScalarType::Char => "char",
        ScalarType::UChar => "uchar",
        ScalarType::Short => "short",
        ScalarType::UShort => "ushort",
        ScalarType::Int => "int",
        ScalarType::UInt => "uint",
        ScalarType::Float => "float",
        ScalarType::Double => "double",
    }
}

impl<W: Write> Serializer for &mut PlySerializer<W> {
    type Ok = ();
    type Error = PlyError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&PlyScalarValue::UChar(if v { 1 } else { 0 }))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&PlyScalarValue::Char(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&PlyScalarValue::Short(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&PlyScalarValue::Int(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        // PLY doesn't have 64-bit integers, so we'll use 32-bit
        self.write_scalar(&PlyScalarValue::Int(v as i32))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&PlyScalarValue::UChar(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&PlyScalarValue::UShort(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&PlyScalarValue::UInt(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        // PLY doesn't have 64-bit integers, so we'll use 32-bit
        self.write_scalar(&PlyScalarValue::UInt(v as u32))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&PlyScalarValue::Float(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&PlyScalarValue::Double(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&PlyScalarValue::UChar(v as u8))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{v}")?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        // Serialize bytes as a sequence
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            ser::SerializeSeq::serialize_element(&mut seq, byte)?;
        }
        ser::SerializeSeq::end(seq)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(PlyError::UnsupportedFormat(
            "None values not supported in PLY format".to_string(),
        ))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
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
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        // For PLY lists, we need to write the count first
        if let Some(len) = len {
            match self.format {
                PlyFormat::Ascii => {
                    write!(self.writer, "{} ", len)?;
                }
                PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                    // Write count as uchar for lists (standard PLY convention)
                    self.write_binary_scalar(&PlyScalarValue::UChar(len as u8))?;
                }
            }
        }
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        // Write header if not already written
        if self.header.is_some() && self.element_count == 0 {
            self.write_header()?;
        }
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(self)
    }
}

impl<'a, W: Write> SerializeSeq for &'a mut PlySerializer<W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self.format {
            PlyFormat::Ascii => {
                write!(self.writer, " ")?;
            }
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                // No separator needed for binary format
            }
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: Write> SerializeTuple for &mut PlySerializer<W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        match self.format {
            PlyFormat::Ascii => {
                write!(self.writer, " ")?;
            }
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                // No separator needed for binary format
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: Write> SerializeTupleStruct for &mut PlySerializer<W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        match self.format {
            PlyFormat::Ascii => {
                write!(self.writer, " ")?;
            }
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                // No separator needed for binary format
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.format {
            PlyFormat::Ascii => {
                writeln!(self.writer)?;
            }
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                // No newline needed for binary format
            }
        }
        Ok(())
    }
}

impl<W: Write> SerializeTupleVariant for &mut PlySerializer<W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        match self.format {
            PlyFormat::Ascii => {
                write!(self.writer, " ")?;
            }
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                // No separator needed for binary format
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.format {
            PlyFormat::Ascii => {
                writeln!(self.writer)?;
            }
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                // No newline needed for binary format
            }
        }
        Ok(())
    }
}

impl<W: Write> SerializeMap for &mut PlySerializer<W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // In PLY context, the key is typically an element name
        // We'll store it for later use
        // For now, we'll just serialize it as a string
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // The value should be a sequence of elements
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W: Write> SerializeStruct for &mut PlySerializer<W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        match self.format {
            PlyFormat::Ascii => {
                write!(self.writer, " ")?;
            }
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                // No separator needed for binary format
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.format {
            PlyFormat::Ascii => {
                writeln!(self.writer)?;
            }
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                // No newline needed for binary format
            }
        }
        Ok(())
    }
}

impl<W: Write> SerializeStructVariant for &mut PlySerializer<W> {
    type Ok = ();
    type Error = PlyError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        match self.format {
            PlyFormat::Ascii => {
                write!(self.writer, " ")?;
            }
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                // No separator needed for binary format
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.format {
            PlyFormat::Ascii => {
                writeln!(self.writer)?;
            }
            PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                // No newline needed for binary format
            }
        }
        Ok(())
    }
}

/// Convenience function to serialize a value to PLY format
pub fn to_writer<W, T>(writer: W, header: &PlyHeader, value: &T) -> Result<(), PlyError>
where
    W: Write,
    T: Serialize,
{
    let mut serializer = PlySerializer::with_header(writer, header.clone());
    value.serialize(&mut serializer)
}

/// Convenience function to serialize a value to a PLY string
pub fn to_string<T>(header: &PlyHeader, value: &T) -> Result<String, PlyError>
where
    T: Serialize,
{
    let mut buffer = Vec::new();
    to_writer(&mut buffer, header, value)?;
    String::from_utf8(buffer).map_err(|e| PlyError::Serde(format!("UTF-8 encoding error: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ElementDef, PlyFormat, PlyHeader, PropertyType, ScalarType};
    use serde::Serialize;

    #[derive(Serialize)]
    struct Vertex {
        x: f32,
        y: f32,
        z: f32,
    }

    #[test]
    fn test_serialize_scalar() {
        let mut buffer = Vec::new();
        let mut serializer = PlySerializer::new(&mut buffer, PlyFormat::Ascii);

        42i32.serialize(&mut serializer).unwrap();
        assert_eq!(String::from_utf8(buffer).unwrap(), "42");
    }

    #[test]
    fn test_serialize_vertex() {
        let vertex = Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        let mut buffer = Vec::new();
        let mut serializer = PlySerializer::new(&mut buffer, PlyFormat::Ascii);

        vertex.serialize(&mut serializer).unwrap();
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result.trim(), "1 2 3");
    }

    #[test]
    fn test_header_serialization() {
        let header = PlyHeader {
            format: PlyFormat::Ascii,
            version: "1.0".to_string(),
            elements: vec![ElementDef {
                name: "vertex".to_string(),
                count: 3,
                properties: vec![
                    PropertyType::Scalar {
                        data_type: ScalarType::Float,
                        name: "x".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::Float,
                        name: "y".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::Float,
                        name: "z".to_string(),
                    },
                ],
            }],
            comments: vec!["Test PLY file".to_string()],
            obj_info: vec![],
        };

        let mut buffer = Vec::new();
        let mut serializer = PlySerializer::with_header(&mut buffer, header);
        serializer.write_header().unwrap();

        let result = String::from_utf8(buffer).unwrap();
        assert!(result.contains("ply"));
        assert!(result.contains("format ascii 1.0"));
        assert!(result.contains("comment Test PLY file"));
        assert!(result.contains("element vertex 3"));
        assert!(result.contains("property float x"));
        assert!(result.contains("end_header"));
    }
}
