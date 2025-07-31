//! Chunked parsing for large files using PlyFile API

use serde::Deserialize;
use serde_ply::{PlyError, PlyFile};

#[derive(Deserialize, Debug)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

fn main() -> Result<(), PlyError> {
    println!("=== Chunked PLY Parsing ===\n");

    demo_basic_chunked()?;
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
    let mut vertex_reader = ply_file.element_reader()?;
    let mut total = 0;

    while let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file)? {
        total += chunk.len();
        println!("Got {} vertices, total: {}", chunk.len(), total);
    }

    println!("Completed: {} vertices\n", total);
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
    while !ply_file.is_header_ready() {
        if let Some(chunk) = chunk_iter.next() {
            ply_file.feed_data(chunk);
        } else {
            break;
        }
    }

    println!("Header parsed, processing vertices...");

    // Process vertices with interleaved feeding
    let mut vertex_reader = ply_file.element_reader()?;
    let mut total = 0;

    loop {
        if let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file)? {
            total += chunk.len();
            println!("Processed {} vertices (total: {})", chunk.len(), total);
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

    println!("Completed: {} vertices total\n", total);
    Ok(())
}
