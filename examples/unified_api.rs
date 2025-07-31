//! Example showing the unified PlyFile API

use serde::{Deserialize, Serialize};
use serde_ply::{PlyError, PlyFile};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct ColorVertex {
    x: f32,
    y: f32,
    z: f32,
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Deserialize, Debug)]
struct Face {
    vertex_indices: Vec<u32>,
}

fn main() -> Result<(), PlyError> {
    println!("=== Unified PLY File API Examples ===\n");

    example_complete_data()?;
    example_chunked_loading()?;
    example_multi_element()?;

    Ok(())
}

fn example_complete_data() -> Result<(), PlyError> {
    println!("--- Loading from complete data ---");

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
0.0 0.0 0.0 255 0 0
1.0 1.0 1.0 0 255 0
"#;

    let mut ply_file = PlyFile::new();
    ply_file.feed_data(ply_data.as_bytes());

    let mut vertex_reader = ply_file.element_reader()?;
    let mut vertices = Vec::new();

    while let Some(chunk) = vertex_reader.next_chunk::<ColorVertex>(&mut ply_file)? {
        vertices.extend(chunk);
    }

    println!("Loaded {} vertices:", vertices.len());
    for vertex in &vertices {
        println!("  {:?}", vertex);
    }
    println!();

    Ok(())
}

fn example_chunked_loading() -> Result<(), PlyError> {
    println!("--- Chunked loading simulation ---");

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
0.0 1.0 0.0 0 0 255
"#;

    let mut ply_file = PlyFile::new();
    let chunks: Vec<&[u8]> = ply_data.as_bytes().chunks(30).collect();
    let mut chunk_iter = chunks.iter();

    // Feed chunks until header is ready
    while !ply_file.is_header_ready() {
        if let Some(chunk) = chunk_iter.next() {
            println!("Feeding header chunk ({} bytes)", chunk.len());
            ply_file.feed_data(chunk);
        } else {
            break;
        }
    }

    // Process vertices with interleaved feeding
    let mut vertex_reader = ply_file.element_reader()?;
    let mut vertices = Vec::new();

    loop {
        if let Some(chunk) = vertex_reader.next_chunk::<ColorVertex>(&mut ply_file)? {
            println!("Parsed {} vertices", chunk.len());
            vertices.extend(chunk);
        }

        if vertex_reader.is_finished() {
            break;
        }

        if let Some(chunk) = chunk_iter.next() {
            println!("Feeding data chunk ({} bytes)", chunk.len());
            ply_file.feed_data(chunk);
        } else {
            break;
        }
    }

    println!("Total vertices loaded: {}\n", vertices.len());
    Ok(())
}

fn example_multi_element() -> Result<(), PlyError> {
    println!("--- Multi-element file ---");

    let ply_data = r#"ply
format ascii 1.0
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

    let mut ply_file = PlyFile::new();
    ply_file.feed_data(ply_data.as_bytes());

    // Parse vertices
    let mut vertex_reader = ply_file.element_reader()?;
    let mut vertices = Vec::new();

    while let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file)? {
        vertices.extend(chunk);
    }

    println!("Vertices: {}", vertices.len());
    for (i, vertex) in vertices.iter().enumerate() {
        println!("  {}: ({}, {}, {})", i, vertex.x, vertex.y, vertex.z);
    }

    // Advance to faces
    ply_file.advance_to_next_element()?;
    let mut face_reader = ply_file.element_reader()?;
    let mut faces = Vec::new();

    while let Some(chunk) = face_reader.next_chunk::<Face>(&mut ply_file)? {
        faces.extend(chunk);
    }

    println!("Faces: {}", faces.len());
    for (i, face) in faces.iter().enumerate() {
        println!("  {}: {:?}", i, face.vertex_indices);
    }
    println!();

    Ok(())
}
