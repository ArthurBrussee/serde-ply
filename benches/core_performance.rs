use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use serde::Deserialize;
use serde_ply::PlyFileDeserializer;
use std::io::{BufReader, Cursor};

#[derive(Deserialize)]
#[allow(unused)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Deserialize)]
#[allow(unused)]
struct VertexWithColor {
    x: f32,
    y: f32,
    z: f32,
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Deserialize)]
#[allow(unused)]
struct Face {
    vertex_indices: Vec<u32>,
}

#[derive(Deserialize)]
#[allow(unused)]
struct MultiElementPly {
    vertex: Vec<Vertex>,
    face: Vec<Face>,
}

fn generate_ascii_ply(vertex_count: usize) -> String {
    let mut ply_data = format!(
        r#"ply
format ascii 1.0
element vertex {vertex_count}
property float x
property float y
property float z
end_header
"#
    );

    for i in 0..vertex_count {
        let base = i as f32 * 0.01;
        ply_data.push_str(&format!("{} {} {}\n", base, base + 1.0, base + 2.0));
    }
    ply_data
}

fn generate_binary_ply(vertex_count: usize) -> Vec<u8> {
    let header = format!(
        r#"ply
format binary_little_endian 1.0
element vertex {vertex_count}
property float x
property float y
property float z
end_header
"#
    );

    let mut binary_data = header.into_bytes();
    for i in 0..vertex_count {
        let base = i as f32 * 0.01;
        binary_data.extend_from_slice(&base.to_le_bytes());
        binary_data.extend_from_slice(&(base + 1.0).to_le_bytes());
        binary_data.extend_from_slice(&(base + 2.0).to_le_bytes());
    }
    binary_data
}

fn generate_multi_element_ply(vertex_count: usize, face_count: usize) -> Vec<u8> {
    let header = format!(
        r#"ply
format binary_little_endian 1.0
element vertex {vertex_count}
property float x
property float y
property float z
element face {face_count}
property list uchar uint vertex_indices
end_header
"#
    );

    let mut binary_data = header.into_bytes();

    // Add vertex data
    for i in 0..vertex_count {
        let base = i as f32 * 0.01;
        binary_data.extend_from_slice(&base.to_le_bytes());
        binary_data.extend_from_slice(&(base + 1.0).to_le_bytes());
        binary_data.extend_from_slice(&(base + 2.0).to_le_bytes());
    }

    // Add face data
    for i in 0..face_count {
        let indices_count = 3u8; // Triangle faces
        binary_data.push(indices_count);
        let v0 = (i * 3) as u32;
        let v1 = (i * 3 + 1) as u32;
        let v2 = (i * 3 + 2) as u32;
        binary_data.extend_from_slice(&v0.to_le_bytes());
        binary_data.extend_from_slice(&v1.to_le_bytes());
        binary_data.extend_from_slice(&v2.to_le_bytes());
    }

    binary_data
}

fn generate_realistic_ply(vertex_count: usize) -> Vec<u8> {
    let header = format!(
        r#"ply
format binary_little_endian 1.0
element vertex {vertex_count}
property float x
property float y
property float z
property uchar red
property uchar green
property uchar blue
end_header
"#
    );

    let mut binary_data = header.into_bytes();
    for i in 0..vertex_count {
        let base = i as f32 * 0.01;
        binary_data.extend_from_slice(&base.to_le_bytes());
        binary_data.extend_from_slice(&(base + 1.0).to_le_bytes());
        binary_data.extend_from_slice(&(base + 2.0).to_le_bytes());
        binary_data.push((i % 256) as u8);
        binary_data.push(((i * 2) % 256) as u8);
        binary_data.push(((i * 3) % 256) as u8);
    }
    binary_data
}

fn benchmark_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    // Simple binary parsing (3 floats per vertex)
    let vertex_count = 5000;
    let binary_data = generate_binary_ply(vertex_count);
    group.throughput(Throughput::Bytes(binary_data.len() as u64));

    group.bench_function("simple_binary", |b| {
        b.iter(|| {
            // Parse header once outside the benchmark
            let header_cursor = Cursor::new(&binary_data);
            let mut file = PlyFileDeserializer::from_reader(BufReader::new(header_cursor)).unwrap();
            let _vertices: Vec<Vertex> = file.next_element().unwrap();
        });
    });

    // Realistic binary parsing (6 fields per vertex)
    let realistic_data = generate_realistic_ply(vertex_count);
    group.throughput(Throughput::Bytes(realistic_data.len() as u64));

    group.bench_function("realistic_binary", |b| {
        b.iter(|| {
            // Parse header once outside the benchmark
            let cursor = Cursor::new(&realistic_data);
            let mut file = PlyFileDeserializer::from_reader(BufReader::new(cursor)).unwrap();
            let _vertices: Vec<VertexWithColor> = file.next_element().unwrap();
        });
    });

    // ASCII parsing comparison
    let ascii_data = generate_ascii_ply(vertex_count);
    group.throughput(Throughput::Bytes(ascii_data.len() as u64));

    group.bench_function("ascii", |b| {
        b.iter(|| {
            // Parse header once outside the benchmark
            let cursor = Cursor::new(ascii_data.as_bytes());
            let mut file = PlyFileDeserializer::from_reader(BufReader::new(cursor)).unwrap();
            let _vertices: Vec<Vertex> = file.next_element().unwrap();
        });
    });

    // Multi-element parsing
    let multi_element_data = generate_multi_element_ply(vertex_count, vertex_count / 3);
    group.throughput(Throughput::Bytes(multi_element_data.len() as u64));

    group.bench_function("multi_element", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&multi_element_data));
            let reader = std::io::BufReader::new(cursor);
            let ply: MultiElementPly = serde_ply::from_reader(reader).unwrap();
            black_box(ply);
        });
    });

    group.finish();
}

fn benchmark_chunked_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunked");

    let vertex_count = 500;
    let binary_data = generate_binary_ply(vertex_count);
    group.throughput(Throughput::Bytes(binary_data.len() as u64));

    group.bench_function("chunked_4k", |b| {
        b.iter(|| {
            let mut ply_file = serde_ply::ChunkPlyFile::new();
            let chunk_size = 4096;

            // Feed data in chunks using the buffer_mut API
            for chunk in binary_data.chunks(chunk_size) {
                ply_file.buffer_mut().extend_from_slice(black_box(chunk));

                // Parse all vertices
                let _vertices = ply_file.next_chunk::<Vertex>().unwrap();
            }
        });
    });

    group.bench_function("chunked_all_at_once", |b| {
        b.iter(|| {
            let mut ply_file = serde_ply::ChunkPlyFile::new();

            // Feed all data at once using the buffer_mut API
            ply_file
                .buffer_mut()
                .extend_from_slice(black_box(&binary_data));

            // Parse all vertices in one call
            let _vertices = ply_file.next_chunk::<Vertex>().unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_parsing, benchmark_chunked_parsing);
criterion_main!(benches);
