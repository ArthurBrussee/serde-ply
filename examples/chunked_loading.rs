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
    println!("=== Chunked PLY Parsing ===\n");
    demo_basic_chunked()?;
    demonstrate_binary_chunked()?;
    demo_large_file_simulation()?;
    Ok(())
}

fn demo_basic_chunked() -> Result<(), PlyError> {
    println!("--- Basic chunked parsing ---");

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

    // Feed in small chunks
    for chunk in ply_data.as_bytes().chunks(15) {
        ply_file.feed_data(chunk);
    }

    // Parse all vertices
    let mut total = 0;

    while let Some(chunk) = ply_file.next_chunk::<Vertex>()? {
        total += chunk.len();
        println!("Got {} vertices, total: {}", chunk.len(), total);
    }

    println!("Completed: {total} vertices\n");
    Ok(())
}

fn demo_large_file_simulation() -> Result<(), PlyError> {
    println!("--- Large file simulation ---");

    // Create larger PLY data
    let mut ply_data = String::from(
        r#"ply
format ascii 1.0
element vertex 50
property float x
property float y
property float z
end_header
"#,
    );

    // Add 50 vertices
    for i in 0..50 {
        ply_data.push_str(&format!("{}.0 {}.0 {}.0\n", i, i + 1, i + 2));
    }

    let mut ply_file = PlyFile::new();
    let chunks: Vec<&[u8]> = ply_data.as_bytes().chunks(100).collect();
    let mut chunk_iter = chunks.iter();

    // Process header
    while ply_file.header().is_none() {
        if let Some(chunk) = chunk_iter.next() {
            ply_file.feed_data(chunk);
        } else {
            break;
        }
    }

    println!("Header parsed, processing vertices...");

    // Process vertices with interleaved feeding
    let mut total = 0;

    loop {
        while let Some(chunk) = ply_file.next_chunk::<Vertex>()? {
            total += chunk.len();
            println!("Processed {} vertices (total: {})", chunk.len(), total);
        }

        if let Some(chunk) = chunk_iter.next() {
            ply_file.feed_data(chunk);
        } else {
            break;
        }
    }

    println!("Completed: {total} vertices total\n");
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
    while !{
        let this = &ply_file;
        this.header().is_some()
    } {
        if let Some(chunk) = chunk_iter.next() {
            ply_file.feed_data(chunk);
        } else {
            break;
        }
    }

    for chunk in chunk_iter {
        ply_file.feed_data(chunk);

        while let Some(chunk) = ply_file.next_chunk::<Vertex>()? {
            for vertex in chunk {
                println!("Binary vertex: {vertex:?}");
            }
        }
    }

    Ok(())
}
