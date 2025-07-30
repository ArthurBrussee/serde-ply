//! Fast PLY parsing benchmarks
//!
//! Focused on real-world performance with minimal execution time.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use serde::Deserialize;
use std::io::Cursor;

#[derive(Deserialize)]
struct RealisticVertex {
    x: f32,
    y: f32,
    z: f32,
    nx: f32,
    ny: f32,
    nz: f32,
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Deserialize)]
struct SimpleVertex {
    x: f32,
    y: f32,
    z: f32,
}

fn generate_binary_ply_data(vertex_count: usize) -> Vec<u8> {
    let header = format!(
        r#"ply
format binary_little_endian 1.0
element vertex {}
property float x
property float y
property float z
property float nx
property float ny
property float nz
property uchar red
property uchar green
property uchar blue
end_header
"#,
        vertex_count
    );

    let mut binary_data = header.into_bytes();

    for i in 0..vertex_count {
        let base = i as f32 * 0.01;
        // Position + normal (6 floats) + color (3 u8s)
        binary_data.extend_from_slice(&base.to_le_bytes());
        binary_data.extend_from_slice(&(base + 1.0).to_le_bytes());
        binary_data.extend_from_slice(&(base + 2.0).to_le_bytes());
        binary_data.extend_from_slice(&(0.0f32).to_le_bytes());
        binary_data.extend_from_slice(&(0.0f32).to_le_bytes());
        binary_data.extend_from_slice(&(1.0f32).to_le_bytes());
        binary_data.push((i % 256) as u8);
        binary_data.push(((i * 2) % 256) as u8);
        binary_data.push(((i * 3) % 256) as u8);
    }

    binary_data
}

fn generate_ascii_ply_data(vertex_count: usize) -> String {
    let mut ply_data = format!(
        r#"ply
format ascii 1.0
element vertex {}
property float x
property float y
property float z
property float nx
property float ny
property float nz
property uchar red
property uchar green
property uchar blue
end_header
"#,
        vertex_count
    );

    for i in 0..vertex_count {
        let base = i as f32 * 0.01;
        ply_data.push_str(&format!(
            "{} {} {} 0.0 0.0 1.0 {} {} {}\n",
            base,
            base + 1.0,
            base + 2.0,
            i % 256,
            (i * 2) % 256,
            (i * 3) % 256
        ));
    }

    ply_data
}

fn generate_simple_binary_ply(vertex_count: usize) -> Vec<u8> {
    let header = format!(
        r#"ply
format binary_little_endian 1.0
element vertex {}
property float x
property float y
property float z
end_header
"#,
        vertex_count
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

fn bench_format_comparison(c: &mut Criterion) {
    let vertex_count = 1000; // Small for fast benchmarks
    let binary_data = generate_binary_ply_data(vertex_count);
    let ascii_data = generate_ascii_ply_data(vertex_count);
    let simple_binary = generate_simple_binary_ply(vertex_count);

    let mut group = c.benchmark_group("format_1k");
    group.throughput(Throughput::Bytes(binary_data.len() as u64));

    group.bench_function("simple_binary", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&simple_binary));
            let mut reader = std::io::BufReader::new(cursor);
            let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();
            let vertices: Vec<SimpleVertex> =
                serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
            black_box(vertices);
        })
    });

    group.bench_function("realistic_binary", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&binary_data));
            let mut reader = std::io::BufReader::new(cursor);
            let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();
            let vertices: Vec<RealisticVertex> =
                serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
            black_box(vertices);
        })
    });

    group.throughput(Throughput::Bytes(ascii_data.len() as u64));
    group.bench_function("realistic_ascii", |b| {
        b.iter(|| {
            let cursor = std::io::Cursor::new(black_box(&ascii_data));
            let mut reader = std::io::BufReader::new(cursor);
            let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();
            let vertices: Vec<RealisticVertex> =
                serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
            black_box(vertices);
        })
    });

    group.finish();
}

fn bench_scaling(c: &mut Criterion) {
    let sizes = vec![500, 1000, 2000]; // Much smaller for speed

    let mut group = c.benchmark_group("scaling");

    for size in sizes {
        let binary_data = generate_binary_ply_data(size);
        group.throughput(Throughput::Bytes(binary_data.len() as u64));

        group.bench_function(&format!("{}vertices", size), |b| {
            b.iter(|| {
                let cursor = Cursor::new(black_box(&binary_data));
                let mut reader = std::io::BufReader::new(cursor);
                let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();
                let vertices: Vec<RealisticVertex> =
                    serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
                black_box(vertices);
            })
        });
    }

    group.finish();
}

fn bench_chunked_parsing(c: &mut Criterion) {
    let vertex_count = 1000;
    let binary_data = generate_simple_binary_ply(vertex_count);

    let mut group = c.benchmark_group("chunked_vs_sequential");
    group.throughput(Throughput::Bytes(binary_data.len() as u64));

    // Sequential parsing baseline
    group.bench_function("sequential_binary", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&binary_data));
            let mut reader = std::io::BufReader::new(cursor);
            let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();
            let vertices: Vec<SimpleVertex> =
                serde_ply::parse_elements(&mut reader, &header, "vertex").unwrap();
            black_box(vertices);
        })
    });

    // Chunked parsing with different chunk sizes
    for chunk_size in [1024, 8192, 65536] {
        group.bench_function(&format!("chunked_{}bytes", chunk_size), |b| {
            b.iter(|| {
                let cursor = Cursor::new(black_box(&binary_data));
                let mut reader = std::io::BufReader::new(cursor);
                let header = serde_ply::PlyHeader::parse(&mut reader).unwrap();

                // Create element parser from header
                let mut header_parser = serde_ply::chunked_header_parser();

                // Simulate header parsing (we already have the header)
                let header_data = b"ply\nformat binary_little_endian 1.0\nelement vertex 1000\nproperty float x\nproperty float y\nproperty float z\nend_header\n";
                header_parser.parse_from_bytes(header_data).unwrap();
                let mut parser = header_parser.element_parser::<SimpleVertex>("vertex").unwrap();

                // Read remaining data after header
                let mut remaining_data = Vec::new();
                std::io::Read::read_to_end(&mut reader, &mut remaining_data).unwrap();

                let mut all_vertices = Vec::new();
                let mut pos = 0;

                while pos < remaining_data.len() {
                    let end = std::cmp::min(pos + chunk_size, remaining_data.len());
                    let chunk = &remaining_data[pos..end];
                    pos = end;

                    if let Some(vertices) = parser.parse_from_bytes(chunk).unwrap() {
                        all_vertices.extend(vertices);
                    }

                    if parser.is_complete() {
                        break;
                    }
                }
                black_box(all_vertices);
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_format_comparison,
    bench_scaling,
    bench_chunked_parsing
);
criterion_main!(benches);
