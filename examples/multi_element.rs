use serde::Deserialize;
use std::io::{BufReader, Cursor};

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
struct PlyData {
    vertex: Vec<Vertex>,
    face: Vec<Face>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ply_data = r#"ply
format ascii 1.0
comment Example multi-element PLY file
element vertex 4
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
element face 2
property list uchar uint vertex_indices
end_header
0.0 0.0 0.0 255 0 0
1.0 0.0 0.0 0 255 0
1.0 1.0 0.0 0 0 255
0.0 1.0 0.0 255 255 0
3 0 1 2
3 0 2 3
"#;

    println!("=== Native Serde Multi-Element Parsing ===\n");
    let cursor = Cursor::new(ply_data);
    let ply: PlyData = serde_ply::from_reader(BufReader::new(cursor))?;

    println!("Parsed {} vertices:", ply.vertex.len());
    for (i, vertex) in ply.vertex.iter().enumerate() {
        println!(
            "  {}: ({}, {}, {}) RGB({}, {}, {})",
            i, vertex.x, vertex.y, vertex.z, vertex.red, vertex.green, vertex.blue
        );
    }

    println!("\nParsed {} faces:", ply.face.len());
    for (i, face) in ply.face.iter().enumerate() {
        println!("  {}: {:?}", i, face.vertex_indices);
    }

    println!("\nCompleted successfully!");

    Ok(())
}
