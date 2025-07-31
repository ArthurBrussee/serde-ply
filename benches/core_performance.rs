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
    let vertex_count = 1000;
    let binary_data = generate_binary_ply(vertex_count);
    group.throughput(Throughput::Bytes(binary_data.len() as u64));

    group.bench_function("simple_binary", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&binary_data));
            let mut reader = std::io::BufReader::new(cursor);
            let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();
            let _vertices: Vec<Vertex> =
                serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
        });
    });

    // Realistic binary parsing (6 fields per vertex)
    let realistic_data = generate_realistic_ply(vertex_count);
    group.throughput(Throughput::Bytes(realistic_data.len() as u64));

    group.bench_function("realistic_binary", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&realistic_data));
            let mut reader = std::io::BufReader::new(cursor);
            let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();
            let _vertices: Vec<VertexWithColor> =
                serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
        });
    });

    // ASCII parsing comparison
    let ascii_data = generate_ascii_ply(vertex_count);
    group.throughput(Throughput::Bytes(ascii_data.len() as u64));

    group.bench_function("ascii", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(ascii_data.as_bytes()));
            let mut reader = std::io::BufReader::new(cursor);
            let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();
            let _vertices: Vec<Vertex> =
                serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
        });
    });

    group.finish();
}

fn benchmark_chunked_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunked");

    let vertex_count = 1000;
    let binary_data = generate_binary_ply(vertex_count);
    group.throughput(Throughput::Bytes(binary_data.len() as u64));

    group.bench_function("chunked_4k", |b| {
        b.iter(|| {
            let mut header_parser = serde_ply::chunked_header_parser();
            let mut pos = 0;
            let chunk_size = 4096;

            // Parse header
            while pos < binary_data.len() {
                let end = (pos + chunk_size).min(binary_data.len());
                let chunk = &binary_data[pos..end];
                if header_parser.parse_from_bytes(chunk).unwrap().is_some() {
                    break;
                }
                pos = end;
            }

            let mut file_parser = header_parser.into_file_parser().unwrap();

            // Parse elements
            while pos < binary_data.len() {
                let end = (pos + chunk_size).min(binary_data.len());
                let chunk = &binary_data[pos..end];
                file_parser.add_data(chunk);
                let _ = file_parser.parse_chunk::<Vertex>("vertex").unwrap();
                pos = end;
                if file_parser.is_element_complete("vertex") {
                    break;
                }
            }
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_parsing, benchmark_chunked_parsing);
criterion_main!(benches);
