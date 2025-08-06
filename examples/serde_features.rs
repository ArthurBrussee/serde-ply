use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
struct Vertex {
    // Field renaming
    #[serde(rename = "x")]
    position_x: f32,

    // Multiple aliases for different PLY dialects
    #[serde(alias = "y", alias = "pos_y")]
    position_y: f32,

    z: f32,

    // Custom conversion: u8 color to normalized f32
    #[serde(deserialize_with = "u8_to_normalized")]
    red: f32,
    #[serde(deserialize_with = "u8_to_normalized")]
    green: f32,
    #[serde(deserialize_with = "u8_to_normalized")]
    blue: f32,

    // Optional field with default
    #[serde(default)]
    confidence: f32,

    // Optional PLY property
    normal_x: Option<f32>,

    // Skip computed fields
    #[serde(skip)]
    #[allow(unused)]
    magnitude: f32,
}

fn u8_to_normalized<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let val: u8 = u8::deserialize(deserializer)?;
    Ok(val as f32 / 255.0)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // PLY with various naming conventions
    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float pos_y
property float z
property uchar red
property uchar green
property uchar blue
property float normal_x
end_header
1.0 2.0 3.0 255 128 64 0.707
4.0 5.0 6.0 200 100 50 -0.707
"#;

    use std::io::{BufReader, Cursor};
    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = serde_ply::PlyHeader::parse(&mut reader)?;

    let vertices: Vec<Vertex> = serde_ply::parse_elements(&mut reader, &header)?;

    for (i, vertex) in vertices.iter().enumerate() {
        println!(
            "Vertex {}: position=({}, {}, {})",
            i, vertex.position_x, vertex.position_y, vertex.z
        );
        println!(
            "  color=({:.3}, {:.3}, {:.3})",
            vertex.red, vertex.green, vertex.blue
        );
        println!(
            "  normal_x={:?}, confidence={}",
            vertex.normal_x, vertex.confidence
        );
    }

    Ok(())
}
