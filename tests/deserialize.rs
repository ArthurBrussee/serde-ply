use serde::Deserialize;
use serde_ply::{PlyFormat, PlyReader};
use std::{
    collections::HashMap,
    io::{BufReader, Cursor},
};

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
    #[serde(alias = "vertex_index")]
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
5.0 3.0 2.0
1.0 0.0 0.0
0.5 1.0 0.0
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = PlyReader::from_reader(cursor).unwrap();

    assert_eq!(reader.header().format, PlyFormat::Ascii);
    assert_eq!(reader.header().elem_defs.len(), 1);

    let vertices: Vec<Vertex> = reader.next_element().unwrap();
    assert_eq!(vertices.len(), 3);
    assert_eq!(
        vertices[0],
        Vertex {
            x: 5.0,
            y: 3.0,
            z: 2.0
        }
    );
}

#[test]
fn test_parse_rn() {
    let ply_data = "ply\r\nformat ascii 1.0\r\nelement vertex 1\r\nproperty float x\r\nproperty float y\r\nproperty float z\r\nend_header\r\n0 0 1\r\n";
    let mut reader = PlyReader::from_reader(Cursor::new(ply_data)).unwrap();
    assert_eq!(reader.header().format, PlyFormat::Ascii);
    assert_eq!(reader.header().elem_defs.len(), 1);
    let vertices: Vec<Vertex> = reader.next_element().unwrap();
    assert_eq!(vertices.len(), 1);
    assert_eq!(
        vertices[0],
        Vertex {
            x: 0.0,
            y: 0.0,
            z: 1.0
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
    let mut reader = PlyReader::from_reader(BufReader::new(cursor)).unwrap();

    assert_eq!(reader.header().format, PlyFormat::Ascii);
    assert_eq!(reader.header().elem_defs.len(), 1);
    let result = reader.next_element::<Vec<Vertex>>();
    assert!(result.is_err());
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

    // Test header parsing by creating deserializer first
    let cursor = Cursor::new(ply_data);
    let file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();

    assert_eq!(file.header().format, PlyFormat::Ascii);
    assert_eq!(file.header().elem_defs.len(), 2);
    assert_eq!(file.header().comments[0], "made by Greg Turk");
    assert_eq!(file.header().comments[1], "this file is a cube");

    // Use native serde to parse both vertices and faces
    #[derive(Deserialize, Debug)]
    struct PlyData {
        vertex: Vec<Vertex>,
        face: Vec<Face>,
    }

    let cursor = Cursor::new(ply_data);
    let reader = BufReader::new(cursor);
    let ply: PlyData = serde_ply::from_reader(reader).unwrap();

    assert_eq!(ply.vertex.len(), 8);
    assert_eq!(ply.face.len(), 6);
    assert_eq!(ply.face[0].vertex_indices, vec![0, 1, 2, 3]);
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
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let points: Vec<AllTypes> = file.next_element().unwrap();
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
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let vertices: Vec<Vertex> = file.next_element().unwrap();
    let faces: Vec<Face> = file.next_element().unwrap();

    assert_eq!(vertices.len(), 0);
    assert_eq!(faces.len(), 0);
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
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let vertices: Vec<Vertex> = file.next_element().unwrap();
    assert_eq!(file.header().format, PlyFormat::BinaryLittleEndian);
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
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();

    assert_eq!(file.header().format, PlyFormat::BinaryBigEndian);

    let vertices: Vec<Vertex> = file.next_element().unwrap();
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
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let vertices: Vec<VertexWithList> = file.next_element().unwrap();
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
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let vertices: Vec<Vertex> = file.next_element().unwrap();
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
    #[derive(Deserialize, Debug)]
    struct PlyData {
        #[allow(dead_code)]
        vertex: Vec<Vertex>,
    }

    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
end_header
1.0 2.0 3.0
"#;

    let cursor = Cursor::new(ply_data);
    let reader = BufReader::new(cursor);
    let result = serde_ply::from_reader::<PlyData>(reader);
    assert!(result.is_err());
}

#[derive(Deserialize, Debug)]
struct BasicVertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
struct MissingFieldVertex {
    x: f32,
    y: f32,
    z: f32,
    missing_field: f32, // Not in PLY
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
struct ScalarFace {
    vertex_indices: u32,
}

fn create_binary_vertex_data(x: f32, y: f32, z: f32) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&x.to_le_bytes());
    data.extend_from_slice(&y.to_le_bytes());
    data.extend_from_slice(&z.to_le_bytes());
    data
}

#[test]
fn test_missing_required_field() {
    let mut binary_data = b"ply\nformat binary_little_endian 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nend_header\n".to_vec();
    binary_data.extend_from_slice(&create_binary_vertex_data(1.0, 2.0, 3.0));

    let cursor = Cursor::new(binary_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let result = file.next_element::<Vec<MissingFieldVertex>>();
    assert!(result.is_err());
}

#[test]
fn test_extra_ply_fields_ignored_ascii() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
end_header
1.0 2.0 3.0 255 128 64
"#;

    let cursor = Cursor::new(ply_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let vertices: Vec<BasicVertex> = file.next_element().unwrap();
    assert_eq!(vertices.len(), 1);
    assert_eq!(vertices[0].x, 1.0);
    assert_eq!(vertices[0].y, 2.0);
    assert_eq!(vertices[0].z, 3.0);
}

#[test]
fn test_type_coercion_ascii() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property double x
property int y
property uchar z
end_header
1.5 42 200
"#;

    #[derive(Deserialize, Debug)]
    struct CoercedVertex {
        x: f32, // double -> float
        y: f32, // int32 -> float
        z: f32, // uchar -> float
    }

    let cursor = Cursor::new(ply_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let vertices: Vec<CoercedVertex> = file.next_element().unwrap();
    assert_eq!(vertices.len(), 1);
    assert_eq!(vertices[0].x, 1.5);
    assert_eq!(vertices[0].y, 42.0);
    assert_eq!(vertices[0].z, 200.0);
}

#[test]
fn test_type_coercion_binary() {
    let mut binary_data = b"ply\nformat binary_little_endian 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty uchar z\nproperty uint w\nend_header\n".to_vec();
    binary_data.extend_from_slice(&1.5f32.to_le_bytes());
    binary_data.extend_from_slice(&2.5f32.to_le_bytes());
    binary_data.extend_from_slice(&3u8.to_le_bytes());
    binary_data.extend_from_slice(&4u32.to_le_bytes());

    #[derive(Deserialize, Debug)]
    struct CoercedVertex {
        x: f64, // float -> double
        y: f32, // float -> float
        z: u32, // u8 -> uint
        w: u8,  // u8 -> uint
    }

    let cursor = Cursor::new(binary_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let vertices: Vec<CoercedVertex> = file.next_element().unwrap();
    assert_eq!(vertices.len(), 1);
    assert!((vertices[0].x - 1.5).abs() < 0.001);
    assert_eq!(vertices[0].y, 2.5);
    assert_eq!(vertices[0].z, 3);
    assert_eq!(vertices[0].w, 4);
}

#[test]
fn test_integer_overflow_ascii() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property int x
property int y
property int z
end_header
2147483647 -2147483648 1000000000
"#;

    #[derive(Deserialize, Debug)]
    #[allow(unused)]
    struct SmallIntVertex {
        x: i8, // Will overflow
        y: i8, // Will overflow
        z: i8, // Will overflow
    }

    let cursor = Cursor::new(ply_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let result = file.next_element::<Vec<SmallIntVertex>>();
    assert!(result.is_err())
}

#[test]
fn test_integer_overflow_binary() {
    let mut binary_data = b"ply\nformat binary_little_endian 1.0\nelement vertex 1\nproperty int x\nproperty int y\nproperty int z\nend_header\n".to_vec();
    binary_data.extend_from_slice(&2147483647i32.to_le_bytes());
    binary_data.extend_from_slice(&(-2147483648i32).to_le_bytes());
    binary_data.extend_from_slice(&1000000000i32.to_le_bytes());

    #[derive(Deserialize, Debug)]
    #[allow(unused)]
    struct SmallIntVertex {
        x: i8, // Will overflow
        y: i8, // Will overflow
        z: i8, // Will overflow
    }

    let cursor = Cursor::new(binary_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let result = file.next_element::<Vec<SmallIntVertex>>();
    assert!(result.is_err())
}

#[test]
fn test_malformed_list_count_ascii() {
    let ply_data = r#"ply
format ascii 1.0
element face 1
property list uchar uint vertex_indices
end_header
invalid_count 0 1 2
"#;

    let cursor = Cursor::new(ply_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let result = file.next_element::<Vec<Face>>();
    assert!(result.is_err());
}

#[test]
fn test_list_count_mismatch_ascii() {
    let ply_data = r#"ply
format ascii 1.0
element face 1
property list uchar uint vertex_indices
end_header
5 0 1 2
"#;

    let cursor = Cursor::new(ply_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let result = file.next_element::<Vec<Face>>();
    assert!(result.is_err());
}

#[test]
fn test_list_count_mismatch_binary() {
    let mut binary_data = b"ply\nformat binary_little_endian 1.0\nelement face 1\nproperty list uchar uint vertex_indices\nend_header\n".to_vec();
    // Say we have 5 elements but only provide 3
    binary_data.push(5u8); // count = 5
    binary_data.extend_from_slice(&0u32.to_le_bytes());
    binary_data.extend_from_slice(&1u32.to_le_bytes());
    binary_data.extend_from_slice(&2u32.to_le_bytes());
    // Missing 2 more elements

    let cursor = Cursor::new(binary_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let result = file.next_element::<Vec<Face>>();
    assert!(result.is_err());
}

#[test]
fn test_infinity_and_nan_ascii() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
end_header
inf -inf nan
1.0 2.0 3.0
"#;

    let cursor = Cursor::new(ply_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let vertices: Vec<BasicVertex> = file.next_element().unwrap();
    assert_eq!(vertices.len(), 2);

    // Check special float values
    assert!(vertices[0].x.is_infinite() && vertices[0].x.is_sign_positive());
    assert!(vertices[0].y.is_infinite() && vertices[0].y.is_sign_negative());
    assert!(vertices[0].z.is_nan());
}

#[test]
fn test_infinity_and_nan_binary() {
    let mut binary_data = b"ply\nformat binary_little_endian 1.0\nelement vertex 2\nproperty float x\nproperty float y\nproperty float z\nend_header\n".to_vec();

    // First vertex with special values
    binary_data.extend_from_slice(&f32::INFINITY.to_le_bytes());
    binary_data.extend_from_slice(&f32::NEG_INFINITY.to_le_bytes());
    binary_data.extend_from_slice(&f32::NAN.to_le_bytes());

    // Second vertex with normal values
    binary_data.extend_from_slice(&create_binary_vertex_data(1.0, 2.0, 3.0));

    let cursor = Cursor::new(binary_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let vertices: Vec<BasicVertex> = file.next_element().unwrap();
    assert_eq!(vertices.len(), 2);

    // Check special float values
    assert!(vertices[0].x.is_infinite() && vertices[0].x.is_sign_positive());
    assert!(vertices[0].y.is_infinite() && vertices[0].y.is_sign_negative());
    assert!(vertices[0].z.is_nan());
}

#[test]
fn test_property_order_dependency() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property float z
property float x
property float y
end_header
3.0 1.0 2.0
"#;

    let cursor = Cursor::new(ply_data);
    let mut file = PlyReader::from_reader(BufReader::new(cursor)).unwrap();
    let vertices: Vec<BasicVertex> = file.next_element().unwrap();
    assert_eq!(vertices.len(), 1);

    assert_eq!(vertices[0].x, 1.0);
    assert_eq!(vertices[0].y, 2.0);
    assert_eq!(vertices[0].z, 3.0);
}

#[derive(Deserialize, Debug, PartialEq)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Deserialize, Debug)]
struct PlyData {
    vertex: Vec<Vertex>,
    face: Vec<Face>,
}

#[derive(Deserialize, Debug)]
#[allow(unused)]
struct MultiElementPly {
    vertex: Vec<Vertex>,
    color: Vec<Color>,
}

#[test]
fn test_multi_element_struct() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
element face 1
property list uchar uint vertex_indices
end_header
0.0 0.0 0.0
1.0 0.0 0.0
3 0 1 2
"#;

    let cursor = Cursor::new(ply_data);
    let reader = BufReader::new(cursor);
    let ply: PlyData = serde_ply::from_reader(reader).unwrap();

    assert_eq!(ply.vertex.len(), 2);
    assert_eq!(
        ply.vertex[0],
        Vertex {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }
    );
    assert_eq!(
        ply.vertex[1],
        Vertex {
            x: 1.0,
            y: 0.0,
            z: 0.0
        }
    );

    assert_eq!(ply.face.len(), 1);
    assert_eq!(ply.face[0].vertex_indices, vec![0, 1, 2]);
}

#[test]
fn test_multi_element_hashmap() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
element color 2
property uchar red
property uchar green
property uchar blue
end_header
0.0 0.0 0.0
1.0 0.0 0.0
255 128 64
32 16 8
"#;

    let cursor = Cursor::new(ply_data);
    let reader = BufReader::new(cursor);
    let elements: HashMap<String, Vec<HashMap<String, f32>>> =
        serde_ply::from_reader(reader).unwrap();

    assert_eq!(elements.len(), 2);
    assert!(elements.contains_key("vertex"));
    assert!(elements.contains_key("color"));

    let vertices = &elements["vertex"];
    assert_eq!(vertices.len(), 2);
    assert_eq!(vertices[0]["x"], 0.0);
    assert_eq!(vertices[1]["x"], 1.0);

    let colors = &elements["color"];
    assert_eq!(colors.len(), 2);
    assert_eq!(colors[0]["red"], 255.0);
    assert_eq!(colors[1]["red"], 32.0);
}
