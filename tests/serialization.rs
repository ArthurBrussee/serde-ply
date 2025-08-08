use serde::Serialize;
use serde_ply::{ElementDef, PlyFormat, PlyHeader, PlyProperty, ScalarType};
use std::io::Cursor;

#[derive(Serialize, Debug, PartialEq)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Serialize, Debug, PartialEq)]
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
fn test_complete_binary_ply_file() {
    let vertices = vec![
        VertexWithColor {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            red: 255,
            green: 128,
            blue: 64,
        },
        VertexWithColor {
            x: 4.0,
            y: 5.0,
            z: 6.0,
            red: 200,
            green: 100,
            blue: 50,
        },
    ];

    let faces = [Face {
        vertex_indices: vec![0, 1],
    }];

    let header = PlyHeader {
        format: PlyFormat::BinaryLittleEndian,
        version: "1.0".to_string(),
        elements: vec![
            ElementDef {
                name: "vertex".to_string(),
                row_count: vertices.len(),
                properties: vec![
                    PlyProperty::scalar("x".to_string(), ScalarType::F32),
                    PlyProperty::scalar("y".to_string(), ScalarType::F32),
                    PlyProperty::scalar("z".to_string(), ScalarType::F32),
                    PlyProperty::scalar("red".to_string(), ScalarType::U8),
                    PlyProperty::scalar("green".to_string(), ScalarType::U8),
                    PlyProperty::scalar("blue".to_string(), ScalarType::U8),
                ],
            },
            ElementDef {
                name: "face".to_string(),
                row_count: faces.len(),
                properties: vec![PlyProperty::list(
                    "vertex_indices".to_string(),
                    ScalarType::U8,
                    ScalarType::U32,
                )],
            },
        ],
        comments: vec!["Generated test file".to_string()],
        obj_info: vec![],
    };

    let mut buffer = Vec::new();
    serde_ply::to_writer(&mut buffer, &header, &vertices).unwrap();

    let result = buffer;
    assert!(!result.is_empty());

    let header_str = String::from_utf8_lossy(&result[..200]);
    assert!(header_str.contains("ply"));
    assert!(header_str.contains("binary_little_endian"));
    assert!(header_str.contains("vertex 2"));
}

#[test]
fn test_binary_round_trip() {
    #[derive(Serialize, serde::Deserialize, Debug, PartialEq)]
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
            x: 1.5,
            y: -2.5,
            z: 3.7,
            red: 255,
            green: 128,
            blue: 0,
        },
        RoundTripVertex {
            x: -10.0,
            y: 20.0,
            z: -30.0,
            red: 100,
            green: 200,
            blue: 150,
        },
    ];

    let header = PlyHeader {
        format: PlyFormat::BinaryLittleEndian,
        version: "1.0".to_string(),
        elements: vec![ElementDef {
            name: "vertex".to_string(),
            row_count: original_vertices.len(),
            properties: vec![
                PlyProperty::scalar("x".to_string(), ScalarType::F32),
                PlyProperty::scalar("y".to_string(), ScalarType::F32),
                PlyProperty::scalar("z".to_string(), ScalarType::F32),
                PlyProperty::scalar("red".to_string(), ScalarType::U8),
                PlyProperty::scalar("green".to_string(), ScalarType::U8),
                PlyProperty::scalar("blue".to_string(), ScalarType::U8),
            ],
        }],
        comments: vec![],
        obj_info: vec![],
    };

    let ply_bytes = serde_ply::to_bytes(&header, &original_vertices).unwrap();

    let cursor = Cursor::new(ply_bytes);
    let mut file =
        serde_ply::PlyFileDeserializer::from_reader(std::io::BufReader::new(cursor)).unwrap();
    let deserialized_vertices: Vec<RoundTripVertex> = file.next_element().unwrap();

    assert_eq!(original_vertices, deserialized_vertices);
}

#[test]
fn test_ascii_round_trip() {
    #[derive(Serialize, serde::Deserialize, Debug, PartialEq)]
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
            x: 1.0,
            y: 2.0,
            z: 3.0,
            red: 255,
            green: 128,
            blue: 64,
        },
        RoundTripVertex {
            x: 4.0,
            y: 5.0,
            z: 6.0,
            red: 200,
            green: 100,
            blue: 50,
        },
    ];

    let header = PlyHeader {
        format: PlyFormat::Ascii,
        version: "1.0".to_string(),
        elements: vec![ElementDef {
            name: "vertex".to_string(),
            row_count: original_vertices.len(),
            properties: vec![
                PlyProperty::scalar("x".to_string(), ScalarType::F32),
                PlyProperty::scalar("y".to_string(), ScalarType::F32),
                PlyProperty::scalar("z".to_string(), ScalarType::F32),
                PlyProperty::scalar("red".to_string(), ScalarType::U8),
                PlyProperty::scalar("green".to_string(), ScalarType::U8),
                PlyProperty::scalar("blue".to_string(), ScalarType::U8),
            ],
        }],
        comments: vec![],
        obj_info: vec![],
    };

    let ply_bytes = serde_ply::to_bytes(&header, &original_vertices).unwrap();
    let ply_str = String::from_utf8(ply_bytes).unwrap();

    let cursor = Cursor::new(ply_str.as_bytes());
    let mut file =
        serde_ply::PlyFileDeserializer::from_reader(std::io::BufReader::new(cursor)).unwrap();
    let deserialized_vertices: Vec<RoundTripVertex> = file.next_element().unwrap();

    assert_eq!(original_vertices, deserialized_vertices);
}

#[test]
fn test_simple_ascii_output() {
    let vertices = vec![
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        Vertex {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
    ];

    let header = PlyHeader {
        format: PlyFormat::Ascii,
        version: "1.0".to_string(),
        elements: vec![ElementDef {
            name: "vertex".to_string(),
            row_count: vertices.len(),
            properties: vec![
                PlyProperty::scalar("x".to_string(), ScalarType::F32),
                PlyProperty::scalar("y".to_string(), ScalarType::F32),
                PlyProperty::scalar("z".to_string(), ScalarType::F32),
            ],
        }],
        comments: vec![],
        obj_info: vec![],
    };

    let ply_string = serde_ply::to_string(&header, &vertices).unwrap();

    assert!(ply_string.contains("ply"));
    assert!(ply_string.contains("format ascii 1.0"));
    assert!(ply_string.contains("element vertex 2"));
    assert!(ply_string.contains("1 2 3"));
    assert!(ply_string.contains("4 5 6"));
}

#[test]
fn test_to_string_rejects_binary_format() {
    let vertices = vec![Vertex {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    }];

    let header = PlyHeader {
        format: PlyFormat::BinaryLittleEndian,
        version: "1.0".to_string(),
        elements: vec![ElementDef {
            name: "vertex".to_string(),
            row_count: vertices.len(),
            properties: vec![
                PlyProperty::scalar("x".to_string(), ScalarType::F32),
                PlyProperty::scalar("y".to_string(), ScalarType::F32),
                PlyProperty::scalar("z".to_string(), ScalarType::F32),
            ],
        }],
        comments: vec![],
        obj_info: vec![],
    };

    let result = serde_ply::to_string(&header, &vertices);
    assert!(result.is_err());
}
