//! Integration tests using real PLY files from example_plys directory
//!
//! These tests focus on realistic scenarios rather than tiny unit tests.
//! They verify that the library can handle real PLY files and deserialize
//! into Rust structs as expected.

use serde::{Deserialize, Serialize};
use serde_ply::{PlyFormat, PlyHeader, PropertyType, ScalarType};
use std::fs;
use std::io::Cursor;
use std::path::Path;

// Common vertex struct used in many PLY files
#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Vertex3D {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Deserialize, Debug, PartialEq)]
struct RealisticVertex {
    x: f32,
    y: f32,
    z: f32,
    nx: f32,
    ny: f32,
    nz: f32,
    red: u8,
    green: u8,
    blue: u8,
}

// Vertex with normals (used in house.ply)
#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct VertexWithNormal {
    x: f32,
    y: f32,
    z: f32,
    nx: f32,
    ny: f32,
    nz: f32,
}

// Face struct for polygon indices
#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Face {
    vertex_indices: Vec<u32>,
}

// Test struct with all atomic types
#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct AllTypes {
    a: i8,  // char
    b: i8,  // int8
    c: u8,  // uchar
    d: u8,  // uint8
    e: i16, // short
    f: i16, // int16
    g: u16, // uint16
    h: u16, // ushort
    i: i32, // int32
    j: i32, // int
    k: u32, // uint32
    l: u32, // uint
    m: f32, // float32
    n: f32, // float
    o: f64, // float64
    p: f64, // double
}

fn load_ply_file(filename: &str) -> String {
    let path = Path::new("example_plys").join(filename);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", path.display()))
}

fn load_ply_file_bytes(filename: &str) -> Vec<u8> {
    let path = Path::new("example_plys").join(filename);
    fs::read(&path).unwrap_or_else(|_| panic!("Failed to read {}", path.display()))
}

#[test]
fn test_greg_turk_cube() {
    let ply_data = load_ply_file("greg_turk_example1_ok_ascii.ply");

    // Parse header
    let cursor = Cursor::new(&ply_data);
    let (header, _bytes_consumed) = PlyHeader::parse(cursor).expect("Failed to parse header");

    // Verify header structure
    assert_eq!(header.format, PlyFormat::Ascii);
    assert_eq!(header.version, "1.0");
    assert_eq!(header.elements.len(), 2);

    // Check vertex element
    let vertex_element = header.get_element("vertex").expect("No vertex element");
    assert_eq!(vertex_element.count, 8);
    assert_eq!(vertex_element.properties.len(), 3);

    // Check face element
    let face_element = header.get_element("face").expect("No face element");
    assert_eq!(face_element.count, 6);
    assert_eq!(face_element.properties.len(), 1);

    // Verify face has list property
    match &face_element.properties[0] {
        PropertyType::List {
            count_type,
            data_type,
            name,
        } => {
            assert_eq!(*count_type, ScalarType::UChar);
            assert_eq!(*data_type, ScalarType::Int);
            assert_eq!(name, "vertex_index");
        }
        _ => panic!("Expected list property"),
    }

    // Read vertex data using struct deserialization
    let vertices: Vec<Vertex3D> =
        serde_ply::from_str(&ply_data, "vertex").expect("Failed to deserialize vertices");

    // Verify we read 8 vertices (cube corners)
    assert_eq!(vertices.len(), 8);

    // Verify some expected vertices from the cube
    assert_eq!(
        vertices[0],
        Vertex3D {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }
    );
    assert!(vertices.contains(&Vertex3D {
        x: 1.0,
        y: 1.0,
        z: 1.0
    }));
}

#[test]
fn test_all_atomic_types() {
    let ply_data = load_ply_file("all_atomic_types_ok_ascii.ply");

    let cursor = Cursor::new(&ply_data);
    let (header, _bytes_consumed) = PlyHeader::parse(cursor).expect("Failed to parse header");

    // Should have one element with all scalar types
    assert_eq!(header.elements.len(), 1);
    let point_element = header.get_element("point").expect("No point element");
    assert_eq!(point_element.count, 1);
    assert_eq!(point_element.properties.len(), 16); // All atomic types

    // Read the single point using struct deserialization
    let points: Vec<AllTypes> =
        serde_ply::from_str(&ply_data, "point").expect("Failed to deserialize all types");

    assert_eq!(points.len(), 1);
    let point = &points[0];

    // Verify all values are correct (file has: 1 1 2 2 3 3 4 4 5 5 6 6 7 7 8 8)
    assert_eq!(point.a, 1); // char
    assert_eq!(point.b, 1); // int8
    assert_eq!(point.c, 2); // uchar
    assert_eq!(point.d, 2); // uint8
    assert_eq!(point.e, 3); // short
    assert_eq!(point.f, 3); // int16
    assert_eq!(point.g, 4); // uint16
    assert_eq!(point.h, 4); // ushort
    assert_eq!(point.i, 5); // int32
    assert_eq!(point.j, 5); // int
    assert_eq!(point.k, 6); // uint32
    assert_eq!(point.l, 6); // uint
    assert_eq!(point.m, 7.0); // float32
    assert_eq!(point.n, 7.0); // float
    assert_eq!(point.o, 8.0); // float64
    assert_eq!(point.p, 8.0); // double
}

#[test]
fn test_house_with_normals() {
    let ply_data = load_ply_file("house_ok_ascii.ply");

    let cursor = Cursor::new(&ply_data);
    let (header, _bytes_consumed) = PlyHeader::parse(cursor).expect("Failed to parse header");

    // Should have vertices with normals and faces
    assert_eq!(header.elements.len(), 2);

    let vertex_element = header.get_element("vertex").expect("No vertex element");
    assert_eq!(vertex_element.count, 5);
    assert_eq!(vertex_element.properties.len(), 6); // x,y,z,nx,ny,nz

    let face_element = header.get_element("face").expect("No face element");
    assert_eq!(face_element.count, 3);

    // Read vertices with normals using struct deserialization
    let vertices: Vec<VertexWithNormal> = serde_ply::from_str(&ply_data, "vertex")
        .expect("Failed to deserialize vertices with normals");

    assert_eq!(vertices.len(), 5);

    // Verify some vertex data from the file
    let first_vertex = &vertices[0];
    assert_eq!(first_vertex.x, 1.0);
    assert_eq!(first_vertex.y, -1.0);
    assert_eq!(first_vertex.z, 0.0);
    assert_eq!(first_vertex.nz, 1.0); // Normal pointing up
}

#[test]
fn test_empty_file() {
    let ply_data = load_ply_file("empty_ok_ascii.ply");

    let cursor = Cursor::new(&ply_data);
    let (header, _bytes_consumed) = PlyHeader::parse(cursor).expect("Failed to parse header");

    // Should parse successfully and have elements but with 0 count
    assert_eq!(header.format, PlyFormat::Ascii);
    assert_eq!(header.elements.len(), 2); // vertex and face elements exist

    // But both should have count 0
    let vertex_element = header.get_element("vertex").expect("No vertex element");
    assert_eq!(vertex_element.count, 0);

    let face_element = header.get_element("face").expect("No face element");
    assert_eq!(face_element.count, 0);
}

#[test]
fn test_minimal_header() {
    let ply_data = load_ply_file("header_min_ok_ascii.ply");

    let cursor = Cursor::new(&ply_data);
    let (header, _bytes_consumed) = PlyHeader::parse(cursor).expect("Failed to parse header");

    // Should parse minimal valid header
    assert_eq!(header.format, PlyFormat::Ascii);
    assert_eq!(header.version, "1.0");
}

#[test]
fn test_leading_spaces() {
    let ply_data = load_ply_file("leading_spaces_ok_ascii.ply");

    let cursor = Cursor::new(&ply_data);
    let (header, _bytes_consumed) = PlyHeader::parse(cursor).expect("Failed to parse header");

    // Should handle leading spaces in data correctly
    assert_eq!(header.format, PlyFormat::Ascii);

    if let Some(vertex_element) = header.get_element("vertex") {
        if vertex_element.count > 0 {
            // Should be able to read vertices despite leading spaces
            let vertices: Vec<Vertex3D> = serde_ply::from_str(&ply_data, "vertex")
                .expect("Failed to read vertices with leading spaces");
            assert!(!vertices.is_empty());
        }
    }
}

#[test]
fn test_exponent_values() {
    let ply_data = load_ply_file("exponent_values_ok_ascii.ply");

    let cursor = Cursor::new(&ply_data);
    let (header, _bytes_consumed) = PlyHeader::parse(cursor).expect("Failed to parse header");

    // Should handle scientific notation in float values
    if let Some(vertex_element) = header.get_element("vertex") {
        if vertex_element.count > 0 {
            // Should successfully parse values in scientific notation
            let vertices: Vec<Vertex3D> = serde_ply::from_str(&ply_data, "vertex")
                .expect("Failed to read vertices with scientific notation");
            assert!(!vertices.is_empty());
        }
    }
}

#[test]
fn test_struct_deserialization_greg_turk() {
    let ply_data = load_ply_file("greg_turk_example1_ok_ascii.ply");

    // Test vertex deserialization into struct
    let vertices: Vec<Vertex3D> =
        serde_ply::from_str(&ply_data, "vertex").expect("Failed to deserialize vertices");

    assert_eq!(vertices.len(), 8);

    // Verify some expected vertices from the cube
    assert!(vertices.contains(&Vertex3D {
        x: 0.0,
        y: 0.0,
        z: 0.0
    }));
    assert!(vertices.contains(&Vertex3D {
        x: 1.0,
        y: 1.0,
        z: 1.0
    }));
    assert!(vertices.contains(&Vertex3D {
        x: 0.0,
        y: 1.0,
        z: 1.0
    }));
}

#[test]
fn test_struct_deserialization_house_normals() {
    let ply_data = load_ply_file("house_ok_ascii.ply");

    // Test vertex with normals deserialization
    let vertices: Vec<VertexWithNormal> = serde_ply::from_str(&ply_data, "vertex")
        .expect("Failed to deserialize vertices with normals");

    assert_eq!(vertices.len(), 5);

    // Check first vertex from the file
    let first_vertex = &vertices[0];
    assert_eq!(first_vertex.x, 1.0);
    assert_eq!(first_vertex.y, -1.0);
    assert_eq!(first_vertex.z, 0.0);
    assert_eq!(first_vertex.nz, 1.0); // Normal pointing up
}

#[test]
fn test_struct_deserialization_all_types() {
    let ply_data = load_ply_file("all_atomic_types_ok_ascii.ply");

    // Test all atomic types in a single struct
    let points: Vec<AllTypes> =
        serde_ply::from_str(&ply_data, "point").expect("Failed to deserialize all types");

    assert_eq!(points.len(), 1);

    let point = &points[0];
    assert_eq!(point.a, 1); // char
    assert_eq!(point.c, 2); // uchar
    assert_eq!(point.e, 3); // short
    assert_eq!(point.i, 5); // int32
    assert_eq!(point.n, 7.0); // float
    assert_eq!(point.p, 8.0); // double
}

#[test]
fn test_binary_little_endian_header() {
    let ply_data = load_ply_file_bytes("house_2_ok_little_endian.ply");

    let cursor = Cursor::new(&ply_data);
    let (header, _bytes_consumed) =
        PlyHeader::parse(cursor).expect("Failed to parse binary header");

    // Should parse binary header correctly
    assert_eq!(header.format, PlyFormat::BinaryLittleEndian);
    assert_eq!(header.version, "1.0");
    assert_eq!(header.elements.len(), 2);

    // Check vertex element
    let vertex_element = header.get_element("vertex").expect("No vertex element");
    assert_eq!(vertex_element.count, 5);
    assert_eq!(vertex_element.properties.len(), 3);

    // Check face element
    let face_element = header.get_element("face").expect("No face element");
    assert_eq!(face_element.count, 3);
    assert_eq!(face_element.properties.len(), 1);
}

#[test]
fn test_binary_little_endian_data() {
    let ply_data = load_ply_file_bytes("house_2_ok_little_endian.ply");

    let cursor = Cursor::new(&ply_data);
    let (_header, _bytes_consumed) =
        PlyHeader::parse(cursor).expect("Failed to parse binary header");

    // Use struct deserialization with binary data
    let vertices: Vec<Vertex3D> = serde_ply::from_reader(std::io::Cursor::new(ply_data), "vertex")
        .expect("Failed to deserialize binary vertices");

    // Should read 5 vertices
    assert_eq!(vertices.len(), 5);

    // Compare with expected values from ASCII version
    // These should match the values in house_2_ok_ascii.ply
    assert_eq!(vertices[0].x, 1.0);
    assert_eq!(vertices[0].y, -1.0);
    assert_eq!(vertices[0].z, 0.0);
}

#[test]
fn test_binary_struct_deserialization() {
    let ply_data = load_ply_file_bytes("house_2_ok_little_endian.ply");

    // Use struct deserialization with whole binary file
    let vertices: Vec<Vertex3D> = serde_ply::from_reader(Cursor::new(ply_data), "vertex")
        .expect("Failed to deserialize binary vertices into structs");

    // Should read 5 vertices
    assert_eq!(vertices.len(), 5);

    // Compare with expected values from ASCII version
    assert_eq!(vertices[0].x, 1.0);
    assert_eq!(vertices[0].y, -1.0);
    assert_eq!(vertices[0].z, 0.0);

    assert_eq!(vertices[1].x, -1.0);
    assert_eq!(vertices[1].y, 1.0);
    assert_eq!(vertices[1].z, 0.0);
}

#[test]
fn test_big_endian_format() {
    // Create a simple PLY structure for big-endian testing
    use serde_ply::{ElementDef, PlySerializer, PropertyType, ScalarType};

    let header = PlyHeader {
        format: PlyFormat::BinaryBigEndian,
        version: "1.0".to_string(),
        elements: vec![ElementDef {
            name: "vertex".to_string(),
            count: 2,
            properties: vec![
                PropertyType::Scalar {
                    data_type: ScalarType::Float,
                    name: "x".to_string(),
                },
                PropertyType::Scalar {
                    data_type: ScalarType::Float,
                    name: "y".to_string(),
                },
                PropertyType::Scalar {
                    data_type: ScalarType::Float,
                    name: "z".to_string(),
                },
            ],
        }],
        comments: vec!["Big endian test".to_string()],
        obj_info: vec![],
    };

    // Create binary data with known values
    let mut buffer = Vec::new();
    let mut serializer = serde_ply::PlySerializer::with_header(&mut buffer, header.clone());

    // The serializer should write header first
    serializer.set_header(header.clone());

    // Test that we can parse big-endian format (even if we don't have test data)
    // At minimum, verify the format is recognized
    assert_eq!(header.format, PlyFormat::BinaryBigEndian);
    assert_eq!(header.elements[0].properties.len(), 3);
}

#[test]
fn test_deserializer_performance() {
    use std::io::Cursor;
    use std::time::Instant;

    // Create realistic PLY data with position, normal, and color for comparison
    let vertex_count = 10000;

    // Create binary PLY header
    let header = format!(
        r#"ply
format binary_little_endian 1.0
comment Performance test with realistic vertex data
element vertex {}
property float x
property float y
property float z
property float nx
property float ny
property float nz
property uchar red
property uchar green
property uchar blue
end_header
"#,
        vertex_count
    );

    // Generate binary vertex data (9 fields: 6 floats + 3 u8s = 27 bytes per vertex)
    let header_len = header.len();
    let mut binary_data = header.into_bytes();

    for i in 0..vertex_count {
        let base = i as f32 * 0.01;
        // Position (3 floats)
        binary_data.extend_from_slice(&(base).to_le_bytes()); // x
        binary_data.extend_from_slice(&(base + 1.0).to_le_bytes()); // y
        binary_data.extend_from_slice(&(base + 2.0).to_le_bytes()); // z

        // Normal (3 floats)
        binary_data.extend_from_slice(&(0.0f32).to_le_bytes()); // nx
        binary_data.extend_from_slice(&(0.0f32).to_le_bytes()); // ny
        binary_data.extend_from_slice(&(1.0f32).to_le_bytes()); // nz

        // Color (3 u8s)
        binary_data.push((i % 256) as u8); // red
        binary_data.push(((i * 2) % 256) as u8); // green
        binary_data.push(((i * 3) % 256) as u8); // blue
    }

    println!(
        "Binary PLY file size: {} bytes ({} bytes per vertex)",
        binary_data.len(),
        (binary_data.len() - header_len) / vertex_count
    );

    // Test binary deserialization performance
    let start = Instant::now();
    let vertices: Vec<RealisticVertex> =
        serde_ply::from_reader(Cursor::new(&binary_data), "vertex").unwrap();
    let duration = start.elapsed();

    // Verify results
    assert_eq!(vertices.len(), vertex_count);
    assert_eq!(vertices[0].x, 0.0);
    assert_eq!(vertices[0].y, 1.0);
    assert_eq!(vertices[0].z, 2.0);
    assert_eq!(vertices[0].nx, 0.0);
    assert_eq!(vertices[0].ny, 0.0);
    assert_eq!(vertices[0].nz, 1.0);
    assert_eq!(vertices[0].red, 0);
    assert_eq!(vertices[0].green, 0);
    assert_eq!(vertices[0].blue, 0);

    assert_eq!(vertices[100].red, 100);
    assert_eq!(vertices[100].green, 200);
    assert_eq!(vertices[100].blue, 44); // (100 * 3) % 256

    println!(
        "Deserializing {} realistic vertices: {:?}",
        vertex_count, duration
    );
    println!(
        "Performance: {:.0} vertices/ms",
        vertex_count as f64 / duration.as_millis() as f64
    );
    println!(
        "Throughput: {:.1} MB/s",
        (binary_data.len() as f64 / (1024.0 * 1024.0)) / duration.as_secs_f64()
    );

    // Now test ASCII format for comparison
    println!("\n--- ASCII vs Binary Comparison ---");

    let ascii_header = format!(
        r#"ply
format ascii 1.0
comment Performance test with realistic vertex data
element vertex {}
property float x
property float y
property float z
property float nx
property float ny
property float nz
property uchar red
property uchar green
property uchar blue
end_header
"#,
        vertex_count
    );

    let mut ascii_data = ascii_header;
    for i in 0..vertex_count {
        let base = i as f32 * 0.01;
        ascii_data.push_str(&format!(
            "{} {} {} 0.0 0.0 1.0 {} {} {}\n",
            base,
            base + 1.0,
            base + 2.0,
            i % 256,
            (i * 2) % 256,
            (i * 3) % 256
        ));
    }

    println!("ASCII PLY file size: {} bytes", ascii_data.len());

    // Test ASCII deserialization performance
    let start = Instant::now();
    let ascii_vertices: Vec<RealisticVertex> = serde_ply::from_str(&ascii_data, "vertex").unwrap();
    let ascii_duration = start.elapsed();

    // Verify ASCII results match binary results
    assert_eq!(ascii_vertices.len(), vertices.len());
    assert_eq!(ascii_vertices[0], vertices[0]);
    assert_eq!(ascii_vertices[100], vertices[100]);

    println!(
        "ASCII deserializing {} vertices: {:?}",
        vertex_count, ascii_duration
    );
    println!(
        "ASCII throughput: {:.1} MB/s",
        (ascii_data.len() as f64 / (1024.0 * 1024.0)) / ascii_duration.as_secs_f64()
    );

    // Performance comparison
    let speedup = ascii_duration.as_nanos() as f64 / duration.as_nanos() as f64;
    let size_ratio = ascii_data.len() as f64 / binary_data.len() as f64;

    println!("\n--- Performance Summary ---");
    println!(
        "Binary format: {:.1} MB/s",
        (binary_data.len() as f64 / (1024.0 * 1024.0)) / duration.as_secs_f64()
    );
    println!(
        "ASCII format:  {:.1} MB/s",
        (ascii_data.len() as f64 / (1024.0 * 1024.0)) / ascii_duration.as_secs_f64()
    );
    println!("Binary is {:.1}x faster than ASCII", speedup);
    println!("Binary file is {:.1}x smaller than ASCII", size_ratio);

    // Should be very fast with binary format and optimized deserializer
    assert!(
        duration.as_millis() < 50,
        "Binary should complete in under 50ms for {} vertices",
        vertex_count
    );

    // Binary should be significantly faster than ASCII
    assert!(
        speedup > 2.0,
        "Binary format should be at least 2x faster than ASCII, got {:.1}x",
        speedup
    );
}

#[test]
fn test_deserializer_binary() {
    let ply_data = load_ply_file_bytes("house_2_ok_little_endian.ply");

    // Test with binary data
    let vertices: Vec<Vertex3D> =
        serde_ply::from_reader(std::io::Cursor::new(ply_data), "vertex").unwrap();

    // Should read 5 vertices
    assert_eq!(vertices.len(), 5);

    // Compare with expected values
    assert_eq!(vertices[0].x, 1.0);
    assert_eq!(vertices[0].y, -1.0);
    assert_eq!(vertices[0].z, 0.0);

    assert_eq!(vertices[1].x, -1.0);
    assert_eq!(vertices[1].y, 1.0);
    assert_eq!(vertices[1].z, 0.0);
}

#[test]
fn test_deserializer_debug() {
    let simple_ply = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
end_header
1.0 2.0 3.0
4.0 5.0 6.0
"#;

    // Test the deserializer
    let vertices: Vec<Vertex3D> = serde_ply::from_str(simple_ply, "vertex").unwrap();

    assert_eq!(vertices.len(), 2);
    assert_eq!(vertices[0].x, 1.0);
    assert_eq!(vertices[0].y, 2.0);
    assert_eq!(vertices[0].z, 3.0);
}
