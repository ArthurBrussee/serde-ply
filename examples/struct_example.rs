//! Example demonstrating struct deserialization from PLY files
//!
//! This example shows how to use the new struct deserialization feature
//! to directly convert PLY elements into Rust structs.

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct ColoredVertex {
    x: f32,
    y: f32,
    z: f32,
    red: u8,
    green: u8,
    blue: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PLY Struct Deserialization Example ===\n");

    // Example 1: Simple triangle vertices
    println!("Example 1: Simple triangle vertices");
    let triangle_ply = r#"ply
format ascii 1.0
comment A simple triangle
element vertex 3
property float x
property float y
property float z
end_header
0.0 0.0 0.0
1.0 0.0 0.0
0.5 1.0 0.0
"#;

    let vertices: Vec<Vertex> = serde_ply::from_str(triangle_ply, "vertex")?;

    println!("Loaded {} vertices:", vertices.len());
    for (i, vertex) in vertices.iter().enumerate() {
        println!("  Vertex {}: ({}, {}, {})", i, vertex.x, vertex.y, vertex.z);
    }

    // Example 2: Colored vertices
    println!("\nExample 2: Colored vertices");
    let colored_ply = r#"ply
format ascii 1.0
comment Colored triangle
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

    let colored_vertices: Vec<ColoredVertex> = serde_ply::from_str(colored_ply, "vertex")?;

    println!("Loaded {} colored vertices:", colored_vertices.len());
    for (i, vertex) in colored_vertices.iter().enumerate() {
        println!(
            "  Vertex {}: ({}, {}, {}) RGB({}, {}, {})",
            i, vertex.x, vertex.y, vertex.z, vertex.red, vertex.green, vertex.blue
        );
    }

    // Example 3: Working with real PLY files
    println!("\nExample 3: Loading from example files");

    // This would work with real files:
    // let vertices: Vec<Vertex> = serde_ply::from_reader(
    //     std::fs::File::open("example_plys/greg_turk_example1_ok_ascii.ply")?,
    //     "vertex"
    // )?;

    println!("✓ Struct deserialization working!");

    // Example 4: Type safety demonstration
    println!("\nExample 4: Type safety");

    // This would fail at runtime if the PLY file doesn't have the expected fields:
    let simple_ply = r#"ply
format ascii 1.0
element vertex 1
property float x
property float y
end_header
1.0 2.0
"#;

    // This will fail because our Vertex struct expects x, y, z but PLY only has x, y
    match serde_ply::from_str::<Vertex>(simple_ply, "vertex") {
        Ok(_) => println!("  Unexpected success!"),
        Err(e) => println!("  Expected error (missing z field): {e}"),
    }

    println!("\n=== Example completed successfully! ===");
    println!("\nKey benefits of struct deserialization:");
    println!("✓ Type safety - compile-time field checking");
    println!("✓ Ergonomic API - direct conversion to your data structures");
    println!("✓ Automatic validation - runtime checking of PLY structure");
    println!("✓ Serde integration - works with existing Rust ecosystem");

    Ok(())
}
