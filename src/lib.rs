//! Fast PLY parser with type-level format specialization

pub mod de;
pub mod ser;

pub use de::{
    AsciiElementDeserializer, BinaryElementDeserializer, ChunkedElementParser, ChunkedHeaderParser,
    FormatDeserializer,
};
pub use ser::{
    elements_to_bytes, elements_to_writer, to_bytes, to_string, to_writer, PlySerializer,
};

use std::fmt;
use std::io::BufRead;
use std::str::FromStr;
use thiserror::Error;

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

#[derive(Debug, Clone)]
pub struct ElementDef {
    pub name: String,
    pub count: usize,
    pub properties: Vec<PropertyType>,
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

        // Read first line - must be "ply"
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
                        let data_type = ScalarType::parse(parts[1])?;
                        let name = parts[2].to_string();

                        element
                            .properties
                            .push(PropertyType::Scalar { data_type, name });
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

    pub fn get_element(&self, name: &str) -> Option<&ElementDef> {
        self.elements.iter().find(|e| e.name == name)
    }

    pub fn has_element(&self, name: &str) -> bool {
        self.elements.iter().any(|e| e.name == name)
    }
}

/// Parse elements from a reader after the header has been parsed
pub fn parse_elements<R, T>(
    reader: R,
    header: &PlyHeader,
    element_name: &str,
) -> Result<Vec<T>, PlyError>
where
    R: BufRead,
    T: for<'de> serde::Deserialize<'de>,
{
    let element_def = header
        .get_element(element_name)
        .ok_or_else(|| PlyError::MissingElement(element_name.to_string()))?;

    // Validate struct compatibility with PLY properties once upfront
    let properties = element_def.properties.to_vec();

    let mut results = Vec::new();

    match header.format {
        PlyFormat::Ascii => {
            let mut deserializer =
                AsciiElementDeserializer::new(reader, element_def.count, properties);
            while let Some(element) = deserializer.next_element::<T>()? {
                results.push(element);
            }
        }
        PlyFormat::BinaryLittleEndian => {
            let mut deserializer = BinaryElementDeserializer::<_, byteorder::LittleEndian>::new(
                reader,
                element_def.count,
                properties,
            );
            while let Some(element) = deserializer.next_element::<T>()? {
                results.push(element);
            }
        }
        PlyFormat::BinaryBigEndian => {
            let mut deserializer = BinaryElementDeserializer::<_, byteorder::BigEndian>::new(
                reader,
                element_def.count,
                properties,
            );
            while let Some(element) = deserializer.next_element::<T>()? {
                results.push(element);
            }
        }
    }

    Ok(results)
}

/// Create a chunked header parser for async-compatible header parsing
pub fn chunked_header_parser() -> ChunkedHeaderParser {
    ChunkedHeaderParser::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

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

        let cursor = BufReader::new(Cursor::new(header_text));
        let header = PlyHeader::parse(cursor).unwrap();

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
