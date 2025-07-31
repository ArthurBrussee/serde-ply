//! Consolidated parsing tests with real PLY data

use serde::Deserialize;
use serde_ply::{PlyFormat, PlyHeader};
use std::io::{BufReader, Cursor};

#[derive(Deserialize, Debug, PartialEq)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Deserialize, Debug, PartialEq)]
struct VertexWithNormal {
    x: f32,
    y: f32,
    z: f32,
    nx: f32,
    ny: f32,
    nz: f32,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Face {
    #[serde(alias = "vertex_indices", alias = "vertex_index")]
    vertex_indices: Vec<u32>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct AllTypes {
    a: i8,
    b: i8,
    c: u8,
    d: u8,
    e: i16,
    f: i16,
    g: u16,
    h: u16,
    i: i32,
    j: i32,
    k: u32,
    l: u32,
    m: f32,
    n: f32,
    o: f64,
    p: f64,
}

#[test]
fn test_basic_ascii_parsing() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
end_header
0.0 0.0 0.0
1.0 0.0 0.0
0.5 1.0 0.0
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    assert_eq!(header.format, PlyFormat::Ascii);
    assert_eq!(header.elements.len(), 1);

    let vertices: Vec<Vertex> = serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
    assert_eq!(vertices.len(), 3);
    assert_eq!(
        vertices[0],
        Vertex {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }
    );
}

#[test]
fn test_ascii_incomplete() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
end_header
0.0 0.0 0.0
1.0 0.0
0.5 1.0 0.0
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    assert_eq!(header.format, PlyFormat::Ascii);
    assert_eq!(header.elements.len(), 1);
    let res = serde_ply::parse_elements::<_, Vertex>(&mut reader, &header, "vertex");
    assert!(res.is_err());
}

#[test]
fn test_greg_turk_cube() {
    let ply_data = r#"ply
format ascii 1.0
comment made by Greg Turk
comment this file is a cube
element vertex 8
property float x
property float y
property float z
element face 6
property list uchar int vertex_index
end_header
0 0 0
0 0 1
0 1 1
0 1 0
1 0 0
1 0 1
1 1 1
1 1 0
4 0 1 2 3
4 7 6 5 4
4 0 4 5 1
4 1 5 6 2
4 2 6 7 3
4 3 7 4 0
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    assert_eq!(header.format, PlyFormat::Ascii);
    assert_eq!(header.elements.len(), 2);

    let vertices: Vec<Vertex> = serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
    let faces: Vec<Face> = serde_ply::parse_elements(&mut reader, &header, "face").unwrap();

    assert_eq!(vertices.len(), 8);
    assert_eq!(faces.len(), 6);
    assert_eq!(faces[0].vertex_indices, vec![0, 1, 2, 3]);
}

#[test]
fn test_all_scalar_types() {
    let ply_data = r#"ply
format ascii 1.0
element point 1
property char a
property int8 b
property uchar c
property uint8 d
property short e
property int16 f
property uint16 g
property ushort h
property int32 i
property int j
property uint32 k
property uint l
property float32 m
property float n
property float64 o
property double p
end_header
1 1 2 2 3 3 4 4 5 5 6 6 7 7 8 8
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    let points: Vec<AllTypes> = serde_ply::parse_elements(&mut reader, &header, "point").unwrap();
    assert_eq!(points.len(), 1);

    let point = &points[0];
    assert_eq!(point.a, 1);
    assert_eq!(point.c, 2);
    assert_eq!(point.e, 3);
    assert_eq!(point.g, 4);
    assert_eq!(point.i, 5);
    assert_eq!(point.k, 6);
    assert_eq!(point.m, 7.0);
    assert_eq!(point.o, 8.0);
}

#[test]
fn test_house_with_normals() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 5
property float x
property float y
property float z
property float nx
property float ny
property float nz
element face 3
property list uchar uint vertex_indices
end_header
1.000000 -1.000000 0.000000 -0.000000 0.000000 1.000000
-1.000000 1.000000 0.000000 -0.000000 0.000000 1.000000
-1.000000 -1.000000 0.000000 -0.000000 0.000000 1.000000
1.000000 1.000000 0.000000 -0.000000 0.000000 1.000000
0.000000 2.000000 0.000000 0.000000 0.000000 1.000000
3 0 1 2
3 0 3 1
3 1 3 4
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<VertexWithNormal> =
        serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
    let faces: Vec<Face> = serde_ply::parse_elements(&mut reader, &header, "face").unwrap();

    assert_eq!(vertices.len(), 5);
    assert_eq!(faces.len(), 3);
    assert_eq!(vertices[0].nx, 0.0);
    assert_eq!(vertices[0].nz, 1.0);
}

#[test]
fn test_empty_elements() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 0
property float x
property float y
property float z
element face 0
property list uchar uint vertex_indices
end_header
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<Vertex> = serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
    let faces: Vec<Face> = serde_ply::parse_elements(&mut reader, &header, "face").unwrap();

    assert_eq!(vertices.len(), 0);
    assert_eq!(faces.len(), 0);
}

#[test]
fn test_scientific_notation() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
end_header
1.5e2 -2.3e-1 4.7E+3
6.8e-4 9.1E2 -3.2e+1
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<Vertex> = serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
    assert_eq!(vertices.len(), 2);
    assert_eq!(vertices[0].x, 150.0);
    assert!((vertices[0].y - (-0.23)).abs() < 0.001);
    assert_eq!(vertices[0].z, 4700.0);
}

#[test]
fn test_binary_little_endian() {
    // Binary PLY: 2 vertices with x,y,z floats
    let mut binary_data = b"ply\nformat binary_little_endian 1.0\nelement vertex 2\nproperty float x\nproperty float y\nproperty float z\nend_header\n".to_vec();

    // Vertex 1: (1.0, 2.0, 3.0)
    binary_data.extend_from_slice(&1.0f32.to_le_bytes());
    binary_data.extend_from_slice(&2.0f32.to_le_bytes());
    binary_data.extend_from_slice(&3.0f32.to_le_bytes());

    // Vertex 2: (4.0, 5.0, 6.0)
    binary_data.extend_from_slice(&4.0f32.to_le_bytes());
    binary_data.extend_from_slice(&5.0f32.to_le_bytes());
    binary_data.extend_from_slice(&6.0f32.to_le_bytes());

    let cursor = Cursor::new(binary_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    assert_eq!(header.format, PlyFormat::BinaryLittleEndian);

    let vertices: Vec<Vertex> = serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
    assert_eq!(vertices.len(), 2);
    assert_eq!(
        vertices[0],
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
    assert_eq!(
        vertices[1],
        Vertex {
            x: 4.0,
            y: 5.0,
            z: 6.0
        }
    );
}

#[test]
fn test_binary_big_endian() {
    let mut binary_data = b"ply\nformat binary_big_endian 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nend_header\n".to_vec();

    binary_data.extend_from_slice(&1.5f32.to_be_bytes());
    binary_data.extend_from_slice(&2.5f32.to_be_bytes());
    binary_data.extend_from_slice(&3.5f32.to_be_bytes());

    let cursor = Cursor::new(binary_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    assert_eq!(header.format, PlyFormat::BinaryBigEndian);

    let vertices: Vec<Vertex> = serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
    assert_eq!(vertices.len(), 1);
    assert_eq!(
        vertices[0],
        Vertex {
            x: 1.5,
            y: 2.5,
            z: 3.5
        }
    );
}

#[test]
fn test_lists() {
    let ply_data = r#"ply
format ascii 1.0
element face 2
property list uchar uint vertex_indices
end_header
3 0 1 2
4 0 1 2 3
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    let faces: Vec<Face> = serde_ply::parse_elements(&mut reader, &header, "face").unwrap();
    assert_eq!(faces.len(), 2);
    assert_eq!(faces[0].vertex_indices, vec![0, 1, 2]);
    assert_eq!(faces[1].vertex_indices, vec![0, 1, 2, 3]);
}

#[test]
fn test_mixed_scalar_and_list() {
    #[derive(Deserialize, Debug)]
    struct VertexWithList {
        x: f32,
        y: f32,
        z: f32,
        indices: Vec<u32>,
    }

    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
property list uchar uint indices
end_header
1.0 2.0 3.0 2 10 20
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<VertexWithList> =
        serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
    assert_eq!(vertices.len(), 1);
    assert_eq!(vertices[0].x, 1.0);
    assert_eq!(vertices[0].y, 2.0);
    assert_eq!(vertices[0].z, 3.0);
    assert_eq!(vertices[0].indices, vec![10, 20]);
}

#[test]
fn test_leading_whitespace() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
end_header
   1.0    2.0    3.0
4.0 5.0 6.0
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<Vertex> = serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
    assert_eq!(vertices.len(), 2);
    assert_eq!(
        vertices[0],
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
}

#[test]
fn test_error_incomplete_data() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
end_header
1.0 2.0 3.0
4.0 5.0
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = PlyHeader::parse(&mut reader).unwrap();

    let result = serde_ply::parse_elements::<_, Vertex>(&mut reader, &header, "vertex");
    assert!(result.is_err());
}
