//! A Serde-based PLY (Polygon File Format) serializer and deserializer.
//!
//! This crate provides a custom Serde data format for reading and writing PLY files.
//! PLY files have a variable header structure that defines the data format, so we
//! parse the header first and use that information to guide deserialization.
//!
//! # Example
//!
//! ```rust
//! use serde::Deserialize;
//! use serde_ply::PlyHeader;
//!
//! #[derive(Deserialize, Debug)]
//! struct Vertex {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//! }
//!
//! let ply_data = r#"ply
//! format ascii 1.0
//! element vertex 1
//! property float x
//! property float y
//! property float z
//! end_header
//! 1.0 2.0 3.0
//! "#;
//!
//! // Parse header to inspect structure
//! let (header, _) = PlyHeader::parse(ply_data.as_bytes()).unwrap();
//! println!("Found {} vertices", header.get_element("vertex").unwrap().count);
//!
//! // Deserialize directly to structs
//! let vertices: Vec<Vertex> = serde_ply::from_str(ply_data, "vertex").unwrap();
//! println!("First vertex: {:?}", vertices[0]);
//! ```

pub mod de;
pub mod ser;

pub use ser::{to_string, to_writer, PlySerializer};

// Element deserialization
pub use de::ElementDeserializer;

use std::fmt;
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;
use thiserror::Error;

/// Errors that can occur during PLY parsing or serialization
#[derive(Error, Debug)]
pub enum PlyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid PLY header: {0}")]
    InvalidHeader(String),

    #[error("Unsupported PLY format: {0}")]
    UnsupportedFormat(String),

    #[error("Property type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Missing required element: {0}")]
    MissingElement(String),

    #[error("Serde error: {0}")]
    Serde(String),
}

impl serde::de::Error for PlyError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        PlyError::Serde(msg.to_string())
    }
}

impl serde::ser::Error for PlyError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        PlyError::Serde(msg.to_string())
    }
}

/// PLY file format (ascii or binary)
#[derive(Debug, Clone, PartialEq)]
pub enum PlyFormat {
    Ascii,
    BinaryLittleEndian,
    BinaryBigEndian,
}

impl fmt::Display for PlyFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlyFormat::Ascii => write!(f, "ascii"),
            PlyFormat::BinaryLittleEndian => write!(f, "binary_little_endian"),
            PlyFormat::BinaryBigEndian => write!(f, "binary_big_endian"),
        }
    }
}

/// PLY scalar data types
#[derive(Debug, Clone, PartialEq)]
pub enum ScalarType {
    Char,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Float,
    Double,
}

impl ScalarType {
    pub fn parse(s: &str) -> Result<Self, PlyError> {
        match s {
            "char" | "int8" => Ok(ScalarType::Char),
            "uchar" | "uint8" => Ok(ScalarType::UChar),
            "short" | "int16" => Ok(ScalarType::Short),
            "ushort" | "uint16" => Ok(ScalarType::UShort),
            "int" | "int32" => Ok(ScalarType::Int),
            "uint" | "uint32" => Ok(ScalarType::UInt),
            "float" | "float32" => Ok(ScalarType::Float),
            "double" | "float64" => Ok(ScalarType::Double),
            _ => Err(PlyError::UnsupportedFormat(format!(
                "Unknown scalar type: {s}"
            ))),
        }
    }

    pub fn size_bytes(&self) -> usize {
        match self {
            ScalarType::Char | ScalarType::UChar => 1,
            ScalarType::Short | ScalarType::UShort => 2,
            ScalarType::Int | ScalarType::UInt | ScalarType::Float => 4,
            ScalarType::Double => 8,
        }
    }
}

impl FromStr for ScalarType {
    type Err = PlyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// PLY property definition
#[derive(Debug, Clone)]
pub enum PropertyType {
    /// A scalar property with a single value
    Scalar { data_type: ScalarType, name: String },
    /// A list property with variable length
    List {
        count_type: ScalarType,
        data_type: ScalarType,
        name: String,
    },
}

/// PLY element definition (e.g., vertex, face)
#[derive(Debug, Clone)]
pub struct ElementDef {
    pub name: String,
    pub count: usize,
    pub properties: Vec<PropertyType>,
}

/// PLY header containing format information and element definitions
#[derive(Debug, Clone)]
pub struct PlyHeader {
    pub format: PlyFormat,
    pub version: String,
    pub elements: Vec<ElementDef>,
    pub comments: Vec<String>,
    pub obj_info: Vec<String>,
}

impl PlyHeader {
    /// Parse a PLY header from a reader
    pub fn parse<R: Read>(reader: R) -> Result<(Self, usize), PlyError> {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        let mut bytes_read = 0;

        // Read the first line - should be "ply"
        bytes_read += buf_reader.read_line(&mut line)?;
        if line.trim() != "ply" {
            return Err(PlyError::InvalidHeader(
                "File must start with 'ply'".to_string(),
            ));
        }

        let mut format = None;
        let mut version = String::new();
        let mut elements = Vec::new();
        let mut comments = Vec::new();
        let mut obj_info = Vec::new();
        let mut current_element: Option<ElementDef> = None;

        loop {
            line.clear();
            let line_bytes = buf_reader.read_line(&mut line)?;
            if line_bytes == 0 {
                return Err(PlyError::InvalidHeader(
                    "Unexpected end of file".to_string(),
                ));
            }
            bytes_read += line_bytes;

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line == "end_header" {
                break;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "format" => {
                    if parts.len() < 3 {
                        return Err(PlyError::InvalidHeader("Invalid format line".to_string()));
                    }
                    format = Some(match parts[1] {
                        "ascii" => PlyFormat::Ascii,
                        "binary_little_endian" => PlyFormat::BinaryLittleEndian,
                        "binary_big_endian" => PlyFormat::BinaryBigEndian,
                        _ => return Err(PlyError::UnsupportedFormat(parts[1].to_string())),
                    });
                    version = parts[2].to_string();
                }
                "comment" => {
                    comments.push(parts[1..].join(" "));
                }
                "obj_info" => {
                    obj_info.push(parts[1..].join(" "));
                }
                "element" => {
                    if parts.len() < 3 {
                        return Err(PlyError::InvalidHeader("Invalid element line".to_string()));
                    }

                    // Save previous element if any
                    if let Some(element) = current_element.take() {
                        elements.push(element);
                    }

                    let name = parts[1].to_string();
                    let count = parts[2].parse::<usize>().map_err(|_| {
                        PlyError::InvalidHeader(format!("Invalid element count: {}", parts[2]))
                    })?;

                    current_element = Some(ElementDef {
                        name,
                        count,
                        properties: Vec::new(),
                    });
                }
                "property" => {
                    let element = current_element.as_mut().ok_or_else(|| {
                        PlyError::InvalidHeader("Property without element".to_string())
                    })?;

                    if parts.len() < 3 {
                        return Err(PlyError::InvalidHeader("Invalid property line".to_string()));
                    }

                    if parts[1] == "list" {
                        // List property: property list <count_type> <data_type> <name>
                        if parts.len() < 5 {
                            return Err(PlyError::InvalidHeader(
                                "Invalid list property line".to_string(),
                            ));
                        }
                        let count_type = ScalarType::parse(parts[2])?;
                        let data_type = ScalarType::parse(parts[3])?;
                        let name = parts[4].to_string();

                        element.properties.push(PropertyType::List {
                            count_type,
                            data_type,
                            name,
                        });
                    } else {
                        // Scalar property: property <type> <name>
                        let data_type = ScalarType::parse(parts[1])?;
                        let name = parts[2].to_string();

                        element
                            .properties
                            .push(PropertyType::Scalar { data_type, name });
                    }
                }
                _ => {
                    // Unknown header line - could be a comment or extension
                    comments.push(line.to_string());
                }
            }
        }

        // Save the last element
        if let Some(element) = current_element {
            elements.push(element);
        }

        let format = format
            .ok_or_else(|| PlyError::InvalidHeader("Missing format specification".to_string()))?;

        Ok((
            PlyHeader {
                format,
                version,
                elements,
                comments,
                obj_info,
            },
            bytes_read,
        ))
    }

    /// Get element definition by name
    pub fn get_element(&self, name: &str) -> Option<&ElementDef> {
        self.elements.iter().find(|e| e.name == name)
    }

    /// Check if this header defines an element with the given name
    pub fn has_element(&self, name: &str) -> bool {
        self.elements.iter().any(|e| e.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_simple_header() {
        let header_text = r#"ply
format ascii 1.0
comment A simple PLY file
element vertex 3
property float x
property float y
property float z
element face 1
property list uchar int vertex_indices
end_header
"#;

        let cursor = Cursor::new(header_text);
        let (header, _) = PlyHeader::parse(cursor).unwrap();

        assert_eq!(header.format, PlyFormat::Ascii);
        assert_eq!(header.version, "1.0");
        assert_eq!(header.elements.len(), 2);
        assert_eq!(header.comments.len(), 1);

        let vertex_element = header.get_element("vertex").unwrap();
        assert_eq!(vertex_element.count, 3);
        assert_eq!(vertex_element.properties.len(), 3);

        let face_element = header.get_element("face").unwrap();
        assert_eq!(face_element.count, 1);
        assert_eq!(face_element.properties.len(), 1);
    }

    #[test]
    fn test_scalar_type_parsing() {
        assert_eq!(ScalarType::parse("float").unwrap(), ScalarType::Float);
        assert_eq!(ScalarType::parse("float32").unwrap(), ScalarType::Float);
        assert_eq!(ScalarType::parse("double").unwrap(), ScalarType::Double);
        assert_eq!(ScalarType::parse("int").unwrap(), ScalarType::Int);
        assert_eq!(ScalarType::parse("uchar").unwrap(), ScalarType::UChar);

        assert!(ScalarType::parse("invalid_type").is_err());
    }
}

/// Parse header and find data start position
fn parse_header_and_data(mut reader: impl Read) -> Result<(PlyHeader, Vec<u8>), PlyError> {
    // Read all data first
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    // Find header end position - handle different line endings
    let mut header_end = 0;
    for i in 0..buffer.len().saturating_sub(12) {
        if buffer[i..].starts_with(b"end_header\r\n") {
            header_end = i + 12;
            break;
        } else if buffer[i..].starts_with(b"end_header\n") {
            header_end = i + 11;
            break;
        }
    }

    if header_end == 0 {
        return Err(PlyError::InvalidHeader("No end_header found".to_string()));
    }

    // Parse header from UTF-8
    let header_bytes = &buffer[..header_end];
    let header_string = String::from_utf8(header_bytes.to_vec())
        .map_err(|e| PlyError::Serde(format!("Invalid UTF-8 in header: {}", e)))?;

    let cursor = std::io::Cursor::new(&header_string);
    let (header, _) = PlyHeader::parse(cursor)?;

    // Get data portion
    let data_portion = buffer[header_end..].to_vec();
    Ok((header, data_portion))
}

/// Deserialize elements from a reader
pub fn from_reader<R, T>(reader: R, element_name: &str) -> Result<Vec<T>, PlyError>
where
    R: Read,
    T: for<'de> serde::Deserialize<'de>,
{
    let (header, data) = parse_header_and_data(reader)?;
    let data_reader = std::io::Cursor::new(data);

    let mut deserializer = ElementDeserializer::new(data_reader, &header, element_name)?;
    let mut results = Vec::new();

    while let Some(element) = deserializer.next_element::<T>()? {
        results.push(element);
    }

    Ok(results)
}

/// Convenience function for deserializing from a string
pub fn from_str<T>(ply_str: &str, element_name: &str) -> Result<Vec<T>, PlyError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    from_reader(std::io::Cursor::new(ply_str), element_name)
}

/// Deserialize elements with explicit header (useful when header is already parsed)
pub fn deserialize_elements<R, T>(
    reader: R,
    header: &PlyHeader,
    element_name: &str,
) -> Result<Vec<T>, PlyError>
where
    R: Read,
    T: for<'de> serde::Deserialize<'de>,
{
    let mut deserializer = ElementDeserializer::new(reader, header, element_name)?;
    let mut results = Vec::new();

    while let Some(element) = deserializer.next_element::<T>()? {
        results.push(element);
    }

    Ok(results)
}
