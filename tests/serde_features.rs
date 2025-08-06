use serde::{Deserialize, Deserializer};
use std::io::{BufReader, Cursor};

#[derive(Deserialize, Debug, PartialEq)]
struct FlexibleVertex {
    #[serde(rename = "x")]
    position_x: f32,
    #[serde(alias = "y", alias = "pos_y")]
    position_y: f32,
    z: f32,
    #[serde(deserialize_with = "u8_to_normalized")]
    red: f32,
    #[serde(deserialize_with = "u8_to_normalized")]
    green: f32,
    #[serde(deserialize_with = "u8_to_normalized")]
    blue: f32,
    #[serde(default)]
    confidence: f32,
    normal_x: Option<f32>,
    #[serde(skip)]
    computed: String,
}

impl Default for FlexibleVertex {
    fn default() -> Self {
        Self {
            position_x: 0.0,
            position_y: 0.0,
            z: 0.0,
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            confidence: 1.0,
            normal_x: None,
            computed: "computed".to_string(),
        }
    }
}

fn u8_to_normalized<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let val: u8 = u8::deserialize(deserializer)?;
    Ok(val as f32 / 255.0)
}

#[test]
fn test_field_renaming() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
end_header
1.0 2.0 3.0 255 128 64
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<FlexibleVertex> = serde_ply::parse_elements(&mut reader, &header).unwrap();

    assert_eq!(vertices.len(), 1);
    assert_eq!(vertices[0].position_x, 1.0);
    assert_eq!(vertices[0].position_y, 2.0);
    assert!((vertices[0].red - 1.0).abs() < 0.001);
    assert!((vertices[0].green - 0.502).abs() < 0.001);
}

#[test]
fn test_field_aliases() {
    let test_cases = [
        // Standard names
        r#"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
end_header
1.0 2.0 3.0 255 128 64
"#,
        // Alternative naming
        r#"ply
format ascii 1.0
element vertex 1
property float x
property float pos_y
property float z
property uchar red
property uchar green
property uchar blue
end_header
1.0 2.0 3.0 255 128 64
"#,
    ];

    for (i, ply_data) in test_cases.iter().enumerate() {
        let cursor = Cursor::new(ply_data);
        let mut reader = BufReader::new(cursor);
        let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();

        let vertices: Vec<FlexibleVertex> =
            serde_ply::parse_elements(&mut reader, &header).unwrap();

        assert_eq!(vertices.len(), 1, "Test case {i}");
        assert_eq!(vertices[0].position_x, 1.0, "Test case {i}");
        assert_eq!(vertices[0].position_y, 2.0, "Test case {i}");
    }
}

#[test]
fn test_optional_fields() {
    let ply_data = r#"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
property float normal_x
end_header
1.0 2.0 3.0 255 128 64 0.707
4.0 5.0 6.0 200 100 50 0.0
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<FlexibleVertex> = serde_ply::parse_elements(&mut reader, &header).unwrap();

    assert_eq!(vertices.len(), 2);
    assert_eq!(vertices[0].normal_x, Some(0.707));
    assert_eq!(vertices[1].normal_x, Some(0.0));
}

#[test]
fn test_default_fields() {
    // Test that fields not present in PLY get default values
    #[derive(Deserialize, Debug)]
    #[allow(unused)]
    struct SimpleVertex {
        x: f32,
        y: f32,
        z: f32,
        #[serde(default)]
        confidence: f32,
    }

    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
end_header
1.0 2.0 3.0
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<SimpleVertex> = serde_ply::parse_elements(&mut reader, &header).unwrap();

    assert_eq!(vertices.len(), 1);
    assert_eq!(vertices[0].confidence, 0.0); // default f32 value
}

#[test]
fn test_transparent_wrappers() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(transparent)]
    struct VertexId(u32);

    #[derive(Deserialize, Debug)]
    #[allow(unused)]
    struct IndexedVertex {
        id: VertexId,
        x: f32,
        y: f32,
        z: f32,
    }

    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property uint id
property float x
property float y
property float z
end_header
42 1.0 2.0 3.0
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<IndexedVertex> = serde_ply::parse_elements(&mut reader, &header).unwrap();

    assert_eq!(vertices.len(), 1);
    assert_eq!(vertices[0].id, VertexId(42));
}

#[test]
fn test_custom_list_conversion() {
    #[derive(Deserialize, Debug)]
    #[allow(unused)]
    struct VertexWithNormalizedIndices {
        x: f32,
        y: f32,
        z: f32,
        #[serde(deserialize_with = "indices_to_normalized")]
        indices: Vec<f32>,
    }

    fn indices_to_normalized<'de, D>(deserializer: D) -> Result<Vec<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let indices: Vec<u32> = Vec::deserialize(deserializer)?;
        Ok(indices.into_iter().map(|i| i as f32 / 100.0).collect())
    }

    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
property list uchar uint indices
end_header
1.0 2.0 3.0 3 100 200 300
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<VertexWithNormalizedIndices> =
        serde_ply::parse_elements(&mut reader, &header).unwrap();

    assert_eq!(vertices.len(), 1);
    assert_eq!(vertices[0].indices, vec![1.0, 2.0, 3.0]);
}

#[test]
fn test_field_order_independence() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct ReorderedVertex {
        z: f32,
        x: f32,
        y: f32,
        blue: u8,
        red: u8,
        green: u8,
    }

    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
end_header
1.0 2.0 3.0 255 128 64
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<ReorderedVertex> = serde_ply::parse_elements(&mut reader, &header).unwrap();

    assert_eq!(vertices.len(), 1);
    assert_eq!(
        vertices[0],
        ReorderedVertex {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            red: 255,
            green: 128,
            blue: 64,
        }
    );
}

#[test]
fn test_struct_validation_errors() {
    #[derive(Deserialize, Debug)]
    #[allow(unused)]
    struct MismatchedVertex {
        x: f32,
        y: f32,
        z: f32,
        missing_field: f32,
    }

    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
end_header
1.0 2.0 3.0
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();

    let result = serde_ply::parse_elements::<MismatchedVertex>(&mut reader, &header);
    assert!(result.is_err());
}

#[test]
fn test_serde_flatten_support() {
    use std::collections::HashMap;

    #[derive(Deserialize, Debug, PartialEq)]
    struct VertexWithFlatten {
        x: f32,
        y: f32,
        z: f32,
        #[serde(flatten)]
        extra: HashMap<String, f32>,
    }

    let ply_data = r#"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
property float val_0
property float val_1
property float confidence
end_header
1.0 2.0 3.0 10.0 20.0 0.95
"#;

    let cursor = Cursor::new(ply_data);
    let mut reader = BufReader::new(cursor);
    let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<VertexWithFlatten> = serde_ply::parse_elements(&mut reader, &header).unwrap();

    assert_eq!(vertices.len(), 1);
    let vertex = &vertices[0];
    assert_eq!(vertex.x, 1.0);
    assert_eq!(vertex.y, 2.0);
    assert_eq!(vertex.z, 3.0);

    // Check flattened fields
    assert_eq!(vertex.extra.get("val_0"), Some(&10.0));
    assert_eq!(vertex.extra.get("val_1"), Some(&20.0));
    assert_eq!(vertex.extra.get("confidence"), Some(&0.95));
    assert_eq!(vertex.extra.len(), 3);
}

#[test]
fn test_serde_flatten_support_binary() {
    use std::collections::HashMap;

    #[derive(Deserialize, Debug, PartialEq)]
    struct VertexWithFlatten {
        x: f32,
        y: f32,
        z: f32,
        #[serde(flatten)]
        extra: HashMap<String, f32>,
    }

    let ply_data = r#"ply
format binary_little_endian 1.0
element vertex 1
property float x
property float y
property float z
property float val_0
property float val_1
property float confidence
end_header
"#;

    // Binary data: x=1.0, y=2.0, z=3.0, val_0=10.0, val_1=20.0, confidence=0.95
    let mut binary_data = Vec::new();
    binary_data.extend_from_slice(&1.0f32.to_le_bytes()); // x
    binary_data.extend_from_slice(&2.0f32.to_le_bytes()); // y
    binary_data.extend_from_slice(&3.0f32.to_le_bytes()); // z
    binary_data.extend_from_slice(&10.0f32.to_le_bytes()); // val_0
    binary_data.extend_from_slice(&20.0f32.to_le_bytes()); // val_1
    binary_data.extend_from_slice(&0.95f32.to_le_bytes()); // confidence

    let mut ply_with_binary = ply_data.as_bytes().to_vec();
    ply_with_binary.extend_from_slice(&binary_data);

    let cursor = Cursor::new(ply_with_binary);
    let mut reader = BufReader::new(cursor);
    let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();

    let vertices: Vec<VertexWithFlatten> = serde_ply::parse_elements(&mut reader, &header).unwrap();

    assert_eq!(vertices.len(), 1);
    let vertex = &vertices[0];
    assert_eq!(vertex.x, 1.0);
    assert_eq!(vertex.y, 2.0);
    assert_eq!(vertex.z, 3.0);

    // Check flattened fields
    assert_eq!(vertex.extra.get("val_0"), Some(&10.0));
    assert_eq!(vertex.extra.get("val_1"), Some(&20.0));
    assert_eq!(vertex.extra.get("confidence"), Some(&0.95));
    assert_eq!(vertex.extra.len(), 3);
}
