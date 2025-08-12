use serde::{de::DeserializeSeed, Deserialize};
use serde_ply::{ChunkPlyFile, PlyError, RowVisitor};

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

struct Mesh {
    verts: Vec<Vertex>,
    faces: Vec<Face>,
}

fn dummy_data(vertex_count: usize) -> Vec<u8> {
    let header = format!(
        "ply\nformat binary_little_endian 1.0\nelement vertex {vertex_count}\nproperty float x\nproperty float y\nproperty float z\nelement face 1\nproperty list uchar uint vertex_indices\nend_header\n"
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

    // Add face data
    binary_data.extend_from_slice(&[3u8; 1]);
    binary_data.extend_from_slice(&[0u8; 3 * 4]);
    binary_data
}

fn main() -> Result<(), PlyError> {
    let vertex_count = 64;

    let binary_data = dummy_data(vertex_count);

    // We're decoding in chunks but want to make one big array of vertices without allocating smaller sub arrays.
    let mut mesh = Mesh {
        verts: Vec::new(),
        faces: Vec::new(),
    };

    let mut file = ChunkPlyFile::new();
    let chunk_size = 15;

    for chunk in binary_data.chunks(chunk_size) {
        // Feed in some data. This might come from eg. an async source.
        file.buffer_mut().extend_from_slice(chunk);

        if let Some(current_element) = file.current_element() {
            // Now use a 'RowVisitor' with the file deserializer. This is a just a custom deserializer that has a callback
            // for each row, you could also implement a custom deserializer wholesale. ChunkPlyFile deserializes to a sequence of rows
            // currently available. It's your responsibility to deserialize the right element.

            if current_element.name == "vertex" {
                RowVisitor::new(|vertex: Vertex| {
                    mesh.verts.push(vertex);
                })
                .deserialize(&mut file)?;
            } else {
                // Handle face elements
                RowVisitor::new(|face: Face| {
                    mesh.faces.push(face);
                })
                .deserialize(&mut file)?;
            }
        }
    }

    println!("Total vertices deserialized: {}", mesh.verts.len());
    println!("Total faces deserialized: {}", mesh.faces.len());
    Ok(())
}
