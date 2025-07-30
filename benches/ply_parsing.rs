//! Benchmarks for PLY parsing performance
//!
//! These benchmarks measure the performance of the optimized PLY deserializer
//! with realistic vertex data, comparing ASCII vs binary formats.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
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
comment Benchmark realistic vertex data
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

    // Generate vertex data: 6 floats + 3 bytes = 27 bytes per vertex
    for i in 0..vertex_count {
        let base = i as f32 * 0.01;

        // Position (3 floats)
        binary_data.extend_from_slice(&base.to_le_bytes()); // x
        binary_data.extend_from_slice(&(base + 1.0).to_le_bytes()); // y
        binary_data.extend_from_slice(&(base + 2.0).to_le_bytes()); // z

        // Normal (3 floats)
        binary_data.extend_from_slice(&(0.0f32).to_le_bytes()); // nx
        binary_data.extend_from_slice(&(0.0f32).to_le_bytes()); // ny
        binary_data.extend_from_slice(&(1.0f32).to_le_bytes()); // nz

        // Color (3 u8s)
        binary_data.push((i % 256) as u8); // red
        binary_data.push(((i * 2) % 256) as u8); // green
        binary_data.push(((i * 3) % 256) as u8); // blue
    }

    binary_data
}

fn generate_ascii_ply_data(vertex_count: usize) -> String {
    let mut ply_data = format!(
        r#"ply
format ascii 1.0
comment Benchmark realistic vertex data
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

fn bench_binary_vs_ascii_formats(c: &mut Criterion) {
    let vertex_count = 5000; // Smaller for faster benchmarks
    let binary_data = generate_binary_ply_data(vertex_count);
    let ascii_data = generate_ascii_ply_data(vertex_count);

    let mut group = c.benchmark_group("format_comparison_5k");

    // Also test different vertex complexities
    let simple_binary = generate_simple_binary_ply(vertex_count);

    group.bench_function("simple_binary", |b| {
        b.iter(|| {
            let vertices: Vec<SimpleVertex> =
                serde_ply::from_reader(Cursor::new(black_box(&simple_binary)), "vertex").unwrap();
            black_box(vertices);
        })
    });

    group.bench_function("realistic_binary", |b| {
        b.iter(|| {
            let vertices: Vec<RealisticVertex> =
                serde_ply::from_reader(Cursor::new(black_box(&binary_data)), "vertex").unwrap();
            black_box(vertices);
        })
    });

    group.bench_function("realistic_ascii", |b| {
        b.iter(|| {
            let vertices: Vec<RealisticVertex> =
                serde_ply::from_str(black_box(&ascii_data), "vertex").unwrap();
            black_box(vertices);
        })
    });

    group.finish();
}

fn bench_scaling_performance(c: &mut Criterion) {
    let sizes = vec![1000, 5000, 25000]; // Smaller max size for faster benchmarks

    let mut group = c.benchmark_group("scaling");

    for size in sizes {
        let binary_data = generate_binary_ply_data(size);

        group.bench_function(&format!("{}k_vertices", size / 1000), |b| {
            b.iter(|| {
                let vertices: Vec<RealisticVertex> =
                    serde_ply::from_reader(Cursor::new(black_box(&binary_data)), "vertex").unwrap();
                black_box(vertices);
            })
        });
    }

    group.finish();
}

fn bench_throughput_and_memory(c: &mut Criterion) {
    // Combined throughput and memory efficiency test
    let vertex_count = 20000; // Moderate size for good statistics
    let binary_data = generate_binary_ply_data(vertex_count);
    let ascii_data = generate_ascii_ply_data(vertex_count);

    let mut group = c.benchmark_group("throughput_20k");

    // Throughput tests with byte measurements
    group.throughput(criterion::Throughput::Bytes(binary_data.len() as u64));
    group.bench_function("binary_throughput", |b| {
        b.iter(|| {
            let vertices: Vec<RealisticVertex> =
                serde_ply::from_reader(Cursor::new(black_box(&binary_data)), "vertex").unwrap();
            black_box(vertices);
        })
    });

    group.throughput(criterion::Throughput::Bytes(ascii_data.len() as u64));
    group.bench_function("ascii_throughput", |b| {
        b.iter(|| {
            let vertices: Vec<RealisticVertex> =
                serde_ply::from_str(black_box(&ascii_data), "vertex").unwrap();
            black_box(vertices);
        })
    });

    group.finish();
}

fn bench_header_parsing(c: &mut Criterion) {
    let binary_data = generate_binary_ply_data(5000); // Smaller dataset

    c.bench_function("header_parse", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&binary_data));
            let (_header, _bytes) = serde_ply::PlyHeader::parse(cursor).unwrap();
            black_box(_header);
        })
    });
}

criterion_group!(
    benches,
    bench_binary_vs_ascii_formats,
    bench_scaling_performance,
    bench_throughput_and_memory,
    bench_header_parsing
);

criterion_main!(benches);
