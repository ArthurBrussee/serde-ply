//! Serde-based PLY file format serialization and deserialization.
//!
//! This crate provides fast, flexible PLY file parsing and writing using serde.
//! It supports both ASCII and binary formats, streaming/chunked processing,
//! and the full PLY specification including list properties.
//!
//! # Quick Start
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use serde_ply::{from_str, to_string, SerializeOptions};
//!
//! #[derive(Deserialize, Serialize)]
//! struct Vertex {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//! }
//!
//! #[derive(Deserialize, Serialize)]
//! struct Mesh {
//!     vertex: Vec<Vertex>,
//! }
//!
//! let ply_text = r#"ply
//! format ascii 1.0
//! element vertex 2
//! property float x
//! property float y
//! property float z
//! end_header
//! 0.0 0.0 0.0
//! 1.0 1.0 1.0
//! "#;
//!
//! let mesh: Mesh = from_str(ply_text)?;
//! let output = to_string(&mesh, SerializeOptions::ascii())?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod de;
mod error;
mod ser;

pub use de::{
    chunked::{PlyChunkedReader, RowVisitor},
    PlyReader,
};
pub use de::{from_bytes, from_reader, from_str};
pub use error::{DeserializeError, SerializeError};
pub use ser::{to_bytes, to_string, to_writer, SerializeOptions};

use std::io::BufRead;

use std::fmt::{self, Display};
use std::str::FromStr;

/// PLY file format encoding.
///
/// Determines how data is stored in the PLY file - as text or binary with specific byte ordering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlyFormat {
    /// Plain text format
    Ascii,
    /// Binary format with little-endian byte order
    BinaryLittleEndian,
    /// Binary format with big-endian byte order
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

/// Scalar data type used in PLY properties.
///
/// Maps PLY type names to Rust types. PLY supports both canonical names
/// (like `float32`) and legacy aliases (like `float`).
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
    pub(crate) fn parse(s: &str) -> Result<Self, DeserializeError> {
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

/// PLY property type definition.
///
/// Properties can be either single scalar values or variable-length lists.
/// Lists store a count followed by that many data elements.
#[derive(Debug, Clone)]
pub enum PropertyType {
    /// Single scalar value
    Scalar(ScalarType),
    /// Variable-length list with count and data types
    List {
        count_type: ScalarType,
        data_type: ScalarType,
    },
}

/// Definition of a single property within a PLY element.
///
/// Contains the property name and its type (scalar or list).
#[derive(Debug, Clone)]
pub struct PlyProperty {
    pub name: String,
    pub property_type: PropertyType,
}

/// Definition of a PLY element type.
///
/// Elements define the structure of data rows in a PLY file. Common examples
/// are "vertex" and "face" elements. Each element has a count and list of properties.
#[derive(Debug, Clone)]
pub struct ElementDef {
    pub name: String,
    pub count: usize,
    pub properties: Vec<PlyProperty>,
}

impl ElementDef {
    /// Find a property by name.
    pub fn get_property(&self, name: &str) -> Option<&PlyProperty> {
        self.properties.iter().find(|p| p.name == name)
    }

    /// Check if the element has a property.
    pub fn has_property(&self, name: &str) -> bool {
        self.get_property(name).is_some()
    }
}

/// PLY file header containing format, elements, and metadata.
///
/// The header defines the structure of the entire PLY file including
/// data format, element definitions, and optional comments.
#[derive(Debug, Clone)]
pub struct PlyHeader {
    pub format: PlyFormat,
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

            // We have reached the end of the header. Don't really care
            // about whitespace here, BUT we do have to be sure that the newline IS present.
            // If its not, the line ends at EOF, and the next chunk of data could then incorrectly contain this newline.
            if line.trim() == "end_header" && line.ends_with("\n") {
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
            elem_defs: elements,
            comments,
            obj_info,
        })
    }

    /// Find an element definition by name.
    pub fn get_element(&self, name: &str) -> Option<ElementDef> {
        self.elem_defs.iter().find(|e| e.name == name).cloned()
    }

    /// Check if the header contains an element with the given name.
    pub fn has_element(&self, name: &str) -> bool {
        self.elem_defs.iter().any(|e| e.name == name)
    }
}

/// Wrapper to serialize PLY lists with `u16` count type.
///
/// By default, PLY lists use `u8` for the element count. Use this wrapper
/// when you need larger counts.
///
/// # Example
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use serde_ply::ListCountU16;
///
/// #[derive(Serialize, Deserialize)]
/// struct Face {
///     // This list can have up to 65535 vertices
///     vertex_indices: ListCountU16<Vec<u32>>,
/// }
/// ```
#[derive(Debug)]
pub struct ListCountU16<T>(pub T);

/// Wrapper to serialize PLY lists with `u32` count type.
///
/// Use this wrapper when you need very large element counts in lists.
///
/// # Example
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use serde_ply::ListCountU32;
///
/// #[derive(Serialize, Deserialize)]
/// struct LargeFace {
///     // This list can have up to 4 billion vertices
///     vertex_indices: ListCountU32<Vec<u32>>,
/// }
/// ```
#[derive(Debug)]
pub struct ListCountU32<T>(pub T);

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

impl_list_count_traits!(ListCountU16);
impl_list_count_traits!(ListCountU32);
