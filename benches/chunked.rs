use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use serde::de::DeserializeSeed;
use serde::Deserialize;
use serde_ply::RowVisitor;

#[derive(Deserialize)]
#[allow(unused)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
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

fn benchmark_chunked_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunked");

    let vertex_count = 50000;
    let binary_data = generate_binary_ply(vertex_count);
    group.throughput(Throughput::Bytes(binary_data.len() as u64));

    group.bench_function("chunked_chunks", |b| {
        b.iter(|| {
            let mut ply_file = serde_ply::PlyChunkedReader::new();
            let chunk_size = 8 * 1024 * 1024;
            let mut vertices: Vec<Vertex> = Vec::new();

            // Feed data in chunks using the buffer_mut API
            for chunk in binary_data.chunks(chunk_size) {
                ply_file.buffer_mut().extend_from_slice(black_box(chunk));

                RowVisitor::new(|vertex: Vertex| {
                    vertices.push(vertex);
                })
                .deserialize(&mut ply_file)
                .unwrap();
            }
        });
    });

    group.bench_function("chunked_all_at_once", |b| {
        b.iter(|| {
            let mut ply_file = serde_ply::PlyChunkedReader::new();
            let mut vertices: Vec<Vertex> = Vec::new();

            // Feed all data at once using the buffer_mut API
            ply_file
                .buffer_mut()
                .extend_from_slice(black_box(&binary_data));

            RowVisitor::new(|vertex: Vertex| {
                vertices.push(vertex);
            })
            .deserialize(&mut ply_file)
            .unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_chunked_parsing);
criterion_main!(benches);
