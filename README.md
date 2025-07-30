# serde_ply

Fast PLY parser using serde for direct struct deserialization. Handles ASCII and binary formats.

## Architecture

Header-first parsing: PLY headers define variable data structure, so we parse the header first to understand what we're reading, then deserialize data accordingly.

Core types:
- `PlyHeader` - Parsed header with format info and element definitions
- `ElementDeserializer` - Direct visitor pattern deserializer
- `ElementDef` / `PropertyType` - Header structure definitions

## API

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Vertex { x: f32, y: f32, z: f32 }

// Main API
let vertices: Vec<Vertex> = serde_ply::from_reader(reader, "vertex")?;
let vertices: Vec<Vertex> = serde_ply::from_str(ply_data, "vertex")?;

// Header inspection
let (header, bytes_consumed) = PlyHeader::parse(reader)?;
```

## Implementation Notes

### Struct as Source of Truth (de.rs)
Key insight: The struct definition becomes the source of truth, not the PLY header. We validate ONCE during setup that the struct matches the PLY structure, then trust the struct completely.

**Setup phase (`ElementDeserializer::new()`):**
- Validate struct fields match PLY properties (fail fast if mismatch)
- Pre-compute field names for serde's map access
- No type information stored - we trust serde's visitor calls

**Reading phase (per field):**
- Format dispatch once per field: create `AsciiScalarDeserializer` or `BinaryScalarDeserializer<LittleEndian>` or `BinaryScalarDeserializer<BigEndian>`
- Serde calls `deserialize_f32()` → we read f32 directly (no type checking, no endianness checking)
- Serde calls `deserialize_i32()` → we read i32 directly (no type checking, no endianness checking)
- Zero runtime dispatch based on PLY scalar types
- Zero runtime dispatch based on endianness
- Zero property lookups or cloning

### Format Detection (lib.rs)
Header parsing automatically detects format from "format" line. Same API works for all formats.

Line ending handling: Searches for both `end_header\n` and `end_header\r\n` when parsing headers.

### Performance
Key optimizations achieved:
1. **Struct as source of truth** - Eliminates all runtime type checking and dispatch
2. **Format-specific types** - `AsciiScalarDeserializer`, `BinaryScalarDeserializer<LittleEndian>`, `BinaryScalarDeserializer<BigEndian>`
3. **Zero runtime checks** - No format checks, no endianness checks, no scalar type checks
4. **byteorder integration** - Direct `reader.read_f32::<E>()` calls eliminate manual buffer management
5. **Validation once, trust forever** - Validate struct compatibility during setup only
6. **Direct visitor pattern** - No intermediate HashMap allocations
7. **Serde drives type reading** - When serde wants f32, we read f32 (no PLY metadata consulted)
8. **Clean separation** - ASCII, binary LE, and binary BE are completely separate code paths

## Current Limitations

1. **Multi-element files**: Reading different element types requires separate calls. Could be improved to handle multiple element types in single pass.

2. **Element positioning**: When reading non-first element types, deserializer doesn't automatically skip to correct data position. Works for single element type files.

3. **Write support**: Serialization exists but not fully implemented for binary formats.

## Test Strategy

Integration tests with real PLY files in `example_plys/`:
- `greg_turk_example1_ok_ascii.ply` - Basic cube
- `house_2_ok_little_endian.ply` - Binary format
- `all_atomic_types_ok_ascii.ply` - All scalar types

Avoid micro unit tests. Focus on real file parsing correctness.

## Binary Format Implementation

Binary reading in `DirectScalarDeserializer::deserialize_binary()`:
- Reads exact byte counts for each type
- Uses `from_le_bytes()` / `from_be_bytes()` for endianness
- Format detection from header determines endianness handling

## Development Notes

When modifying:
- Struct compatibility validation happens once in `ElementDeserializer::new()`
- Format/endianness dispatch happens once per field in `DirectMapAccess::next_value_seed()`
- Type-specific deserializers: `AsciiScalarDeserializer`, `BinaryScalarDeserializer<LittleEndian>`, `BinaryScalarDeserializer<BigEndian>`
- `deserialize_f32()`, `deserialize_i32()` etc. use direct `byteorder` calls with zero runtime checks
- `byteorder` crate handles all byte reading - no manual buffer management needed
- Lists use format-specific deserializers: `AsciiListDeserializer`, `BinaryListDeserializer<E>`
- Error handling via thiserror for clear messages

Performance critical paths (zero dispatch):
- `AsciiScalarDeserializer::deserialize_*()` → token parsing
- `BinaryScalarDeserializer<LittleEndian>::deserialize_*()` → `reader.read_*::<LittleEndian>()`  
- `BinaryScalarDeserializer<BigEndian>::deserialize_*()` → `reader.read_*::<BigEndian>()`
- All hot paths have zero runtime branching or type checking

## Usage Patterns

Single element type (most common):
```rust
let vertices: Vec<Vertex> = serde_ply::from_reader(file, "vertex")?;
```

Multiple element types (requires separate calls):
```rust
let vertices: Vec<Vertex> = serde_ply::from_reader(&file, "vertex")?;
let faces: Vec<Face> = serde_ply::from_reader(&file, "face")?; // Issue: will re-read from start
```

Header inspection for dynamic handling:
```rust
let (header, _) = PlyHeader::parse(&file)?;
for element in &header.elements {
    println!("Element: {}, count: {}", element.name, element.count);
}
```
