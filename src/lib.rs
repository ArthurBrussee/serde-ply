mod de;
mod error;
mod ser;

pub use error::{DeserializeError, SerializeError};

pub use de::{from_reader, from_str};
pub use ser::{to_bytes, to_writer};

pub use de::chunked::{ChunkPlyFile, RowVisitor};

pub use de::PlyFileDeserializer;

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
    pub fn parse(s: &str) -> Result<Self, DeserializeError> {
        match s {
            "char" | "int8" => Ok(ScalarType::I8),
            "uchar" | "uint8" => Ok(ScalarType::U8),
            "short" | "int16" => Ok(ScalarType::I16),
            "ushort" | "uint16" => Ok(ScalarType::U16),
            "int" | "int32" => Ok(ScalarType::I32),
            "uint" | "uint32" => Ok(ScalarType::U32),
            "float" | "float32" => Ok(ScalarType::F32),
            "double" | "float64" => Ok(ScalarType::F64),
            _ => Err(DeserializeError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unknown scalar type: {}", s),
            ))),
        }
    }
}

impl FromStr for ScalarType {
    type Err = DeserializeError;

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
    pub name: String,
    property_type: PropertyType,
}

#[derive(Debug, Clone)]
pub struct ElementDef {
    pub name: String,
    pub count: usize,
    pub properties: Vec<PlyProperty>,
}

impl ElementDef {
    pub fn get_property(&self, name: &str) -> Option<&PlyProperty> {
        self.properties.iter().find(|p| p.name == name)
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.get_property(name).is_some()
    }
}

#[derive(Debug, Clone)]
pub struct PlyHeader {
    pub format: PlyFormat,
    pub version: String,
    pub elem_defs: Vec<ElementDef>,
    pub comments: Vec<String>,
    pub obj_info: Vec<String>,
}

impl PlyHeader {
    pub(crate) fn parse<R: BufRead>(mut reader: R) -> Result<Self, DeserializeError> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line.trim() != "ply" {
            return Err(DeserializeError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "File must start with 'ply'",
            )));
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
                return Err(DeserializeError(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Unexpected end of file",
                )));
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
                        return Err(DeserializeError(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid format line",
                        )));
                    }
                    format = Some(match parts[1] {
                        "ascii" => PlyFormat::Ascii,
                        "binary_little_endian" => PlyFormat::BinaryLittleEndian,
                        "binary_big_endian" => PlyFormat::BinaryBigEndian,
                        _ => {
                            return Err(DeserializeError(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Unknown format: {}", parts[1]),
                            )))
                        }
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
                        return Err(DeserializeError(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid element line",
                        )));
                    }

                    if let Some(element) = current_element.take() {
                        elements.push(element);
                    }

                    let name = parts[1].to_string();
                    let count = parts[2].parse::<usize>().map_err(|_| {
                        DeserializeError(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid element count: {}", parts[2]),
                        ))
                    })?;

                    current_element = Some(ElementDef {
                        name,
                        count,
                        properties: Vec::new(),
                    });
                }
                "property" => {
                    let element = current_element.as_mut().ok_or_else(|| {
                        DeserializeError(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Property without element",
                        ))
                    })?;

                    if parts.len() < 3 {
                        return Err(DeserializeError(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid property line",
                        )));
                    }

                    if parts[1] == "list" {
                        // List property: property list <count_type> <data_type> <name>
                        if parts.len() < 5 {
                            return Err(DeserializeError(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid list property line",
                            )));
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
        let format = format.ok_or_else(|| {
            DeserializeError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing format specification",
            ))
        })?;
        Ok(PlyHeader {
            format,
            version,
            elem_defs: elements,
            comments,
            obj_info,
        })
    }

    pub fn get_element(&self, name: &str) -> Option<ElementDef> {
        self.elem_defs.iter().find(|e| e.name == name).cloned()
    }

    pub fn has_element(&self, name: &str) -> bool {
        self.elem_defs.iter().any(|e| e.name == name)
    }
}

/// Newtype wrappers for PLY list properties with custom count types.
///
/// These types allow you to specify the count type for PLY list properties.
///
/// # Examples
///
/// ```rust
/// use serde::{Deserialize, Serialize};
/// use serde_ply::{ListCountU16, ListCountU32};
///
/// #[derive(Deserialize, Serialize)]
/// struct Face {
///     // Standard list (u8 count, max 255 elements)
///     small_indices: Vec<u32>,
///
///     // Medium list (u16 count, max 65535 elements)
///     medium_indices: ListCountU16<Vec<u32>>,
///
///     // Large list (u32 count, max ~4 billion elements)
///     large_indices: ListCountU32<Vec<u32>>,
/// }
/// ```
/// PLY list with u8 count type (max 255 elements).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListCountU8<T>(pub T);

/// PLY list with u16 count type (max 65535 elements).
#[derive(Debug)]
pub struct ListCountU16<T>(pub T);

/// PLY list with u32 count type (max ~4 billion elements).
#[derive(Debug)]
pub struct ListCountU32<T>(pub T);

/// Helper trait to identify list count types and their corresponding scalar types.
pub trait ListCountType {
    /// The scalar type used for the count field in PLY format.
    fn count_scalar_type() -> ScalarType;
}

impl<T> ListCountType for ListCountU8<T> {
    fn count_scalar_type() -> ScalarType {
        ScalarType::U8
    }
}

impl<T> ListCountType for ListCountU16<T> {
    fn count_scalar_type() -> ScalarType {
        ScalarType::U16
    }
}

impl<T> ListCountType for ListCountU32<T> {
    fn count_scalar_type() -> ScalarType {
        ScalarType::U32
    }
}

// Implement common traits for all ListCount types
macro_rules! impl_list_count_traits {
    ($wrapper:ident) => {
        impl<T> From<T> for $wrapper<T> {
            fn from(inner: T) -> Self {
                $wrapper(inner)
            }
        }
        impl<T> std::ops::Deref for $wrapper<T> {
            type Target = T;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T> std::ops::DerefMut for $wrapper<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<T> serde::Serialize for $wrapper<T>
        where
            T: serde::Serialize,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_newtype_struct(stringify!($wrapper), &self.0)
            }
        }

        impl<'de, T> serde::Deserialize<'de> for $wrapper<T>
        where
            T: serde::Deserialize<'de>,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                T::deserialize(deserializer).map($wrapper)
            }
        }
    };
}

impl_list_count_traits!(ListCountU8);
impl_list_count_traits!(ListCountU16);
impl_list_count_traits!(ListCountU32);
