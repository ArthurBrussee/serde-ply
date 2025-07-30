//! Comprehensive tests for chunked PLY loading functionality

use serde::Deserialize;
use serde_ply::{PlyError, PlyFile};

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
struct ColorVertex {
    x: f32,
    y: f32,
    z: f32,
    red: u8,
    green: u8,
    blue: u8,
}

#[test]
fn test_basic_chunked_loading() {
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

    assert!(ply_file.is_header_ready());

    let mut vertex_reader = ply_file.element_reader("vertex").unwrap();
    let mut all_vertices = Vec::new();

    while let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file).unwrap() {
        all_vertices.extend(chunk);
    }

    assert_eq!(all_vertices.len(), 3);
    assert_eq!(
        all_vertices[0],
        Vertex {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }
    );
    assert_eq!(
        all_vertices[1],
        Vertex {
            x: 1.0,
            y: 0.0,
            z: 0.0
        }
    );
    assert_eq!(
        all_vertices[2],
        Vertex {
            x: 0.5,
            y: 1.0,
            z: 0.0
        }
    );
}

#[test]
fn test_very_small_chunks() {
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

    // Feed data in very small chunks (5 bytes at a time)
    for chunk in ply_data.as_bytes().chunks(5) {
        ply_file.feed_data(chunk);
    }

    assert!(ply_file.is_header_ready());

    let mut vertex_reader = ply_file.element_reader("vertex").unwrap();
    let mut all_vertices = Vec::new();

    while let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file).unwrap() {
        all_vertices.extend(chunk);
    }

    assert_eq!(all_vertices.len(), 2);
    assert_eq!(
        all_vertices[0],
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
    assert_eq!(
        all_vertices[1],
        Vertex {
            x: 4.0,
            y: 5.0,
            z: 6.0
        }
    );
}

#[test]
fn test_header_parsing_across_chunks() {
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
    assert!(ply_file.is_header_ready());

    ply_file.feed_data(data_part);

    let mut vertex_reader = ply_file.element_reader("vertex").unwrap();
    let vertices = vertex_reader
        .next_chunk::<Vertex>(&mut ply_file)
        .unwrap()
        .unwrap();

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

#[test]
fn test_multiple_element_types() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
element face 2
property list uchar int vertex_indices
end_header
0.0 0.0 0.0
1.0 0.0 0.0
0.5 1.0 0.0
3 0 1 2
3 1 2 0
"#;

    let mut ply_file = PlyFile::new();
    ply_file.feed_data(ply_data.as_bytes());

    // Parse all vertices first
    let mut vertex_reader = ply_file.element_reader("vertex").unwrap();
    let mut all_vertices = Vec::new();

    while let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file).unwrap() {
        all_vertices.extend(chunk);
    }

    assert_eq!(all_vertices.len(), 3);
    assert!(vertex_reader.is_finished());

    // Advance to faces
    ply_file.advance_to_next_element().unwrap();

    let mut face_reader = ply_file.element_reader("face").unwrap();
    let mut all_faces = Vec::new();

    while let Some(chunk) = face_reader.next_chunk::<Face>(&mut ply_file).unwrap() {
        all_faces.extend(chunk);
    }

    assert_eq!(all_faces.len(), 2);
    assert_eq!(all_faces[0].vertex_indices, vec![0, 1, 2]);
    assert_eq!(all_faces[1].vertex_indices, vec![1, 2, 0]);
}

#[test]
fn test_binary_little_endian_chunked() {
    // Create binary PLY data
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

    // Feed in chunks
    for chunk in binary_data.chunks(10) {
        ply_file.feed_data(chunk);
    }

    assert!(ply_file.is_header_ready());

    let mut vertex_reader = ply_file.element_reader("vertex").unwrap();
    let mut all_vertices = Vec::new();

    while let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file).unwrap() {
        all_vertices.extend(chunk);
    }

    assert_eq!(all_vertices.len(), 2);
    assert_eq!(
        all_vertices[0],
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
    assert_eq!(
        all_vertices[1],
        Vertex {
            x: 4.0,
            y: 5.0,
            z: 6.0
        }
    );
}

#[test]
fn test_incomplete_data_handling() {
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

    let mut vertex_reader = ply_file.element_reader("vertex").unwrap();

    // Should get first vertex
    let first_chunk = vertex_reader
        .next_chunk::<Vertex>(&mut ply_file)
        .unwrap()
        .unwrap();
    assert_eq!(first_chunk.len(), 1);
    assert_eq!(
        first_chunk[0],
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );

    // No more complete vertices available
    let second_chunk = vertex_reader.next_chunk::<Vertex>(&mut ply_file).unwrap();
    assert!(second_chunk.is_none());

    // Add the missing vertex data
    ply_file.feed_data(b"4.0 5.0 6.0\n");

    // Now should get the second vertex
    let third_chunk = vertex_reader
        .next_chunk::<Vertex>(&mut ply_file)
        .unwrap()
        .unwrap();
    assert_eq!(third_chunk.len(), 1);
    assert_eq!(
        third_chunk[0],
        Vertex {
            x: 4.0,
            y: 5.0,
            z: 6.0
        }
    );

    assert!(vertex_reader.is_finished());
}

#[test]
fn test_error_handling() {
    let mut ply_file = PlyFile::new();

    // Try to get reader before header is ready
    let result = ply_file.element_reader("vertex");
    assert!(result.is_err());

    // Try to advance before header is ready
    let result = ply_file.advance_to_next_element();
    assert!(result.is_err());
}

#[test]
fn test_empty_element() {
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

    let mut vertex_reader = ply_file.element_reader("vertex").unwrap();
    let chunk = vertex_reader.next_chunk::<Vertex>(&mut ply_file).unwrap();

    assert!(chunk.is_none());
    assert!(vertex_reader.is_finished());
}

#[test]
fn test_chunked_list_properties() {
    let ply_data = r#"ply
format ascii 1.0
element face 2
property list uchar int vertex_indices
end_header
3 0 1 2
4 0 1 2 3
"#;

    let mut ply_file = PlyFile::new();

    // Feed in very small chunks to test list parsing
    for chunk in ply_data.as_bytes().chunks(3) {
        ply_file.feed_data(chunk);
    }

    let mut face_reader = ply_file.element_reader("face").unwrap();
    let mut all_faces = Vec::new();

    while let Some(chunk) = face_reader.next_chunk::<Face>(&mut ply_file).unwrap() {
        all_faces.extend(chunk);
    }

    assert_eq!(all_faces.len(), 2);
    assert_eq!(all_faces[0].vertex_indices, vec![0, 1, 2]);
    assert_eq!(all_faces[1].vertex_indices, vec![0, 1, 2, 3]);
}

#[test]
fn test_different_property_types() {
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
1.5 2.5 3.5 255 128 64
"#;

    let mut ply_file = PlyFile::new();
    ply_file.feed_data(ply_data.as_bytes());

    let mut vertex_reader = ply_file.element_reader("vertex").unwrap();
    let vertices = vertex_reader
        .next_chunk::<ColorVertex>(&mut ply_file)
        .unwrap()
        .unwrap();

    assert_eq!(vertices.len(), 1);
    assert_eq!(
        vertices[0],
        ColorVertex {
            x: 1.5,
            y: 2.5,
            z: 3.5,
            red: 255,
            green: 128,
            blue: 64,
        }
    );
}

#[test]
fn test_byte_by_byte_feeding() {
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

    // Feed byte by byte - extreme case
    for byte in ply_data.bytes() {
        ply_file.feed_data(&[byte]);
    }

    assert!(ply_file.is_header_ready());

    let mut vertex_reader = ply_file.element_reader("vertex").unwrap();
    let vertices = vertex_reader
        .next_chunk::<Vertex>(&mut ply_file)
        .unwrap()
        .unwrap();

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

#[test]
fn test_large_chunk_processing() {
    // Test with a larger number of vertices
    let mut ply_data = String::from(
        r#"ply
format ascii 1.0
element vertex 100
property float x
property float y
property float z
end_header
"#,
    );

    // Add 100 vertices
    for i in 0..100 {
        ply_data.push_str(&format!("{}.0 {}.0 {}.0\n", i, i + 1, i + 2));
    }

    let mut ply_file = PlyFile::new();

    // Feed in medium-sized chunks
    for chunk in ply_data.as_bytes().chunks(50) {
        ply_file.feed_data(chunk);
    }

    let mut vertex_reader = ply_file.element_reader("vertex").unwrap();
    let mut all_vertices = Vec::new();
    let mut chunk_count = 0;

    while let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file).unwrap() {
        chunk_count += 1;
        all_vertices.extend(chunk);
    }

    assert_eq!(all_vertices.len(), 100);
    println!(
        "Processed {} vertices in {} chunks",
        all_vertices.len(),
        chunk_count
    );

    // Verify some vertices
    assert_eq!(
        all_vertices[0],
        Vertex {
            x: 0.0,
            y: 1.0,
            z: 2.0
        }
    );
    assert_eq!(
        all_vertices[99],
        Vertex {
            x: 99.0,
            y: 100.0,
            z: 101.0
        }
    );
}
