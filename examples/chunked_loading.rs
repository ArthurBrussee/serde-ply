use serde::Deserialize;
use serde_ply::{ChunkPlyFile, PlyError};

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
    let vertex_count = 64;

    let header = format!(
        "ply\nformat binary_little_endian 1.0\nelement vertex {vertex_count}\nproperty float x\nproperty float y\nproperty float z\nend_header\n"
    );

    let mut binary_data = Vec::new();
    binary_data.extend_from_slice(header.as_bytes());

    // Add binary vertex data
    let vertices = vec![[1.0f32, 2.0f32, 3.0f32]; vertex_count];
    for vertex in &vertices {
        for &coord in vertex {
            binary_data.extend_from_slice(&coord.to_le_bytes());
        }
    }

    let mut ply_file = ChunkPlyFile::new();
    let chunks: Vec<_> = binary_data.chunks(15).collect();
    let chunk_iter = chunks.iter();

    for chunk in chunk_iter {
        // Feed in a chunk of data.
        ply_file.buffer_mut().extend_from_slice(chunk);
        // Parse a chunk of vertices.
        let chunk = ply_file.next_chunk::<Vertex>()?;
        println!("Decoded chunk with {} vertices", chunk.len());
    }

    Ok(())
}
