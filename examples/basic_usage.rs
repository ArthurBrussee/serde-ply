//! Basic usage example demonstrating the advancing reader API

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_ply::PlyHeader;
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

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Face {
    vertex_indices: Vec<u32>,
}

#[derive(Deserialize, Debug)]
struct SimpleVertex {
    x: f32,
    y: f32,
    z: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Multi-element PLY file with vertices and faces
    let ply_data = r#"ply
format ascii 1.0
comment Created by serde_ply example
comment Colored triangle mesh
obj_info author PLY Demo
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
    let header = PlyHeader::parse(&mut reader)?;

    // Parse vertices (reader advances)
    let vertices: Vec<Vertex> = serde_ply::parse_elements(&mut reader, &header, "vertex")?;

    // Parse faces (reader continues from where vertices ended)
    let faces: Vec<Face> = serde_ply::parse_elements(&mut reader, &header, "face")?;

    assert_eq!(vertices.len(), 3);
    assert_eq!(faces.len(), 1);

    // Simple vertex-only file
    let simple_ply = r#"ply
format ascii 1.0
element vertex 4
property float x
property float y
property float z
end_header
-1.0 -1.0 0.0
 1.0 -1.0 0.0
 1.0  1.0 0.0
-1.0  1.0 0.0
"#;

    let cursor = Cursor::new(simple_ply);
    let mut simple_reader = BufReader::new(cursor);
    let simple_header = PlyHeader::parse(&mut simple_reader)?;
    let simple_vertices: Vec<SimpleVertex> =
        serde_ply::parse_elements(&mut simple_reader, &simple_header, "vertex")?;

    assert_eq!(simple_vertices.len(), 4);

    // Binary format handling (in-memory)
    let binary_ply_data = b"ply\nformat binary_little_endian 1.0\nelement vertex 2\nproperty float x\nproperty float y\nproperty float z\nend_header\n\x00\x00\x00\x00\x00\x00\x80\x3f\x00\x00\x00\x00\x00\x00\x00\x40\x00\x00\x80\x3f\x00\x00\x00\x40";
    let cursor = Cursor::new(binary_ply_data);
    let mut binary_reader = BufReader::new(cursor);
    let binary_header = PlyHeader::parse(&mut binary_reader)?;

    let _binary_vertices =
        serde_ply::parse_elements::<_, SimpleVertex>(&mut binary_reader, &binary_header, "vertex");

    // Error handling
    let invalid_ply = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
end_header
1.0 2.0 3.0
4.0 5.0
"#;

    let cursor = Cursor::new(invalid_ply);
    let mut invalid_reader = BufReader::new(cursor);
    let invalid_header = PlyHeader::parse(&mut invalid_reader)?;

    let result = serde_ply::parse_elements::<_, SimpleVertex>(
        &mut invalid_reader,
        &invalid_header,
        "vertex",
    );
    assert!(result.is_err());

    Ok(())
}
