//! Chunked parsing for large files using unified PlyFile API

use serde::Deserialize;
use serde_ply::{PlyConstruct, PlyFile};

#[derive(Deserialize, Debug)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Chunked PLY Parsing Examples ===\n");

    // Example 1: Build from chunks
    demo_chunked_construction()?;

    // Example 2: Streaming element reader
    demo_streaming_elements()?;

    Ok(())
}

fn demo_chunked_construction() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Building PLY from chunks ---");

    let ply_data = r#"ply
format ascii 1.0
element vertex 100
property float x
property float y
property float z
end_header
"#
    .to_string()
        + &(0..100)
            .map(|i| format!("{}.0 {}.0 {}.0\n", i, i + 1, i + 2))
            .collect::<String>();

    // Simulate receiving data in small chunks
    let mut construct = PlyConstruct::new();
    let bytes = ply_data.as_bytes();

    for (i, chunk) in bytes.chunks(256).enumerate() {
        construct.add_chunk(chunk)?;

        if construct.is_header_complete() {
            println!("Header complete after chunk {}", i + 1);
        }
    }

    // Finalize and read all elements
    let mut ply_file = construct.finalize()?;
    let vertices: Vec<Vertex> = ply_file.read_elements("vertex")?;

    println!("Loaded {} vertices from chunks\n", vertices.len());
    Ok(())
}

fn demo_streaming_elements() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Streaming element reader ---");

    let ply_data = r#"ply
format ascii 1.0
element vertex 500
property float x
property float y
property float z
end_header
"#
    .to_string()
        + &(0..500)
            .map(|i| format!("{}.0 {}.0 {}.0\n", i, i + 1, i + 2))
            .collect::<String>();

    // Create PLY file from data using construct
    let mut construct = PlyConstruct::new();
    construct.add_chunk(ply_data.as_bytes())?;
    let mut ply_file = construct.finalize()?;
    let mut element_reader = ply_file.element_reader::<Vertex>("vertex")?;

    // Get just the data portion for streaming
    let header_end = ply_data.find("end_header\n").unwrap() + 11;
    let data_bytes = &ply_data.as_bytes()[header_end..];

    let mut total_processed = 0;

    // Process in chunks
    for chunk in data_bytes.chunks(512) {
        let vertices = element_reader.read_chunk(&mut ply_file, chunk)?;

        if !vertices.is_empty() {
            total_processed += vertices.len();
            println!(
                "Chunk: {} vertices (total: {}/{})",
                vertices.len(),
                element_reader.elements_read(),
                element_reader.total_elements()
            );
        }

        if element_reader.is_complete() {
            break;
        }
    }

    println!(
        "Streaming complete: {} vertices processed\n",
        total_processed
    );
    Ok(())
}
