use serde::{Deserialize, Serialize};
use serde_ply::{from_reader, to_bytes, PlyFormat};
use std::io::Cursor;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Face {
    vertex_indices: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Mesh {
    vertex: Vec<Vertex>,
    face: Vec<Face>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct AllTypes {
    i8_val: i8,
    u8_val: u8,
    i16_val: i16,
    u16_val: u16,
    i32_val: i32,
    u32_val: u32,
    f32_val: f32,
    f64_val: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct AllTypesPly {
    row: Vec<AllTypes>,
}

fn create_test_mesh() -> Mesh {
    Mesh {
        vertex: vec![
            Vertex {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Vertex {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Vertex {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        ],
        face: vec![Face {
            vertex_indices: vec![0, 1, 2],
        }],
    }
}

fn create_all_types() -> AllTypesPly {
    AllTypesPly {
        row: vec![
            AllTypes {
                i8_val: -42,
                u8_val: 255,
                i16_val: -1000,
                u16_val: 65000,
                i32_val: -100000,
                u32_val: 4000000000,
                f32_val: 3.0,
                f64_val: 2.5,
            },
            AllTypes {
                i8_val: 127,
                u8_val: 0,
                i16_val: 32767,
                u16_val: 0,
                i32_val: 2147483647,
                u32_val: 0,
                f32_val: -1.5,
                f64_val: 1e-10,
            },
        ],
    }
}

#[test]
fn roundtrip_ascii() {
    let original = create_test_mesh();

    let bytes = to_bytes(&original, PlyFormat::Ascii).unwrap();

    let str = String::from_utf8(bytes.clone()).unwrap();
    println!("Output: {str}");

    let cursor = Cursor::new(bytes);
    let parsed: Mesh = from_reader(cursor).unwrap();

    assert_eq!(original, parsed);
}

#[test]
fn roundtrip_binary_little_endian() {
    let original = create_test_mesh();

    let bytes = to_bytes(&original, PlyFormat::BinaryLittleEndian).unwrap();
    let cursor = Cursor::new(bytes);
    let parsed: Mesh = from_reader(cursor).unwrap();

    assert_eq!(original, parsed);
}

#[test]
fn roundtrip_binary_big_endian() {
    let original = create_test_mesh();

    let bytes = to_bytes(&original, PlyFormat::BinaryBigEndian).unwrap();
    let cursor = Cursor::new(bytes);
    let parsed: Mesh = from_reader(cursor).unwrap();

    assert_eq!(original, parsed);
}

#[test]
fn roundtrip_all_types_ascii() {
    let original = create_all_types();

    let bytes = to_bytes(&original, PlyFormat::Ascii).unwrap();
    let cursor = Cursor::new(bytes);
    let parsed: AllTypesPly = from_reader(cursor).unwrap();

    assert_eq!(original, parsed);
}

#[test]
fn roundtrip_all_types_binary_le() {
    let original = create_all_types();

    let bytes = to_bytes(&original, PlyFormat::BinaryLittleEndian).unwrap();
    let cursor = Cursor::new(bytes);
    let parsed: AllTypesPly = from_reader(cursor).unwrap();

    assert_eq!(original, parsed);
}

#[test]
fn roundtrip_all_types_binary_be() {
    let original = create_all_types();

    let bytes = to_bytes(&original, PlyFormat::BinaryBigEndian).unwrap();
    let cursor = Cursor::new(bytes);
    let parsed: AllTypesPly = from_reader(cursor).unwrap();

    assert_eq!(original, parsed);
}

#[test]
fn roundtrip_empty_elements() {
    let empty_mesh = Mesh {
        vertex: vec![],
        face: vec![],
    };

    let bytes = to_bytes(&empty_mesh, PlyFormat::Ascii).unwrap();
    let cursor = Cursor::new(bytes);
    let parsed: Mesh = from_reader(cursor).unwrap();

    assert_eq!(empty_mesh, parsed);
}

#[test]
fn roundtrip_single_vertex() {
    let vertices = vec![Vertex {
        x: 1.5,
        y: -2.5,
        z: 3.0,
    }];
    let bytes = to_bytes(&vertices, PlyFormat::BinaryLittleEndian).unwrap();
    let cursor = Cursor::new(bytes);
    let parsed: Vec<Vertex> = from_reader(cursor).unwrap();
    assert_eq!(vertices, parsed);
}

#[test]
fn roundtrip_large_list() {
    let face = Face {
        vertex_indices: (0..1000).collect(),
    };
    let faces = vec![face];

    let bytes = to_bytes(&faces, PlyFormat::Ascii).unwrap();
    let cursor = Cursor::new(bytes);
    let parsed: Vec<Face> = from_reader(cursor).unwrap();

    assert_eq!(faces, parsed);
}
