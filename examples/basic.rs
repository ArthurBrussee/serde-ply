use serde::{Deserialize, Serialize};
use serde_ply::PlyFileDeserializer;
use std::io::{BufReader, Cursor};

#[derive(Deserialize, Serialize, Debug)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Deserialize, Serialize, Debug)]
struct Face {
    vertex_indices: Vec<u32>,
}

#[derive(Deserialize, Debug)]
struct PlyData {
    vertex: Vec<Vertex>,
    face: Vec<Face>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sample PLY file with multiple elements
    let ply_data = r#"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
element face 1
property list uchar uint vertex_indices
end_header
0.0 0.0 0.0 255 0 0
1.0 0.0 0.0 0 255 0
0.5 1.0 0.0 0 0 255
3 0 1 2
"#;

    println!("Decode using serde: \n");
    let cursor = Cursor::new(ply_data);
    let reader = BufReader::new(cursor);
    let ply: PlyData = serde_ply::from_reader(reader)?;

    println!(
        "Parsed {} vertices and {} faces",
        ply.vertex.len(),
        ply.face.len()
    );

    println!("\nParse with header element by element: \n");
    let cursor = Cursor::new(ply_data);
    let mut file = PlyFileDeserializer::from_reader(BufReader::new(cursor))?;
    let vertices: Vec<Vertex> = file.next_element()?;
    let faces: Vec<Face> = file.next_element()?;

    println!(
        "Parsed {} vertices and {} faces",
        vertices.len(),
        faces.len()
    );

    Ok(())
}
