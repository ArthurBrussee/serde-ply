//! Example demonstrating chunked PLY file loading
//!
//! This example shows how to use the PlyFile wrapper to handle chunked data loading,
//! which is useful for async scenarios, streaming data, or large files.

use serde::Deserialize;
use serde_ply::{PlyError, PlyFile};
use std::io::Read;

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

fn main() -> Result<(), PlyError> {
    // Example PLY data as a string (in practice this would come from network/file chunks)
    let ply_data = r#"ply
format ascii 1.0
comment Created by example
element vertex 3
property float x
property float y
property float z
element face 1
property list uchar int vertex_indices
end_header
0.0 0.0 0.0
1.0 0.0 0.0
0.5 1.0 0.0
3 0 1 2
"#;

    println!("=== Chunked PLY Loading Example ===\n");

    // Simulate chunked data loading
    let chunk_size = 50; // Small chunks to demonstrate the chunking
    let chunks: Vec<&[u8]> = ply_data.as_bytes().chunks(chunk_size).collect();

    println!("Simulating {} chunks of data...", chunks.len());

    let mut ply_file = PlyFile::new();

    // Feed chunks one by one
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Processing chunk {}/{}", i + 1, chunks.len());
        ply_file.feed_data(chunk);

        if ply_file.is_header_ready() {
            println!("  Header is ready after chunk {}", i + 1);
            break;
        }
    }

    // Continue feeding remaining chunks if header was parsed early
    for (i, chunk) in chunks
        .iter()
        .enumerate()
        .skip_while(|(i, _)| *i < chunks.len())
    {
        ply_file.feed_data(chunk);
    }

    // Now we can access the header
    if let Some(header) = ply_file.header() {
        println!("\n=== Header Information ===");
        println!("Format: {}", header.format);
        println!("Version: {}", header.version);
        println!("Elements:");
        for element in &header.elements {
            println!("  - {}: {} count", element.name, element.count);
        }
    }

    // Parse vertices in chunks
    println!("\n=== Parsing Vertices ===");
    let mut vertex_reader = ply_file.element_reader("vertex")?;
    let mut all_vertices = Vec::new();

    while let Some(vertex_chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file)? {
        println!("Got vertex chunk with {} vertices", vertex_chunk.len());
        for vertex in &vertex_chunk {
            println!("  Vertex: {:?}", vertex);
        }
        all_vertices.extend(vertex_chunk);
    }

    println!("Total vertices parsed: {}", all_vertices.len());

    // Advance to next element type
    ply_file.advance_to_next_element()?;

    // Parse faces in chunks
    println!("\n=== Parsing Faces ===");
    let mut face_reader = ply_file.element_reader("face")?;
    let mut all_faces = Vec::new();

    while let Some(face_chunk) = face_reader.next_chunk::<Face>(&mut ply_file)? {
        println!("Got face chunk with {} faces", face_chunk.len());
        for face in &face_chunk {
            println!("  Face: {:?}", face);
        }
        all_faces.extend(face_chunk);
    }

    println!("Total faces parsed: {}", all_faces.len());

    // Demonstrate binary format chunked loading
    println!("\n=== Binary Format Example ===");
    demonstrate_binary_chunked_loading()?;

    println!("\n=== Example Complete ===");
    Ok(())
}

fn demonstrate_binary_chunked_loading() -> Result<(), PlyError> {
    use std::io::Write;

    // Create a simple binary PLY in memory
    let header = "ply\nformat binary_little_endian 1.0\nelement vertex 2\nproperty float x\nproperty float y\nproperty float z\nend_header\n";

    let mut binary_data = Vec::new();
    binary_data.extend_from_slice(header.as_bytes());

    // Add binary vertex data (2 vertices, 3 floats each)
    let vertices = [[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]];

    for vertex in &vertices {
        for &coord in vertex {
            binary_data.extend_from_slice(&coord.to_le_bytes());
        }
    }

    // Process in chunks
    let mut ply_file = PlyFile::new();
    let chunk_size = 25;

    for chunk in binary_data.chunks(chunk_size) {
        ply_file.feed_data(chunk);
    }

    if ply_file.is_header_ready() {
        println!("Binary PLY header parsed successfully");

        let mut vertex_reader = ply_file.element_reader("vertex")?;
        while let Some(vertex_chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file)? {
            for vertex in vertex_chunk {
                println!("  Binary vertex: {:?}", vertex);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunked_loading_small_chunks() {
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

        // Feed data byte by byte to test extreme chunking
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
    fn test_multiple_element_types() {
        let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
element face 1
property list uchar int vertex_indices
end_header
0.0 0.0 0.0
1.0 1.0 1.0
3 0 1 2
"#;

        let mut ply_file = PlyFile::new();
        ply_file.feed_data(ply_data.as_bytes());

        // Parse vertices
        let mut vertex_reader = ply_file.element_reader("vertex").unwrap();
        let mut vertex_count = 0;

        while let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file).unwrap() {
            vertex_count += chunk.len();
        }
        assert_eq!(vertex_count, 2);

        // Advance to faces
        ply_file.advance_to_next_element().unwrap();

        let mut face_reader = ply_file.element_reader("face").unwrap();
        let faces = face_reader
            .next_chunk::<Face>(&mut ply_file)
            .unwrap()
            .unwrap();

        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].vertex_indices, vec![0, 1, 2]);
    }
}
