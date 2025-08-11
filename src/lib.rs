mod de;
mod error;
mod ser;

pub use error::PlyError;

pub use de::{from_reader, from_str};
pub use ser::{to_bytes, to_writer};

pub use de::chunked::{ChunkPlyFile, RowVisitor};

pub use de::PlyFileDeserializer;
use serde::de::Error;

use std::io::BufRead;

use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
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
            _ => Err(PlyError::custom(format!("Unknown scalar type: {s}"))),
        }
    }
}

impl FromStr for ScalarType {
    type Err = PlyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScalarType::I8 => write!(f, "int8"),
            ScalarType::U8 => write!(f, "uint8"),
            ScalarType::I16 => write!(f, "int16"),
            ScalarType::U16 => write!(f, "uint16"),
            ScalarType::I32 => write!(f, "int32"),
            ScalarType::U32 => write!(f, "uint32"),
            ScalarType::F32 => write!(f, "float32"),
            ScalarType::F64 => write!(f, "float64"),
        }
    }
}

#[derive(Debug, Clone)]
enum PropertyType {
    /// A scalar property with a single value
    Scalar(ScalarType),
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
    pub(crate) fn parse<R: BufRead>(mut reader: R) -> Result<Self, PlyError> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line.trim() != "ply" {
            return Err(PlyError::custom("File must start with 'ply'"));
        }

        let mut format = None;
        let mut version = String::new();
        let mut elements = Vec::new();
        let mut comments = Vec::new();
        let mut obj_info = Vec::new();
        let mut current_element: Option<ElementDef> = None;

        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                return Err(PlyError::custom("Unexpected end of file"));
            }

            if line == "end_header\n" {
                break;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            match parts[0] {
                "format" => {
                    if parts.len() < 3 {
                        return Err(PlyError::custom("Invalid format line"));
                    }
                    format = Some(match parts[1] {
                        "ascii" => PlyFormat::Ascii,
                        "binary_little_endian" => PlyFormat::BinaryLittleEndian,
                        "binary_big_endian" => PlyFormat::BinaryBigEndian,
                        _ => return Err(PlyError::custom(parts[1])),
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
                        return Err(PlyError::custom("Invalid element line"));
                    }

                    if let Some(element) = current_element.take() {
                        elements.push(element);
                    }

                    let name = parts[1].to_string();
                    let count = parts[2].parse::<usize>().map_err(|_| {
                        PlyError::custom(format!("Invalid element count: {}", parts[2]))
                    })?;

                    current_element = Some(ElementDef {
                        name,
                        row_count: count,
                        properties: Vec::new(),
                    });
                }
                "property" => {
                    let element = current_element
                        .as_mut()
                        .ok_or_else(|| PlyError::custom("Property without element"))?;

                    if parts.len() < 3 {
                        return Err(PlyError::custom("Invalid property line"));
                    }

                    if parts[1] == "list" {
                        // List property: property list <count_type> <data_type> <name>
                        if parts.len() < 5 {
                            return Err(PlyError::custom("Invalid list property line"));
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
                            property_type: PropertyType::Scalar(data_type),
                            name,
                        });
                    }
                }
                _ => {}
            }
        }
        if let Some(element) = current_element {
            elements.push(element);
        }
        let format = format.ok_or_else(|| PlyError::custom("Missing format specification"))?;
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
