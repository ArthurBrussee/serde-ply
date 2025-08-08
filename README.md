# serde_ply

High-performance PLY parser and writer with type-level format specialization.

## Key Features

**Serde integration**: Supports most serde features
**Performance**: High performant implementation (~1.5 GB/s on an M4) while being compatible with serde features.
**Spec compliant**: Read and write ASCII, binary little-endian, and binary big-endian, list, comments, etc.

## API

### Reading PLY Files

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Vertex { x: f32, y: f32, z: f32 }

#[derive(Deserialize)]
struct Face { vertex_indices: Vec<u32> }

// Multi-element parsing
#[derive(Deserialize)]
struct PlyData {
    vertex: Vec<Vertex>,
    face: Vec<Face>,
}

let mut reader = std::io::BufReader::new(file);
let deserializer = serde_ply::PlyFileDeserializer::from_reader(reader)?;
let ply: PlyData = serde::Deserialize::deserialize(deserializer)?;

// Single-element parsing
let mut reader = std::io::BufReader::new(file);
let header = serde_ply::PlyHeader::parse(&mut reader)?;
let vertices: Vec<Vertex> = serde_ply::parse_elements(&mut reader, &header)?;

// Supports Serde field renaming and aliases
#[derive(Deserialize)]
struct FlexibleVertex {
    #[serde(rename = "x")]           // Map PLY "x" to "position_x"
    position_x: f32,
    #[serde(alias = "y", alias = "pos_y")]  // Accept "y" OR "pos_y" from PLY
    position_y: f32,
    z: f32,                          // Direct mapping
}

// Custom field type conversion
#[derive(Deserialize)]
struct VertexWithConversion {
    x: f32,
    y: f32,
    z: f32,
    #[serde(deserialize_with = "u8_to_normalized_f32")]
    red: f32,    // PLY has u8, we want normalized f32
    #[serde(deserialize_with = "u8_to_normalized_f32")]
    green: f32,  // PLY has u8, we want normalized f32
    #[serde(deserialize_with = "u8_to_normalized_f32")]
    blue: f32,   // PLY has u8, we want normalized f32
}

fn u8_to_normalized_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val: u8 = u8::deserialize(deserializer)?;
    Ok(val as f32 / 255.0)  // Convert 0-255 to 0.0-1.0
}

// Advanced serde features
#[derive(Deserialize)]
struct AdvancedVertex {
    #[serde(rename = "position_x")]
    x: f32,                          // Field renaming
    #[serde(alias = "y", alias = "pos_y")]
    y: f32,                          // Multiple aliases
    z: f32,
    #[serde(deserialize_with = "u8_to_normalized_f32")]
    red: f32,                        // Custom conversion
    #[serde(default)]
    confidence: f32,                 // Default if missing
    #[serde(skip)]
    cached_data: String,             // Skip field entirely
    normal_x: Option<f32>,           // Optional PLY property
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

### Chunked PLY Loading
```rust
use serde_ply::PlyFile;

let mut ply_file = PlyFile::new();
let mut chunk_iter = get_data_chunks(); // Your data source

// Feed chunks until header is ready
while !ply_file.is_header_ready() {
    if let Some(chunk) = chunk_iter.next() {
        ply_file.feed_data(&chunk);
    } else { break; }
}

// Parse elements with interleaved data feeding
let mut vertex_reader = ply_file.element_reader()?;
let mut all_vertices = Vec::new();

loop {
    if let Some(chunk) = vertex_reader.next_chunk::<Vertex>(&mut ply_file)? {
        all_vertices.extend(chunk);
    }

    if vertex_reader.is_finished() { break; }

    if let Some(chunk) = chunk_iter.next() {
        ply_file.feed_data(&chunk);
    } else { break; }
}

// Advance to next element
ply_file.advance_to_next_element()?;
let mut face_reader = ply_file.element_reader()?;

// Continue with faces...
```

### Lower Level Chunked API
```rust
use serde_ply::chunked_header_parser;

let mut header_parser = chunked_header_parser();
loop {
    let chunk = read_chunk().await?;
    if header_parser.parse_from_bytes(&chunk)?.is_some() {
        break;
    }
}

let mut file_parser = header_parser.into_file_parser()?;
loop {
    let chunk = read_chunk().await?;
    file_parser.add_data(&chunk);

    if let Some(vertices) = file_parser.parse_chunk::<Vertex>("vertex")? {
        process_vertices(vertices).await;
    }

    if file_parser.is_element_complete("vertex") {
        file_parser.advance_to_next_element();
    }
}
```

## Implementation

### Core Design

**Sequential Element Processing**: Elements are parsed in order as defined in the header. The `element_reader()` returns a reader for the current element type.

**Chunked Loading**: Feed data of any size, parse elements as they become available. Leftover data is automatically preserved between chunks.

**Type-Level Specialization**: Format is determined once per element batch:
```rust
AsciiElementDeserializer<R>::deserialize_f32()   // Direct ASCII parsing
BinaryElementDeserializer<R, E>::deserialize_f32() // Direct binary parsing
```

No runtime format dispatch on the critical path.

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
- Native Serde multi-element parsing
- All PLY scalar types in all formats
- List properties fully supported
- Complete round-trip serialization
- Full Serde field renaming and alias support
- Chunked parsing for large files
- Advanced serde features: `skip`, `default`, `Option<T>`
- Full PLY specification compliance

## Implementation Details

**Advanced Serde Support**: Comprehensive support for serde's feature set:
- `#[serde(rename = "...")]` - field renaming for different PLY naming conventions
- `#[serde(alias = "...")]` - multiple aliases per field (handles various PLY dialects)
- `#[serde(deserialize_with = "...")]` - custom field conversion (u8→f32, scaling, etc.)
- `#[serde(default)]` - default values for missing PLY properties
- `#[serde(skip)]` - computed fields not present in PLY data
- `#[serde(transparent)]` - zero-cost wrapper types (VertexId(u32), Temperature(f32))
- `Option<T>` fields - graceful handling of optional PLY properties
- Field order independence - PLY property order doesn't need to match struct
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

**Custom Type Conversion**:
- `#[serde(deserialize_with)]` for field-level type conversion
- PLY property type drives input parsing, custom function handles conversion
- Moderate performance impact (~30% slower for complex conversions, still >600 MiB/s)
- Perfect for normalizing colors (u8 → f32), scaling coordinates, etc.

**Advanced Feature Performance**:
- Basic parsing: ~1000 MiB/s (no conversions)
- With conversions: ~700 MiB/s (custom type transforms)
- Skip/default fields: no performance impact
- Option<T> fields: minimal overhead
- Transparent wrappers: zero-cost abstractions

**Chunked Architecture**:
- `PlyFile` manages complete chunked loading lifecycle
- Automatic leftover data management between chunks
- Progressive element access - parse as data becomes available
- Binary/ASCII boundary handling for incomplete elements
- Current element tracking - no need to specify element names

**PLY Specification Compliance**:
- **Complete scalar type support**: All PLY data types (char, uchar, short, ushort, int, uint, float, double)
- **Alternative type names**: Supports both traditional (int) and modern (int32) type names
- **List properties**: Variable-length lists with any count type (uchar, ushort, uint) and element type
- **Multi-element files**: Vertices, faces, edges, materials, custom user-defined elements
- **Complex structures**: Multiple lists per element, mixed scalar and list properties
- **Binary formats**: Little-endian and big-endian with proper byte ordering
- **Comments and metadata**: Full support for comment and obj_info header lines
- **Large datasets**: Efficient handling of files with thousands of elements and large lists

**Benchmarks**: Run `cargo bench --features benchmarks` to measure performance on your system.

Coding style follows grounded engineering principles (see .rules file):
- Concise code over verbose comments
- Type-level solutions over runtime checks
- Measure performance, don't assume
- Integration tests with real PLY files
