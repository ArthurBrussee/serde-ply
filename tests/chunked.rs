use serde::Deserialize;
use serde_ply::ChunkPlyFile;

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
2.0 0.0 0.0
0.5 1.0 0.0
"#;

    let mut ply_file = ChunkPlyFile::new();
    ply_file.buffer_mut().extend_from_slice(ply_data.as_bytes());
    let vertices: Vec<Vertex> = ply_file.next_chunk().unwrap();
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

    let mut ply_file = ChunkPlyFile::new();
    ply_file.buffer_mut().extend_from_slice(&binary_data);

    let parsed: Vec<Vertex> = ply_file.next_chunk().unwrap();
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
"#;

    let mut ply_file = ChunkPlyFile::new();
    ply_file.buffer_mut().extend_from_slice(ply_data.as_bytes());

    // Should get first vertex
    let chunk1: Vec<Vertex> = ply_file.next_chunk().unwrap();
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
    let chunk2: Vec<Vertex> = ply_file.next_chunk().unwrap();
    assert!(chunk2.is_empty());

    // Add the missing vertex data
    ply_file.buffer_mut().extend_from_slice(b"4.0 5.0 6.0\n");

    // Now should get the second vertex
    let chunk3: Vec<Vertex> = ply_file.next_chunk().unwrap();
    assert_eq!(chunk3.len(), 1);
    assert_eq!(
        chunk3[0],
        Vertex {
            x: 4.0,
            y: 5.0,
            z: 6.0
        }
    );
    assert!(ply_file.next_chunk::<Vertex>().is_err());
}

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

    let mut ply_file = ChunkPlyFile::new();

    // Feed only header + first complete vertex (12 bytes)
    let header_size = header.len();
    let first_vertex_size = 12;
    ply_file
        .buffer_mut()
        .extend_from_slice(&binary_data[..header_size + first_vertex_size]);

    // Should parse exactly one vertex
    let chunk1: Vec<Vertex> = ply_file.next_chunk().unwrap();
    assert_eq!(chunk1.len(), 1);
    assert_eq!(
        chunk1[0],
        Vertex {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );

    // Call again - should return empty list (no complete elements available)
    assert!(ply_file.next_chunk::<Vec<Vertex>>().unwrap().is_empty());

    // Feed partial second vertex (only 8 bytes out of 12)
    ply_file.buffer_mut().extend_from_slice(
        &binary_data[header_size + first_vertex_size..header_size + first_vertex_size + 8],
    );

    // Should still return empty list (incomplete element)
    assert!(ply_file.next_chunk::<Vec<Vertex>>().unwrap().is_empty());

    // Feed remaining data
    ply_file
        .buffer_mut()
        .extend_from_slice(&binary_data[header_size + first_vertex_size + 8..]);

    // Now should get the second vertex
    let chunk2: Vec<Vertex> = ply_file.next_chunk().unwrap();
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

    let mut ply_file = ChunkPlyFile::new();
    ply_file.buffer_mut().extend_from_slice(ply_data.as_bytes());

    // Parse vertices
    let vertices: Vec<Vertex> = ply_file.next_chunk().unwrap();
    assert_eq!(vertices.len(), 2);

    // Parse colors
    let colors: Vec<Color> = ply_file.next_chunk().unwrap();
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

    let mut ply_file = ChunkPlyFile::new();
    ply_file.buffer_mut().extend_from_slice(ply_data.as_bytes());

    let faces: Vec<Face> = ply_file.next_chunk().unwrap();
    assert_eq!(faces.len(), 2);
    assert_eq!(faces[0].vertex_indices, vec![0, 1, 2]);
    assert_eq!(faces[1].vertex_indices, vec![0, 1, 2, 3]);
}

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

    let mut ply_file = ChunkPlyFile::new();

    // Feed data in 3-byte chunks
    let data = ply_data.as_bytes();
    let mut vertices: Vec<Vertex> = vec![];
    for chunk in data.chunks(3) {
        ply_file.buffer_mut().extend_from_slice(chunk);
        vertices.extend(ply_file.next_chunk::<Vec<Vertex>>().unwrap());
    }

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
fn test_zero_elements() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 0
property float x
property float y
property float z
end_header
"#;

    let mut ply_file = ChunkPlyFile::new();
    ply_file.buffer_mut().extend_from_slice(ply_data.as_bytes());
    assert!(ply_file.next_chunk::<Vec<Vertex>>().unwrap().is_empty());
}

#[test]
fn test_binary_incomplete_lists() {
    let header = "ply\nformat binary_little_endian 1.0\nelement face 2\nproperty list uchar int vertex_indices\nend_header\n";

    let mut binary_data = Vec::new();
    binary_data.extend_from_slice(header.as_bytes());

    // Add complete first face: count=3, indices=[0, 1, 2]
    binary_data.push(3u8); // count
    binary_data.extend_from_slice(&0i32.to_le_bytes()); // index 0
    binary_data.extend_from_slice(&1i32.to_le_bytes()); // index 1
    binary_data.extend_from_slice(&2i32.to_le_bytes()); // index 2

    // Add incomplete second face: count=4, but only 2 indices
    binary_data.push(4u8); // count
    binary_data.extend_from_slice(&0i32.to_le_bytes()); // index 0
    binary_data.extend_from_slice(&1i32.to_le_bytes()); // index 1
                                                        // Missing 2 more indices

    let mut ply_file = ChunkPlyFile::new();
    ply_file.buffer_mut().extend_from_slice(&binary_data);

    // Should parse only the first complete face
    let faces1: Vec<Face> = ply_file.next_chunk().unwrap();
    assert_eq!(faces1.len(), 1);
    assert_eq!(faces1[0].vertex_indices, vec![0, 1, 2]);

    // Call again - should return no faces (incomplete second face)
    assert!(ply_file.next_chunk::<Vec<Face>>().unwrap().is_empty());

    // Add missing data for second face
    ply_file.buffer_mut().extend_from_slice(&2i32.to_le_bytes()); // index 2
    ply_file.buffer_mut().extend_from_slice(&3i32.to_le_bytes()); // index 3

    // Now should get the second face
    let faces2 = ply_file.next_chunk::<Vec<Face>>().unwrap();
    assert_eq!(faces2.len(), 1);
    assert_eq!(faces2[0].vertex_indices, vec![0, 1, 2, 3]);
}

#[test]
fn test_binary_lists() {
    let header = "ply\nformat binary_little_endian 1.0\nelement face 2\nproperty list uchar int vertex_indices\nend_header\n";

    let mut binary_data = Vec::new();
    binary_data.extend_from_slice(header.as_bytes());

    // Add face data: [3, 0, 1, 2] and [4, 0, 1, 2, 3]
    // First face: count=3, indices=[0, 1, 2]
    binary_data.push(3u8); // count
    binary_data.extend_from_slice(&0i32.to_le_bytes()); // index 0
    binary_data.extend_from_slice(&1i32.to_le_bytes()); // index 1
    binary_data.extend_from_slice(&2i32.to_le_bytes()); // index 2

    // Second face: count=4, indices=[0, 1, 2, 3]
    binary_data.push(4u8); // count
    binary_data.extend_from_slice(&0i32.to_le_bytes()); // index 0
    binary_data.extend_from_slice(&1i32.to_le_bytes()); // index 1
    binary_data.extend_from_slice(&2i32.to_le_bytes()); // index 2
    binary_data.extend_from_slice(&3i32.to_le_bytes()); // index 3

    let mut ply_file = ChunkPlyFile::new();

    let mut faces = vec![];
    // Feed in small chunks to test list boundary detection
    for chunk in binary_data.chunks(6) {
        ply_file.buffer_mut().extend_from_slice(chunk);
        faces.extend(ply_file.next_chunk::<Vec<Face>>().unwrap());
    }

    assert_eq!(faces.len(), 2);
    assert_eq!(faces[0].vertex_indices, vec![0, 1, 2]);
    assert_eq!(faces[1].vertex_indices, vec![0, 1, 2, 3]);
}
