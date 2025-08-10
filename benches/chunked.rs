use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use serde::Deserialize;

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

    let vertex_count = 5000;
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

criterion_group!(benches, benchmark_chunked_parsing);
criterion_main!(benches);
