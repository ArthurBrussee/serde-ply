//! Core performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use serde::Deserialize;
use std::io::Cursor;

#[derive(Deserialize)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Deserialize)]
struct VertexWithColor {
    x: f32,
    y: f32,
    z: f32,
    red: u8,
    green: u8,
    blue: u8,
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
    let vertex_count = 500;
    let binary_data = generate_binary_ply(vertex_count);
    group.throughput(Throughput::Bytes(binary_data.len() as u64));

    // Parse header once outside the benchmark
    let mut header_cursor = Cursor::new(&binary_data);
    let header = serde_ply::PlyHeader::parse(&mut header_cursor).unwrap();
    let header_size = header_cursor.position() as usize;

    group.bench_function("simple_binary", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(black_box(&binary_data[header_size..]));
            let _vertices: Vec<Vertex> =
                serde_ply::PlyFile::parse_elements(&mut cursor, &header, "vertex").unwrap();
        });
    });

    // Realistic binary parsing (6 fields per vertex)
    let realistic_data = generate_realistic_ply(vertex_count);
    group.throughput(Throughput::Bytes(realistic_data.len() as u64));

    group.bench_function("realistic_binary", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(black_box(&realistic_data));
            let header = serde_ply::PlyHeader::parse(&mut cursor).unwrap();
            let _vertices: Vec<VertexWithColor> =
                serde_ply::PlyFile::parse_elements(&mut cursor, &header, "vertex").unwrap();
        });
    });

    // ASCII parsing comparison
    let ascii_data = generate_ascii_ply(vertex_count);
    group.throughput(Throughput::Bytes(ascii_data.len() as u64));

    group.bench_function("ascii", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(black_box(ascii_data.as_bytes()));
            let header = serde_ply::PlyHeader::parse(&mut cursor).unwrap();
            let _vertices: Vec<Vertex> =
                serde_ply::PlyFile::parse_elements(&mut cursor, &header, "vertex").unwrap();
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
            let mut ply_file = serde_ply::PlyFile::new();
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
            let mut ply_file = serde_ply::PlyFile::new();

            // Feed all data at once using the buffer_mut API
            ply_file
                .buffer_mut()
                .extend_from_slice(black_box(&binary_data));

            // Parse all vertices in one call
            let _vertices = ply_file.next_chunk::<Vertex>().unwrap().unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_parsing, benchmark_chunked_parsing);
criterion_main!(benches);
