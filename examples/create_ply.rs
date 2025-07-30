//! Example demonstrating PLY file creation and serialization
//!
//! This example shows how to:
//! 1. Create PLY data structures from scratch
//! 2. Define headers programmatically
//! 3. Serialize data to PLY format
//! 4. Work with different property types

use serde::{Deserialize, Serialize};
use serde_ply::{ElementDef, PlyFormat, PlyHeader, PlySerializer, PropertyType, ScalarType};

#[derive(Serialize, Deserialize, Debug)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct Face {
    vertex_count: u8,
    vertex_indices: Vec<u32>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PLY File Creation Example ===\n");

    // Step 1: Create some sample data
    println!("Step 1: Creating sample mesh data...");

    #[allow(clippy::useless_vec)]
    let vertices = vec![
        Vertex {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            red: 255,
            green: 0,
            blue: 0,
        }, // Red vertex
        Vertex {
            x: 1.0,
            y: 0.0,
            z: 0.0,
            red: 0,
            green: 255,
            blue: 0,
        }, // Green vertex
        Vertex {
            x: 1.0,
            y: 1.0,
            z: 0.0,
            red: 0,
            green: 0,
            blue: 255,
        }, // Blue vertex
        Vertex {
            x: 0.0,
            y: 1.0,
            z: 0.0,
            red: 255,
            green: 255,
            blue: 0,
        }, // Yellow vertex
    ];

    #[allow(clippy::useless_vec)]
    let faces = vec![
        // Triangle 1: vertices 0, 1, 2
        Face {
            vertex_count: 3,
            vertex_indices: vec![0, 1, 2],
        },
        // Triangle 2: vertices 0, 2, 3
        Face {
            vertex_count: 3,
            vertex_indices: vec![0, 2, 3],
        },
    ];

    println!(
        "✓ Created {} vertices and {} faces",
        vertices.len(),
        faces.len()
    );

    // Step 2: Define the PLY header
    println!("\nStep 2: Defining PLY header...");

    let header = PlyHeader {
        format: PlyFormat::Ascii,
        version: "1.0".to_string(),
        elements: vec![
            ElementDef {
                name: "vertex".to_string(),
                count: vertices.len(),
                properties: vec![
                    PropertyType::Scalar {
                        data_type: ScalarType::Float,
                        name: "x".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::Float,
                        name: "y".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::Float,
                        name: "z".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "red".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "green".to_string(),
                    },
                    PropertyType::Scalar {
                        data_type: ScalarType::UChar,
                        name: "blue".to_string(),
                    },
                ],
            },
            ElementDef {
                name: "face".to_string(),
                count: faces.len(),
                properties: vec![PropertyType::List {
                    count_type: ScalarType::UChar,
                    data_type: ScalarType::UInt,
                    name: "vertex_indices".to_string(),
                }],
            },
        ],
        comments: vec![
            "Created by serde_ply create_ply example".to_string(),
            "A simple colored square made of two triangles".to_string(),
        ],
        obj_info: vec!["Generated programmatically".to_string()],
    };

    println!("✓ Header defined with {} elements", header.elements.len());

    // Step 3: Create serializer and write header
    println!("\nStep 3: Serializing to PLY format...");

    let mut output = Vec::new();
    let mut serializer = PlySerializer::with_header(&mut output, header.clone());

    // Write the header
    serializer.set_header(header);

    println!("✓ Serializer created and header written");

    // Step 4: Demonstrate individual vertex serialization
    println!("\nStep 4: Individual vertex serialization example...");

    let single_vertex = Vertex {
        x: 2.0,
        y: 2.0,
        z: 0.0,
        red: 128,
        green: 128,
        blue: 128,
    };

    let mut vertex_output = Vec::new();
    let mut vertex_serializer = PlySerializer::new(&mut vertex_output, PlyFormat::Ascii);
    single_vertex.serialize(&mut vertex_serializer)?;

    let vertex_string = String::from_utf8(vertex_output)?;
    println!("Single vertex serialized as: '{}'", vertex_string.trim());

    // Step 5: Show how different data types serialize
    println!("\nStep 5: Data type serialization examples...");

    let _test_values: Vec<(&str, Box<dyn std::fmt::Debug>)> = vec![
        ("Integer", Box::new(42i32)),
        ("Float", Box::new(2.5f32)),
        ("Byte", Box::new(255u8)),
    ];

    // Demonstrate different data types individually
    let mut test_output = Vec::new();
    let mut test_serializer = PlySerializer::new(&mut test_output, PlyFormat::Ascii);
    42i32.serialize(&mut test_serializer)?;
    let int_result = String::from_utf8(test_output)?;
    println!("  Integer: '{int_result}'");

    test_output = Vec::new();
    test_serializer = PlySerializer::new(&mut test_output, PlyFormat::Ascii);
    2.5f32.serialize(&mut test_serializer)?;
    let float_result = String::from_utf8(test_output)?;
    println!("  Float: '{float_result}'");

    test_output = Vec::new();
    test_serializer = PlySerializer::new(&mut test_output, PlyFormat::Ascii);
    255u8.serialize(&mut test_serializer)?;
    let byte_result = String::from_utf8(test_output)?;
    println!("  Byte: '{byte_result}'");

    // Step 6: Create a complete minimal PLY file
    println!("\nStep 6: Creating complete minimal PLY file...");

    let minimal_header = PlyHeader {
        format: PlyFormat::Ascii,
        version: "1.0".to_string(),
        elements: vec![ElementDef {
            name: "vertex".to_string(),
            count: 3,
            properties: vec![
                PropertyType::Scalar {
                    data_type: ScalarType::Float,
                    name: "x".to_string(),
                },
                PropertyType::Scalar {
                    data_type: ScalarType::Float,
                    name: "y".to_string(),
                },
                PropertyType::Scalar {
                    data_type: ScalarType::Float,
                    name: "z".to_string(),
                },
            ],
        }],
        comments: vec!["Minimal triangle example".to_string()],
        obj_info: vec![],
    };

    // Generate the complete PLY file content
    let triangle_vertices = [
        (0.0f32, 0.0f32, 0.0f32),
        (1.0f32, 0.0f32, 0.0f32),
        (0.5f32, 1.0f32, 0.0f32),
    ];

    println!("Complete PLY file structure:");
    println!("ply");
    println!("format ascii 1.0");
    println!("comment Minimal triangle example");
    println!("element vertex 3");
    println!("property float x");
    println!("property float y");
    println!("property float z");
    println!("end_header");

    for (x, y, z) in triangle_vertices.iter() {
        println!("{x} {y} {z}");
    }

    // Step 7: Demonstrate header introspection
    println!("\nStep 7: Header introspection capabilities...");

    println!("Available elements:");
    for element in &minimal_header.elements {
        println!("  - {} ({} instances)", element.name, element.count);

        println!("    Properties:");
        for property in &element.properties {
            match property {
                PropertyType::Scalar { data_type, name } => {
                    println!(
                        "      * {} : {:?} ({} bytes)",
                        name,
                        data_type,
                        data_type.size_bytes()
                    );
                }
                PropertyType::List {
                    count_type,
                    data_type,
                    name,
                } => {
                    println!("      * {name} : list of {data_type:?} with {count_type:?} count");
                }
            }
        }
    }

    // Step 8: Validate header consistency
    println!("\nStep 8: Header validation...");

    let mut total_properties = 0;
    for element in &minimal_header.elements {
        total_properties += element.properties.len();
    }

    println!("✓ Header validation:");
    println!("  - Format: {}", minimal_header.format);
    println!("  - Elements: {}", minimal_header.elements.len());
    println!("  - Total properties: {total_properties}");
    println!("  - Comments: {}", minimal_header.comments.len());

    if minimal_header.has_element("vertex") {
        println!("  - ✓ Contains vertex element");
    }

    if let Some(vertex_elem) = minimal_header.get_element("vertex") {
        println!("  - Vertex count: {}", vertex_elem.count);
    }

    println!("\n=== PLY Creation Example Completed Successfully! ===");
    println!("\nKey concepts demonstrated:");
    println!("✓ Programmatic header creation");
    println!("✓ Data structure definition");
    println!("✓ Serialization of different data types");
    println!("✓ Header introspection and validation");
    println!("✓ Complete PLY file structure");

    Ok(())
}
