//! Chunked PLY loading with interleaved data feeding and processing

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

fn main() -> Result<(), PlyError> {
    println!("=== Interleaved Chunked PLY Loading ===\n");

    let ply_data = r#"ply
format ascii 1.0
element vertex 4
property float x
property float y
property float z
element face 2
property list uchar int vertex_indices
end_header
0.0 0.0 0.0
1.0 0.0 0.0
0.5 1.0 0.0
0.0 1.0 1.0
3 0 1 2
3 1 2 3
"#;

    // Simulate network chunks
    let chunks: Vec<&[u8]> = ply_data.as_bytes().chunks(25).collect();
    println!("Processing {} chunks...\n", chunks.len());

    let mut ply_file = PlyFile::new();
    let mut chunk_iter = chunks.iter();

    // Feed chunks until header is ready
    while !ply_file.is_header_ready() {
        if let Some(chunk) = chunk_iter.next() {
            println!("Feeding chunk ({} bytes) for header...", chunk.len());
            ply_file.feed_data(chunk);
        } else {
            break;
        }
    }

    if let Some(header) = ply_file.header() {
        println!("Header ready! Format: {}", header.format);
        for element in &header.elements {
            println!("  Element: {} (count: {})", element.name, element.count);
        }
        println!();
    }

    // Process vertices with interleaved data feeding
    let mut vertex_reader = ply_file.element_reader()?;
    let mut all_vertices = Vec::new();

    loop {
        // Try to parse available vertices
        if let Some(vertex_chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file)? {
            println!("Parsed {} vertices", vertex_chunk.len());
            all_vertices.extend(vertex_chunk);
        }

        // Check if we need more data
        if vertex_reader.is_finished() {
            println!(
                "Finished parsing vertices (total: {})\n",
                all_vertices.len()
            );
            break;
        }

        // Feed more data if available
        if let Some(chunk) = chunk_iter.next() {
            println!("Feeding chunk ({} bytes) for vertices...", chunk.len());
            ply_file.feed_data(chunk);
        } else {
            println!("No more data available");
            break;
        }
    }

    // Advance to faces
    ply_file.advance_to_next_element()?;
    let mut face_reader = ply_file.element_reader()?;
    let mut all_faces = Vec::new();

    loop {
        // Try to parse available faces
        if let Some(face_chunk) = face_reader.next_chunk::<Face>(&mut ply_file)? {
            println!("Parsed {} faces", face_chunk.len());
            all_faces.extend(face_chunk);
        }

        if face_reader.is_finished() {
            println!("Finished parsing faces (total: {})\n", all_faces.len());
            break;
        }

        // Feed remaining data
        if let Some(chunk) = chunk_iter.next() {
            println!("Feeding chunk ({} bytes) for faces...", chunk.len());
            ply_file.feed_data(chunk);
        } else {
            break;
        }
    }

    // Show results
    println!("=== Results ===");
    for (i, vertex) in all_vertices.iter().enumerate() {
        println!("Vertex {}: {:?}", i, vertex);
    }
    for (i, face) in all_faces.iter().enumerate() {
        println!("Face {}: {:?}", i, face);
    }

    // Demonstrate binary format
    println!("\n=== Binary Format Demo ===");
    demonstrate_binary_chunked()?;

    Ok(())
}

fn demonstrate_binary_chunked() -> Result<(), PlyError> {
    let header = "ply\nformat binary_little_endian 1.0\nelement vertex 2\nproperty float x\nproperty float y\nproperty float z\nend_header\n";

    let mut binary_data = Vec::new();
    binary_data.extend_from_slice(header.as_bytes());

    // Add binary vertex data
    let vertices = [[1.0f32, 2.0f32, 3.0f32], [4.0f32, 5.0f32, 6.0f32]];
    for vertex in &vertices {
        for &coord in vertex {
            binary_data.extend_from_slice(&coord.to_le_bytes());
        }
    }

    let mut ply_file = PlyFile::new();
    let chunks: Vec<&[u8]> = binary_data.chunks(15).collect();
    let mut chunk_iter = chunks.iter();

    // Header parsing
    while !ply_file.is_header_ready() {
        if let Some(chunk) = chunk_iter.next() {
            ply_file.feed_data(chunk);
        } else {
            break;
        }
    }

    // Interleaved vertex parsing
    let mut vertex_reader = ply_file.element_reader()?;
    loop {
        if let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file)? {
            for vertex in chunk {
                println!("Binary vertex: {:?}", vertex);
            }
        }

        if vertex_reader.is_finished() {
            break;
        }

        if let Some(chunk) = chunk_iter.next() {
            ply_file.feed_data(chunk);
        } else {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interleaved_feeding() {
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
        let chunks: Vec<&[u8]> = ply_data.as_bytes().chunks(10).collect();
        let mut chunk_iter = chunks.iter();

        // Feed until header ready
        while !ply_file.is_header_ready() {
            if let Some(chunk) = chunk_iter.next() {
                ply_file.feed_data(chunk);
            } else {
                break;
            }
        }

        assert!(ply_file.is_header_ready());

        // Interleaved vertex parsing
        let mut vertex_reader = ply_file.element_reader().unwrap();
        let mut all_vertices = Vec::new();

        loop {
            if let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file).unwrap() {
                all_vertices.extend(chunk);
            }

            if vertex_reader.is_finished() {
                break;
            }

            if let Some(chunk) = chunk_iter.next() {
                ply_file.feed_data(chunk);
            } else {
                break;
            }
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
    }
}
