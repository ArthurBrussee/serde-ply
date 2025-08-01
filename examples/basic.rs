//! Basic PLY parsing and writing

use serde::{Deserialize, Serialize};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read PLY file
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

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = serde_ply::PlyHeader::parse(&mut reader)?;

    let vertices: Vec<Vertex> = serde_ply::PlyFile::parse_elements(&mut reader, &header, "vertex")?;
    let faces: Vec<Face> = serde_ply::PlyFile::parse_elements(&mut reader, &header, "face")?;

    println!(
        "Parsed {} vertices and {} faces",
        vertices.len(),
        faces.len()
    );

    // Write PLY file
    use serde_ply::{ElementDef, PlyFormat, PlyHeader, PropertyType, ScalarType};

    let output_header = PlyHeader {
        format: PlyFormat::Ascii,
        version: "1.0".to_string(),
        elements: vec![ElementDef {
            name: "vertex".to_string(),
            count: vertices.len(),
            properties: vec![
                PropertyType::Scalar {
                    data_type: ScalarType::F32,
                    name: "x".to_string(),
                },
                PropertyType::Scalar {
                    data_type: ScalarType::F32,
                    name: "y".to_string(),
                },
                PropertyType::Scalar {
                    data_type: ScalarType::F32,
                    name: "z".to_string(),
                },
                PropertyType::Scalar {
                    data_type: ScalarType::U8,
                    name: "red".to_string(),
                },
                PropertyType::Scalar {
                    data_type: ScalarType::U8,
                    name: "green".to_string(),
                },
                PropertyType::Scalar {
                    data_type: ScalarType::U8,
                    name: "blue".to_string(),
                },
            ],
        }],
        comments: vec![],
        obj_info: vec![],
    };

    let ply_string = serde_ply::PlyFile::to_string(&output_header, &vertices)?;
    println!("Generated PLY:\n{ply_string}");

    Ok(())
}
