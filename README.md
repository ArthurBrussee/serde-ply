# serde-ply

[![Crates.io](https://img.shields.io/crates/v/serde-ply.svg)](https://crates.io/crates/serde-ply)
[![Documentation](https://docs.rs/serde-ply/badge.svg)](https://docs.rs/serde-ply)

Flexible and fast PLY parser and writer using serde. While PLY is an older format, it's still used in various geometry processing applications and research. PLY files act as simple key-value tables, and can be decoded in a streaming manner.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
serde-ply = "0.1"
serde = { version = "1.0", features = ["derive"] }
```

## Features

- Supports serializing and deserializing PLY files
- Supports full PLY specification including list properties
- Supports binary and ASCII formats
- Supports deserializing PLY files in chunks, for streaming data processing
- High performance (1 GB/s+ deserialization)
- Zero-copy where possible
- Full serde integration

## Quick Start

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Deserialize, Serialize)]
struct Mesh {
    vertex: Vec<Vertex>,
}

// Read PLY file
let mesh: Mesh = serde_ply::from_reader(reader)?;

// Write PLY file
let bytes = serde_ply::to_bytes(&mesh, serde_ply::PlyFormat::Ascii, vec![])?;
```

## Examples

Please see the `examples/` folder for comprehensive usage examples including:
- Basic serialization and deserialization
- Chunked loading for large files

## Contributions

Contributions are welcome! Please open an issue or submit a pull request. Please run the tests to check if everything still works, and `cargo bench` to check if performance is still acceptable.
