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
    current_field: usize,
}

impl<W: Write> PlySerializer<W> {
    /// Create a new PLY serializer
    pub fn new(writer: W, format: PlyFormat) -> Self {
        Self {
            writer,
            header: None,
            format,
            current_field: 0,
        }
    }

    /// Create a PLY serializer with a pre-defined header
    pub fn with_header(writer: W, header: PlyHeader) -> Self {
        let format = header.format.clone();
        Self {
            writer,
            header: Some(header),
            format,
            current_field: 0,
        }
    }

    /// Serialize elements one by one for proper PLY format
    pub fn serialize_elements<T>(&mut self, elements: &[T]) -> Result<(), PlyError>
    where
        T: Serialize,
    {
        // Write header if not already written
        self.write_header()?;

        for element in elements {
            self.current_field = 0;
            element.serialize(&mut *self)?;
            match self.format {
                PlyFormat::Ascii => {
                    writeln!(self.writer)?;
                }
                PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                    // No newline for binary format
                }
            }
        }
        Ok(())
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
                write!(self.writer, "{value}")?;
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
                    write!(self.writer, "{len} ")?;
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

impl<W: Write> SerializeSeq for &mut PlySerializer<W> {
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
        // Add space before field (except for first field)
        if self.current_field > 0 {
            match self.format {
                PlyFormat::Ascii => {
                    write!(self.writer, " ")?;
                }
                PlyFormat::BinaryLittleEndian | PlyFormat::BinaryBigEndian => {
                    // No space separators in binary format
                }
            }
        }
        value.serialize(&mut **self)?;
        self.current_field += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // No newline here - elements handle their own line separation
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
    serializer.write_header()?;
    value.serialize(&mut serializer)
}

/// Convenience function to serialize elements to PLY format properly
pub fn elements_to_writer<W, T>(
    writer: W,
    header: &PlyHeader,
    elements: &[T],
) -> Result<(), PlyError>
where
    W: Write,
    T: Serialize,
{
    let mut serializer = PlySerializer::with_header(writer, header.clone());
    serializer.serialize_elements(elements)
}

/// Convenience function to serialize elements to PLY bytes
pub fn elements_to_bytes<T>(header: &PlyHeader, elements: &[T]) -> Result<Vec<u8>, PlyError>
where
    T: Serialize,
{
    let mut buffer = Vec::new();
    elements_to_writer(&mut buffer, header, elements)?;
    Ok(buffer)
}

/// Convenience function to serialize a value to a PLY string (ASCII format only)
pub fn to_string<T>(header: &PlyHeader, value: &T) -> Result<String, PlyError>
where
    T: Serialize,
{
    if !matches!(header.format, PlyFormat::Ascii) {
        return Err(PlyError::UnsupportedFormat(
            "to_string only supports ASCII format - use to_bytes for binary formats".to_string(),
        ));
    }

    let mut buffer = Vec::new();
    to_writer(&mut buffer, header, value)?;
    String::from_utf8(buffer).map_err(|e| PlyError::Serde(format!("UTF-8 encoding error: {e}")))
}

/// Convenience function to serialize a value to PLY bytes (works for all formats)
pub fn to_bytes<T>(header: &PlyHeader, value: &T) -> Result<Vec<u8>, PlyError>
where
    T: Serialize,
{
    let mut buffer = Vec::new();
    to_writer(&mut buffer, header, value)?;
    Ok(buffer)
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

    #[derive(Serialize)]
    struct VertexWithColor {
        x: f32,
        y: f32,
        z: f32,
        red: u8,
        green: u8,
        blue: u8,
    }

    #[derive(Serialize)]
    struct Face {
        vertex_indices: Vec<u32>,
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

    #[test]
    fn test_binary_scalar_serialization() {
        // Test f32
        let mut buffer = Vec::new();
        let mut serializer = PlySerializer::new(&mut buffer, PlyFormat::BinaryLittleEndian);
        1.5f32.serialize(&mut serializer).unwrap();
        assert_eq!(buffer, 1.5f32.to_le_bytes());

        // Test u32
        let mut buffer = Vec::new();
        let mut serializer = PlySerializer::new(&mut buffer, PlyFormat::BinaryLittleEndian);
        42u32.serialize(&mut serializer).unwrap();
        assert_eq!(buffer, 42u32.to_le_bytes());

        // Test u8
        let mut buffer = Vec::new();
        let mut serializer = PlySerializer::new(&mut buffer, PlyFormat::BinaryLittleEndian);
        255u8.serialize(&mut serializer).unwrap();
        assert_eq!(buffer, [255]);
    }

    #[test]
    fn test_binary_vertex_serialization() {
        let vertex = VertexWithColor {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            red: 255,
            green: 128,
            blue: 0,
        };

        let mut buffer = Vec::new();
        let mut serializer = PlySerializer::new(&mut buffer, PlyFormat::BinaryLittleEndian);

        vertex.serialize(&mut serializer).unwrap();

        let expected = [
            1.0f32.to_le_bytes().as_ref(),
            2.0f32.to_le_bytes().as_ref(),
            3.0f32.to_le_bytes().as_ref(),
            &[255u8],
            &[128u8],
            &[0u8],
        ]
        .concat();

        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_binary_list_serialization() {
        let face = Face {
            vertex_indices: vec![0, 1, 2],
        };

        let mut buffer = Vec::new();
        let mut serializer = PlySerializer::new(&mut buffer, PlyFormat::BinaryLittleEndian);

        face.serialize(&mut serializer).unwrap();

        let expected = [
            &[3u8], // count
            0u32.to_le_bytes().as_ref(),
            1u32.to_le_bytes().as_ref(),
            2u32.to_le_bytes().as_ref(),
        ]
        .concat();

        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_complete_binary_ply_file() {
        let vertices = vec![
            VertexWithColor {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                red: 255,
                green: 0,
                blue: 0,
            },
            VertexWithColor {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                red: 0,
                green: 255,
                blue: 0,
            },
            VertexWithColor {
                x: 0.5,
                y: 1.0,
                z: 0.0,
                red: 0,
                green: 0,
                blue: 255,
            },
        ];

        let header = PlyHeader {
            format: PlyFormat::BinaryLittleEndian,
            version: "1.0".to_string(),
            elements: vec![ElementDef {
                name: "vertex".to_string(),
                count: vertices.len(),
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
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "red".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "green".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "blue".to_string(),
                    },
                ],
            }],
            comments: vec![],
            obj_info: vec![],
        };

        let result = to_bytes(&header, &vertices);

        // This should work now with proper binary support
        match result {
            Ok(ply_bytes) => {
                println!("Binary PLY serialization succeeded");
                // Convert just the header portion to string to verify
                let header_str = String::from_utf8_lossy(&ply_bytes[..200]); // First 200 bytes should be header
                assert!(header_str.contains("format binary_little_endian 1.0"));
                assert!(header_str.contains("end_header"));

                // Verify we have the expected total size
                // Header + 3 vertices * (3 floats + 3 bytes) = header + 3 * 15 = header + 45 bytes
                assert!(ply_bytes.len() > 45);
            }
            Err(e) => {
                panic!("Binary serialization failed: {e}");
            }
        }
    }

    #[test]
    fn test_ascii_still_works_with_to_string() {
        let vertices = vec![
            VertexWithColor {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                red: 255,
                green: 0,
                blue: 0,
            },
            VertexWithColor {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                red: 0,
                green: 255,
                blue: 0,
            },
        ];

        let header = PlyHeader {
            format: PlyFormat::Ascii,
            version: "1.0".to_string(),
            elements: vec![ElementDef {
                name: "vertex".to_string(),
                count: vertices.len(),
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
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "red".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "green".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "blue".to_string(),
                    },
                ],
            }],
            comments: vec![],
            obj_info: vec![],
        };

        let result = to_string(&header, &vertices).unwrap();

        // Verify header
        assert!(result.contains("format ascii 1.0"));
        assert!(result.contains("end_header"));

        // Verify data (should be space-separated)
        assert!(result.contains("0 0 0 255 0 0"));
        assert!(result.contains("1 0 0 0 255 0"));
    }

    #[test]
    fn test_to_string_rejects_binary_format() {
        let vertex = Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        let header = PlyHeader {
            format: PlyFormat::BinaryLittleEndian,
            version: "1.0".to_string(),
            elements: vec![],
            comments: vec![],
            obj_info: vec![],
        };

        let result = to_string(&header, &vertex);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("to_string only supports ASCII format"));
    }

    #[test]
    fn test_binary_round_trip() {
        use crate::parse_elements;
        use std::io::Cursor;

        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct RoundTripVertex {
            x: f32,
            y: f32,
            z: f32,
            red: u8,
            green: u8,
            blue: u8,
        }

        let original_vertices = vec![
            RoundTripVertex {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                red: 255,
                green: 0,
                blue: 0,
            },
            RoundTripVertex {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                red: 0,
                green: 255,
                blue: 0,
            },
            RoundTripVertex {
                x: 0.5,
                y: 1.0,
                z: 0.0,
                red: 0,
                green: 0,
                blue: 255,
            },
        ];

        let header = PlyHeader {
            format: PlyFormat::BinaryLittleEndian,
            version: "1.0".to_string(),
            elements: vec![ElementDef {
                name: "vertex".to_string(),
                count: original_vertices.len(),
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
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "red".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "green".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "blue".to_string(),
                    },
                ],
            }],
            comments: vec![],
            obj_info: vec![],
        };

        // Serialize to binary using proper elements API
        let ply_bytes = elements_to_bytes(&header, &original_vertices).unwrap();

        // Deserialize back
        let cursor = Cursor::new(ply_bytes);
        let mut reader = std::io::BufReader::new(cursor);
        let header = crate::PlyHeader::parse(&mut reader).unwrap();
        let deserialized_vertices: Vec<RoundTripVertex> =
            parse_elements(&mut reader, &header, "vertex").unwrap();

        // Verify they match
        assert_eq!(original_vertices.len(), deserialized_vertices.len());
        for (original, deserialized) in original_vertices.iter().zip(deserialized_vertices.iter()) {
            assert_eq!(original, deserialized);
        }
    }

    #[test]
    fn test_debug_binary_output() {
        let vertex = VertexWithColor {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            red: 255,
            green: 128,
            blue: 0,
        };

        let header = PlyHeader {
            format: PlyFormat::BinaryLittleEndian,
            version: "1.0".to_string(),
            elements: vec![ElementDef {
                name: "vertex".to_string(),
                count: 1,
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
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "red".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "green".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "blue".to_string(),
                    },
                ],
            }],
            comments: vec![],
            obj_info: vec![],
        };

        let ply_bytes = to_bytes(&header, &vertex).unwrap();

        // Find where header ends
        let header_end = ply_bytes
            .windows(11)
            .position(|w| w == b"end_header\n")
            .unwrap()
            + 11;

        println!("Header ends at byte {header_end}");
        println!("Total bytes: {}", ply_bytes.len());
        println!("Data portion: {:?}", &ply_bytes[header_end..]);

        // Expected data: 1.0f32, 2.0f32, 3.0f32, 255u8, 128u8, 0u8
        let expected = [
            1.0f32.to_le_bytes().as_ref(),
            2.0f32.to_le_bytes().as_ref(),
            3.0f32.to_le_bytes().as_ref(),
            &[255u8],
            &[128u8],
            &[0u8],
        ]
        .concat();

        println!("Expected data: {expected:?}");
        assert_eq!(&ply_bytes[header_end..], expected);
    }

    #[test]
    #[allow(dead_code)]
    fn test_debug_array_vs_single() {
        let single_vertex = VertexWithColor {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            red: 255,
            green: 0,
            blue: 0,
        };

        let vertex_array = vec![VertexWithColor {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            red: 255,
            green: 0,
            blue: 0,
        }];

        let header = PlyHeader {
            format: PlyFormat::BinaryLittleEndian,
            version: "1.0".to_string(),
            elements: vec![ElementDef {
                name: "vertex".to_string(),
                count: 1,
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
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "red".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "green".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "blue".to_string(),
                    },
                ],
            }],
            comments: vec![],
            obj_info: vec![],
        };

        let single_bytes = to_bytes(&header, &single_vertex).unwrap();
        let array_bytes = to_bytes(&header, &vertex_array).unwrap();

        println!("Single vertex bytes: {}", single_bytes.len());
        println!("Array bytes: {}", array_bytes.len());

        // Find data portions
        let single_header_end = single_bytes
            .windows(11)
            .position(|w| w == b"end_header\n")
            .unwrap()
            + 11;
        let array_header_end = array_bytes
            .windows(11)
            .position(|w| w == b"end_header\n")
            .unwrap()
            + 11;

        println!("Single data: {:?}", &single_bytes[single_header_end..]);
        println!("Array data: {:?}", &array_bytes[array_header_end..]);
    }

    #[test]
    fn test_simple_ascii_output() {
        let vertex = VertexWithColor {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            red: 255,
            green: 128,
            blue: 0,
        };

        let header = PlyHeader {
            format: PlyFormat::Ascii,
            version: "1.0".to_string(),
            elements: vec![ElementDef {
                name: "vertex".to_string(),
                count: 1,
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
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "red".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "green".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "blue".to_string(),
                    },
                ],
            }],
            comments: vec![],
            obj_info: vec![],
        };

        let vertices = vec![vertex];
        let ply_bytes = elements_to_bytes(&header, &vertices).unwrap();
        let ply_str = String::from_utf8(ply_bytes).unwrap();

        println!("ASCII output:\n{ply_str}");

        // Verify it contains the header
        assert!(ply_str.contains("format ascii 1.0"));
        assert!(ply_str.contains("end_header"));

        // Verify it contains our vertex data
        assert!(ply_str.contains("1 2 3 255 128 0"));
    }

    #[test]
    fn test_ascii_round_trip() {
        use crate::parse_elements;
        use std::io::Cursor;

        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct RoundTripVertex {
            x: f32,
            y: f32,
            z: f32,
            red: u8,
            green: u8,
            blue: u8,
        }

        let original_vertices = vec![
            RoundTripVertex {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                red: 255,
                green: 0,
                blue: 0,
            },
            RoundTripVertex {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                red: 0,
                green: 255,
                blue: 0,
            },
            RoundTripVertex {
                x: 0.5,
                y: 1.0,
                z: 0.0,
                red: 0,
                green: 0,
                blue: 255,
            },
        ];

        let header = PlyHeader {
            format: PlyFormat::Ascii,
            version: "1.0".to_string(),
            elements: vec![ElementDef {
                name: "vertex".to_string(),
                count: original_vertices.len(),
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
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "red".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "green".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "blue".to_string(),
                    },
                ],
            }],
            comments: vec![],
            obj_info: vec![],
        };

        // Serialize to ASCII using proper elements API
        let ply_bytes = elements_to_bytes(&header, &original_vertices).unwrap();
        let ply_str = String::from_utf8(ply_bytes).unwrap();

        // Deserialize back
        let cursor = Cursor::new(ply_str.as_bytes());
        let mut reader = std::io::BufReader::new(cursor);
        let header = crate::PlyHeader::parse(&mut reader).unwrap();
        let deserialized_vertices: Vec<RoundTripVertex> =
            parse_elements(&mut reader, &header, "vertex").unwrap();

        // Verify they match
        assert_eq!(original_vertices.len(), deserialized_vertices.len());
        for (original, deserialized) in original_vertices.iter().zip(deserialized_vertices.iter()) {
            assert_eq!(original, deserialized);
        }
    }
}
