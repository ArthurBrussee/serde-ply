# serde_ply

High-performance PLY parser and writer with type-level format specialization.

## Key Features

**Performance**: ~620 MB/s read throughput, 4.8x faster binary vs ASCII writing
**Architecture**: Eliminates all runtime dispatch via type-level specialization
**Memory**: Constant usage regardless of file size, zero intermediate allocations
**Complete Format Support**: Read and write ASCII, binary little-endian, and binary big-endian

## API

### Reading PLY Files

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Vertex { x: f32, y: f32, z: f32 }

#[derive(Deserialize)]  
struct Face { vertex_indices: Vec<u32> }

// Parse header (advances reader)
let mut reader = std::io::BufReader::new(file);
let header = serde_ply::PlyHeader::parse(&mut reader)?;

// Parse elements sequentially (reader advances through data)
let vertices: Vec<Vertex> = serde_ply::parse_elements(&mut reader, &header, "vertex")?;
let faces: Vec<Face> = serde_ply::parse_elements(&mut reader, &header, "face")?;

// Supports Serde field renaming and aliases
#[derive(Deserialize)]
struct FlexibleVertex {
    #[serde(rename = "x")]           // Map PLY "x" to "position_x" 
    position_x: f32,
    #[serde(alias = "y", alias = "pos_y")]  // Accept "y" OR "pos_y" from PLY
    position_y: f32,
    z: f32,                          // Direct mapping
}
```

### Writing PLY Files
```rust
use serde::Serialize;
use serde_ply::{ElementDef, PlyFormat, PlyHeader, PropertyType, ScalarType};

#[derive(Serialize)]
struct Vertex { x: f32, y: f32, z: f32 }

let header = PlyHeader {
    format: PlyFormat::BinaryLittleEndian,
    version: "1.0".to_string(),
    elements: vec![ElementDef {
        name: "vertex".to_string(),
        count: vertices.len(),
        properties: vec![
            PropertyType::Scalar { data_type: ScalarType::Float, name: "x".to_string() },
            PropertyType::Scalar { data_type: ScalarType::Float, name: "y".to_string() },
            PropertyType::Scalar { data_type: ScalarType::Float, name: "z".to_string() },
        ],
    }],
    comments: vec![],
    obj_info: vec![],
};

// Write to bytes (works for all formats)
let ply_bytes = serde_ply::elements_to_bytes(&header, &vertices)?;

// Write to string (ASCII only)
let ply_string = serde_ply::to_string(&header, &vertices)?;

// Write to file
let mut file = File::create("mesh.ply")?;
serde_ply::elements_to_writer(&mut file, &header, &vertices)?;
```

## Implementation

### Core Insight: Advancing Reader Design

The API uses natural reader advancement through PLY data:
1. **Parse header first**: `PlyHeader::parse()` advances reader past header
2. **Sequential element parsing**: Each `parse_elements()` call advances through that element's data
3. **Type-safe parsing**: Validate struct fields match PLY properties (fail fast)
4. **Format specialization**: Create format-specific deserializers per header
5. **Zero runtime dispatch**: When serde calls `deserialize_f32()`, read f32 directly

**Type-Level Specialization Design:**
The library uses Rust's type system to eliminate runtime format checking. Instead of:
```rust
// Runtime dispatch (eliminated)
match format {
    Ascii => parse_ascii_f32(),
    Binary => parse_binary_f32(),
}
```

We use compile-time specialization:
```rust
// Type-level dispatch (current approach) 
AsciiElementDeserializer<R>::deserialize_f32()   // ASCII-only path
BinaryElementDeserializer<R, E>::deserialize_f32() // Binary-only path
```

Format decision happens once per element batch, not per field.

### Performance Optimizations

- **Type-level format specialization**: Zero runtime dispatch - format decision made once per batch
- **byteorder integration**: Direct `reader.read_f32::<E>()` calls  
- **Pre-computed field layout**: Eliminates property lookups
- **Serde visitor pattern**: Zero intermediate allocations

### Performance Results

**Reading (1K vertices) - Latest benchmarks:**
```
  Simple binary:     12.4 µs @ 2.04 GiB/s
  Realistic binary:  36.6 µs @ 709 MiB/s  
  Realistic ASCII:   402 µs @ 98 MiB/s
  Binary advantage:  11x faster
```

**Writing (1K vertices):**
```
  Binary: 158.6 µs @ 6,250 vertices/ms
  ASCII:  766.5 µs @ 1,305 vertices/ms
  Size reduction: 59.9% smaller files
  Speed improvement: 4.8x faster
```

**Type-level specialization improvements:**
- 16% performance boost for realistic binary parsing
- 11-13% improvement across different dataset sizes
- Zero runtime format dispatch overhead

## API Design

**Advancing Reader Benefits:**
- Natural streaming through PLY data without buffering
- Memory efficient - constant usage regardless of file size
- Parse header once, use for multiple element types
- Works with any `Read` implementation (files, network, memory)
- Better error messages with format context

**Current Features:**
- Multi-element files supported with sequential parsing
- All PLY scalar types (char to double) in all formats
- List properties fully supported in ASCII and binary
- Complete round-trip serialization support
- Full Serde field renaming and alias support

## Implementation Details

**Struct Validation**: Before parsing elements, the library validates that struct fields match PLY properties:
- Full support for `#[serde(rename = "...")]` field renaming
- Full support for `#[serde(alias = "...")]` field aliases (multiple names per field)
- PLY property names drive field matching (respects Serde conventions)
- Validates once upfront, not per element
- Clear error messages for field mismatches

**Type-Level Format Specialization**: Zero runtime dispatch on critical paths:
- `AsciiScalarDeserializer::deserialize_*()` → direct token parsing
- `BinaryScalarDeserializer<E>::deserialize_*()` → direct byteorder calls
- `PlySerializer` → format-specific binary/ASCII output

**Complete Binary Support**: All PLY scalar types in both endianness formats:
- Little-endian and big-endian binary formats
- Proper endianness handling via type parameters
- Round-trip validation for all formats

**Memory Efficiency**: 
- Zero intermediate allocations during read/write
- Direct struct population from PLY data
- Constant memory usage regardless of file size

**Benchmarks**: Run `cargo bench --features benchmarks` to measure performance on your system.

Coding style follows grounded engineering principles (see .rules file):
- Concise code over verbose comments
- Type-level solutions over runtime checks
- Measure performance, don't assume
- Integration tests with real PLY files
