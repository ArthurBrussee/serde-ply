//! Example showing the new unified PlyFile API

use serde::{Deserialize, Serialize};
use serde_ply::{PlyConstruct, PlyFile};
use std::io::{BufReader, Cursor};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Deserialize, Debug)]
struct Face {
    vertex_indices: Vec<u32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Unified PLY File API Examples ===\n");

    // Example 1: Load from complete data
    example_from_bytes()?;

    // Example 2: Load from reader (streaming)
    example_from_reader()?;

    // Example 3: Build from chunks (async-like)
    example_from_chunks()?;

    // Example 4: Streaming element reader
    example_streaming_reader()?;

    Ok(())
}

fn example_from_bytes() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 1: PlyFile from construct ---");

    let ply_data = r#"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
end_header
0.0 0.0 0.0 255 0 0
1.0 0.0 0.0 0 255 0
0.5 1.0 0.0 0 0 255
"#;

    // Load PLY file from data using construct (no from_bytes)
    let mut construct = PlyConstruct::new();
    construct.add_chunk(ply_data.as_bytes())?;
    let mut ply_file = construct.finalize()?;

    // Read any element type - same API for all
    let vertices: Vec<Vertex> = ply_file.read_elements("vertex")?;

    println!("Loaded {} vertices", vertices.len());
    println!("First vertex: {:?}\n", vertices[0]);

    Ok(())
}

fn example_from_reader() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 2: PlyFile::from_reader ---");

    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
end_header
1.0 2.0 3.0 128 128 128
4.0 5.0 6.0 255 255 255
"#;

    // Load from any reader
    let cursor = Cursor::new(ply_data);
    let reader = BufReader::new(cursor);
    let mut ply_file = PlyFile::from_reader(reader)?;

    println!("Header format: {:?}", ply_file.header().format);

    // Same read_elements API
    let vertices: Vec<Vertex> = ply_file.read_elements("vertex")?;
    println!("Read {} vertices from stream\n", vertices.len());

    Ok(())
}

fn example_from_chunks() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 3: Building from chunks ---");

    let ply_data = r#"ply
format ascii 1.0
element vertex 4
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
end_header
1.0 0.0 0.0 255 0 0
0.0 1.0 0.0 0 255 0
0.0 0.0 1.0 0 0 255
1.0 1.0 1.0 128 128 128
"#;

    // Simulate receiving data in chunks (like from network)
    let mut construct = PlyConstruct::new();

    let bytes = ply_data.as_bytes();
    let chunk_size = 50; // Small chunks

    for (i, chunk) in bytes.chunks(chunk_size).enumerate() {
        construct.add_chunk(chunk)?;

        if construct.is_header_complete() {
            println!("Header complete after chunk {}", i + 1);
        }
    }

    // Finalize and use
    let mut ply_file = construct.finalize()?;
    let vertices: Vec<Vertex> = ply_file.read_elements("vertex")?;

    println!(
        "Built PLY from {} chunks",
        (bytes.len() + chunk_size - 1) / chunk_size
    );
    println!("Final result: {} vertices\n", vertices.len());

    Ok(())
}

fn example_streaming_reader() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 4: Streaming element reader ---");

    // Create a PLY file with many vertices
    let mut ply_data = String::from(
        r#"ply
format ascii 1.0
element vertex 1000
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
end_header
"#,
    );

    // Add vertex data
    for i in 0..1000 {
        let base = i as f32 * 0.01;
        ply_data.push_str(&format!(
            "{} {} {} {} {} {}\n",
            base,
            base + 1.0,
            base + 2.0,
            (i % 256) as u8,
            ((i * 2) % 256) as u8,
            ((i * 3) % 256) as u8
        ));
    }

    // Create PLY file from data using construct
    let mut construct = PlyConstruct::new();
    construct.add_chunk(ply_data.as_bytes())?;
    let mut ply_file = construct.finalize()?;
    let mut element_reader = ply_file.element_reader::<Vertex>("vertex")?;

    // Process in chunks
    let bytes = ply_data.as_bytes();
    let header_end = ply_data.find("end_header\n").unwrap() + 11;
    let data_portion = &bytes[header_end..];

    let mut total_processed = 0;
    // Process in chunks
    for chunk in data_portion.chunks(1024) {
        let vertices = element_reader.read_chunk(&mut ply_file, chunk)?;
        total_processed += vertices.len();

        if !vertices.is_empty() {
            println!(
                "Processed {} vertices (total: {})",
                vertices.len(),
                total_processed
            );
        }

        if element_reader.is_complete() {
            break;
        }
    }

    println!(
        "Streaming complete: {}/{} vertices\n",
        element_reader.elements_read(),
        element_reader.total_elements()
    );

    Ok(())
}
