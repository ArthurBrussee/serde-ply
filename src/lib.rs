mod de;
mod ply_file;
mod ser;

pub use ser::*;

use byteorder::{BigEndian, LittleEndian};
pub use ply_file::PlyFile;

use std::io::{BufRead, Read};
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;
use std::{fmt, string::FromUtf8Error};
use thiserror::Error;

use crate::de::val_reader::{AsciiValReader, BinValReader};
use crate::de::RowDeserializer;

#[derive(Error, Debug)]
pub enum PlyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid ascii data: {0}")]
    InvalidAscii(#[from] FromUtf8Error),

    #[error("Invalid PLY header: {0}")]
    InvalidHeader(String),

    #[error("Unsupported PLY format: {0}")]
    UnsupportedFormat(String),

    #[error("Parse error: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("Parse error: {0}")]
    ParseFloatError(#[from] ParseFloatError),

    #[error("Property type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Missing required element.")]
    MissingElement,

    #[error("Row deserialization requires struct or map")]
    RowMustBeStructOrMap,

    #[error("Property type mismatch: expected list but found scalar")]
    ExpectedListProperty,

    #[error("Failed to read ASCII token")]
    NoTokenFound,

    #[error("UTF-8 encoding error: {0}")]
    Utf8Encoding(String),
}

impl serde::de::Error for PlyError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        PlyError::Utf8Encoding(msg.to_string())
    }
}

impl serde::ser::Error for PlyError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        PlyError::Utf8Encoding(msg.to_string())
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScalarType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    F32,
    F64,
}

impl ScalarType {
    pub fn parse(s: &str) -> Result<Self, PlyError> {
        match s {
            "char" | "int8" => Ok(ScalarType::I8),
            "uchar" | "uint8" => Ok(ScalarType::U8),
            "short" | "int16" => Ok(ScalarType::I16),
            "ushort" | "uint16" => Ok(ScalarType::U16),
            "int" | "int32" => Ok(ScalarType::I32),
            "uint" | "uint32" => Ok(ScalarType::U32),
            "float" | "float32" => Ok(ScalarType::F32),
            "double" | "float64" => Ok(ScalarType::F64),
            _ => Err(PlyError::UnsupportedFormat(format!(
                "Unknown scalar type: {s}"
            ))),
        }
    }

    pub fn size_bytes(&self) -> usize {
        match self {
            ScalarType::I8 | ScalarType::U8 => 1,
            ScalarType::I16 | ScalarType::U16 => 2,
            ScalarType::I32 | ScalarType::U32 | ScalarType::F32 => 4,
            ScalarType::F64 => 8,
        }
    }
}

impl FromStr for ScalarType {
    type Err = PlyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Clone)]
enum PropertyType {
    /// A scalar property with a single value
    Scalar { data_type: ScalarType },
    /// A list property with variable length
    List {
        count_type: ScalarType,
        data_type: ScalarType,
    },
}
#[derive(Debug, Clone)]
pub struct PlyProperty {
    name: String,
    property_type: PropertyType,
}

impl PlyProperty {
    /// Create a scalar property
    pub fn scalar(name: String, data_type: ScalarType) -> Self {
        Self {
            name,
            property_type: PropertyType::Scalar { data_type },
        }
    }

    /// Create a list property
    pub fn list(name: String, count_type: ScalarType, data_type: ScalarType) -> Self {
        Self {
            name,
            property_type: PropertyType::List {
                count_type,
                data_type,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ElementDef {
    pub name: String,
    pub row_count: usize,
    pub properties: Vec<PlyProperty>,
}

#[derive(Debug, Clone)]
pub struct PlyHeader {
    pub format: PlyFormat,
    pub version: String,
    pub elements: Vec<ElementDef>,
    pub comments: Vec<String>,
    pub obj_info: Vec<String>,
}

impl PlyHeader {
    pub fn parse<R: BufRead>(mut reader: R) -> Result<Self, PlyError> {
        let mut line = String::new();

        reader.read_line(&mut line)?;
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
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                return Err(PlyError::InvalidHeader(
                    "Unexpected end of file".to_string(),
                ));
            }

            let line_content = line.trim();
            if line_content.is_empty() {
                continue;
            }

            if line_content == "end_header" {
                break;
            }

            let parts: Vec<&str> = line_content.split_whitespace().collect();
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

                    if let Some(element) = current_element.take() {
                        elements.push(element);
                    }

                    let name = parts[1].to_string();
                    let count = parts[2].parse::<usize>().map_err(|_| {
                        PlyError::InvalidHeader(format!("Invalid element count: {}", parts[2]))
                    })?;

                    current_element = Some(ElementDef {
                        name,
                        row_count: count,
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

                        element.properties.push(PlyProperty {
                            property_type: PropertyType::List {
                                count_type,
                                data_type,
                            },
                            name,
                        });
                    } else {
                        let data_type = ScalarType::parse(parts[1])?;
                        let name = parts[2].to_string();

                        element.properties.push(PlyProperty {
                            property_type: PropertyType::Scalar { data_type },
                            name,
                        });
                    }
                }
                _ => {
                    comments.push(line_content.to_string());
                }
            }
        }

        if let Some(element) = current_element {
            elements.push(element);
        }

        let format = format
            .ok_or_else(|| PlyError::InvalidHeader("Missing format specification".to_string()))?;

        Ok(PlyHeader {
            format,
            version,
            elements,
            comments,
            obj_info,
        })
    }

    pub fn get_element(&self, name: &str) -> Option<ElementDef> {
        self.elements.iter().find(|e| e.name == name).cloned()
    }

    pub fn has_element(&self, name: &str) -> bool {
        self.elements.iter().any(|e| e.name == name)
    }
}

// TODO: Delete when everything is moved to 'native' serde.
pub fn parse_elements<T>(mut reader: impl Read, header: &PlyHeader) -> Result<Vec<T>, PlyError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let element_def = header.elements[0].clone();
    let count = element_def.row_count;

    match header.format {
        PlyFormat::Ascii => {
            let mut deserializer =
                RowDeserializer::new(AsciiValReader::new(&mut reader), &element_def);
            (0..count)
                .map(|_| T::deserialize(&mut deserializer))
                .collect()
        }
        PlyFormat::BinaryLittleEndian => {
            let mut deserializer = RowDeserializer::new(
                BinValReader::<_, LittleEndian>::new(&mut reader),
                &element_def,
            );
            (0..count)
                .map(|_| T::deserialize(&mut deserializer))
                .collect()
        }
        PlyFormat::BinaryBigEndian => {
            let mut deserializer =
                RowDeserializer::new(BinValReader::<_, BigEndian>::new(&mut reader), &element_def);
            (0..count)
                .map(|_| T::deserialize(&mut deserializer))
                .collect()
        }
    }
}
