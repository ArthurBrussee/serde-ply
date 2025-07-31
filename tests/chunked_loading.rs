//! Focused tests for chunked PLY loading functionality

use serde::Deserialize;
use serde_ply::PlyFile;

#[derive(Deserialize, Debug, PartialEq)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Face {
    vertex_indices: Vec<u32>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

/// Test basic ASCII chunked parsing with all data fed at once
#[test]
fn test_ascii_basic() {
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

    let mut ply_file = PlyFile::new();
    ply_file.feed_data(ply_data.as_bytes());

    let vertices = ply_file.next_chunk::<Vertex>().unwrap().unwrap();
    assert_eq!(vertices.len(), 3);
    assert_eq!(
        vertices[0],
        Vertex {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }
    );
    assert_eq!(
        vertices[2],
        Vertex {
            x: 0.5,
            y: 1.0,
            z: 0.0
        }
    );
}

/// Test binary chunked parsing (little endian)
#[test]
fn test_binary_basic() {
    let header = "ply\nformat binary_little_endian 1.0\nelement vertex 2\nproperty float x\nproperty float y\nproperty float z\nend_header\n";

    let mut binary_data = Vec::new();
    binary_data.extend_from_slice(header.as_bytes());

    // Add binary vertex data: [1.0, 2.0, 3.0] and [4.0, 5.0, 6.0]
    let vertices = [[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]];
    for vertex in &vertices {
        for &coord in vertex {
            binary_data.extend_from_slice(&coord.to_le_bytes());
        }
    }

    let mut ply_file = PlyFile::new();
    ply_file.feed_data(&binary_data);

    let parsed = ply_file.next_chunk::<Vertex>().unwrap().unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(
        parsed[0],
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
    assert_eq!(
        parsed[1],
        Vertex {
            x: 4.0,
            y: 5.0,
            z: 6.0
        }
    );
}

/// Test binary big endian parsing
#[test]
fn test_binary_big_endian() {
    let header = "ply\nformat binary_big_endian 1.0\nelement vertex 1\nproperty float x\nproperty float y\nproperty float z\nend_header\n";

    let mut binary_data = Vec::new();
    binary_data.extend_from_slice(header.as_bytes());

    let vertex = [42.5f32, -17.25f32, 100.0f32];
    for &coord in &vertex {
        binary_data.extend_from_slice(&coord.to_be_bytes());
    }

    let mut ply_file = PlyFile::new();
    ply_file.feed_data(&binary_data);

    let parsed = ply_file.next_chunk::<Vertex>().unwrap().unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(
        parsed[0],
        Vertex {
            x: 42.5,
            y: -17.25,
            z: 100.0
        }
    );
}

/// Test ASCII incomplete data handling - the critical case
#[test]
fn test_ascii_incomplete_data() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
end_header
1.0 2.0 3.0
"#; // Missing second vertex

    let mut ply_file = PlyFile::new();
    ply_file.feed_data(ply_data.as_bytes());

    // Should get first vertex
    let chunk1 = ply_file.next_chunk::<Vertex>().unwrap().unwrap();
    assert_eq!(chunk1.len(), 1);
    assert_eq!(
        chunk1[0],
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );

    // No more complete vertices available
    let chunk2 = ply_file.next_chunk::<Vertex>().unwrap();
    assert!(chunk2.is_none());

    // Add the missing vertex data
    ply_file.feed_data(b"4.0 5.0 6.0\n");

    // Now should get the second vertex
    let chunk3 = ply_file.next_chunk::<Vertex>().unwrap().unwrap();
    assert_eq!(chunk3.len(), 1);
    assert_eq!(
        chunk3[0],
        Vertex {
            x: 4.0,
            y: 5.0,
            z: 6.0
        }
    );

    // Should be done now
    assert!(ply_file.next_chunk::<Vertex>().unwrap().is_none());
}

/// Test binary incomplete elements - the most critical case for binary
#[test]
fn test_binary_incomplete_elements() {
    let header = "ply\nformat binary_little_endian 1.0\nelement vertex 2\nproperty float x\nproperty float y\nproperty float z\nend_header\n";

    let mut binary_data = Vec::new();
    binary_data.extend_from_slice(header.as_bytes());

    // Add 2 vertices worth of data
    let vertices = [[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]];
    for vertex in &vertices {
        for &coord in vertex {
            binary_data.extend_from_slice(&coord.to_le_bytes());
        }
    }

    let mut ply_file = PlyFile::new();

    // Feed only header + first complete vertex (12 bytes)
    let header_size = header.len();
    let first_vertex_size = 12;
    ply_file.feed_data(&binary_data[..header_size + first_vertex_size]);

    // Should parse exactly one vertex
    let chunk1 = ply_file.next_chunk::<Vertex>().unwrap().unwrap();
    assert_eq!(chunk1.len(), 1);
    assert_eq!(
        chunk1[0],
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );

    // Call again - should return None (no complete elements available)
    assert!(ply_file.next_chunk::<Vertex>().unwrap().is_none());

    // Feed partial second vertex (only 8 bytes out of 12)
    ply_file.feed_data(
        &binary_data[header_size + first_vertex_size..header_size + first_vertex_size + 8],
    );

    // Should still return None (incomplete element)
    assert!(ply_file.next_chunk::<Vertex>().unwrap().is_none());

    // Feed remaining data
    ply_file.feed_data(&binary_data[header_size + first_vertex_size + 8..]);

    // Now should get the second vertex
    let chunk2 = ply_file.next_chunk::<Vertex>().unwrap().unwrap();
    assert_eq!(chunk2.len(), 1);
    assert_eq!(
        chunk2[0],
        Vertex {
            x: 4.0,
            y: 5.0,
            z: 6.0
        }
    );
}

/// Test multiple element types in sequence
#[test]
fn test_multiple_element_types() {
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

    let mut ply_file = PlyFile::new();
    ply_file.feed_data(ply_data.as_bytes());

    // Parse vertices
    let vertices = ply_file.next_chunk::<Vertex>().unwrap().unwrap();
    assert_eq!(vertices.len(), 2);

    // Parse colors
    let colors = ply_file.next_chunk::<Color>().unwrap().unwrap();
    assert_eq!(colors.len(), 2);
    assert_eq!(
        colors[0],
        Color {
            red: 255,
            green: 128,
            blue: 64
        }
    );
    assert_eq!(
        colors[1],
        Color {
            red: 32,
            green: 16,
            blue: 8
        }
    );
}

/// Test list properties with ASCII
#[test]
fn test_ascii_lists() {
    let ply_data = r#"ply
format ascii 1.0
element face 2
property list uchar int vertex_indices
end_header
3 0 1 2
4 0 1 2 3
"#;

    let mut ply_file = PlyFile::new();
    ply_file.feed_data(ply_data.as_bytes());

    let faces = ply_file.next_chunk::<Face>().unwrap().unwrap();
    assert_eq!(faces.len(), 2);
    assert_eq!(faces[0].vertex_indices, vec![0, 1, 2]);
    assert_eq!(faces[1].vertex_indices, vec![0, 1, 2, 3]);
}

/// Test header parsing across chunk boundaries
#[test]
fn test_header_across_chunks() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
end_header
1.0 2.0 3.0
"#;

    let mut ply_file = PlyFile::new();

    // Split exactly at header boundary
    let header_end = ply_data.find("end_header\n").unwrap() + "end_header\n".len();
    let header_part = &ply_data.as_bytes()[..header_end];
    let data_part = &ply_data.as_bytes()[header_end..];

    ply_file.feed_data(header_part);
    assert!(ply_file.header().is_some());

    ply_file.feed_data(data_part);

    let vertices = ply_file.next_chunk::<Vertex>().unwrap().unwrap();
    assert_eq!(vertices.len(), 1);
    assert_eq!(
        vertices[0],
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
}

/// Test very small chunks to stress buffer management
#[test]
fn test_small_chunks() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
end_header
1.0 2.0 3.0
4.0 5.0 6.0
"#;

    let mut ply_file = PlyFile::new();

    // Feed data in 3-byte chunks
    for chunk in ply_data.as_bytes().chunks(3) {
        ply_file.feed_data(chunk);
    }

    let vertices = ply_file.next_chunk::<Vertex>().unwrap().unwrap();
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

/// Test zero elements edge case
#[test]
fn test_zero_elements() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 0
property float x
property float y
property float z
end_header
"#;

    let mut ply_file = PlyFile::new();
    ply_file.feed_data(ply_data.as_bytes());

    assert!(ply_file.next_chunk::<Vertex>().unwrap().is_none());
}

/// Test incremental feeding pattern
#[test]
fn test_incremental_feeding() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
end_header
1.0 2.0 3.0
4.0 5.0 6.0
7.0 8.0 9.0
"#;

    let mut ply_file = PlyFile::new();
    let mut all_vertices = Vec::new();

    // Feed and parse incrementally
    for chunk in ply_data.as_bytes().chunks(20) {
        ply_file.feed_data(chunk);

        // Only try to parse if header is ready
        if ply_file.header().is_some() {
            if let Some(vertices) = ply_file.next_chunk::<Vertex>().unwrap() {
                all_vertices.extend(vertices);
            }
        }
    }

    assert_eq!(all_vertices.len(), 3);
    assert_eq!(
        all_vertices[0],
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
    assert_eq!(
        all_vertices[2],
        Vertex {
            x: 7.0,
            y: 8.0,
            z: 9.0
        }
    );
}
