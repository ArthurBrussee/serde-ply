use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use serde::Deserialize;

use std::io::{BufReader, Cursor};

#[derive(Deserialize)]
#[allow(unused)]
struct GaussianSplat {
    x: f32,
    y: f32,
    z: f32,
    rot_x: f32,
    rot_y: f32,
    rot_z: f32,
    rot_w: f32,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
    #[serde(default)]
    sh_val_0: f32,
    #[serde(default)]
    sh_val_1: f32,
    #[serde(default)]
    sh_val_2: f32,
    #[serde(default)]
    sh_val_3: f32,
    #[serde(default)]
    sh_val_4: f32,
    #[serde(default)]
    sh_val_5: f32,
    #[serde(default)]
    sh_val_6: f32,
    #[serde(default)]
    sh_val_7: f32,
    #[serde(default)]
    sh_val_8: f32,
    #[serde(default)]
    sh_val_9: f32,
    #[serde(default)]
    sh_val_10: f32,
    #[serde(default)]
    sh_val_11: f32,
    #[serde(default)]
    sh_val_12: f32,
    #[serde(default)]
    sh_val_13: f32,
    #[serde(default)]
    sh_val_14: f32,
    #[serde(default)]
    sh_val_15: f32,
    #[serde(default)]
    sh_val_16: f32,
    #[serde(default)]
    sh_val_17: f32,
    #[serde(default)]
    sh_val_18: f32,
    #[serde(default)]
    sh_val_19: f32,
    #[serde(default)]
    sh_val_20: f32,
    #[serde(default)]
    sh_val_21: f32,
    #[serde(default)]
    sh_val_22: f32,
    #[serde(default)]
    sh_val_23: f32,
    #[serde(default)]
    sh_val_24: f32,
    #[serde(default)]
    sh_val_25: f32,
    #[serde(default)]
    sh_val_26: f32,
    #[serde(default)]
    sh_val_27: f32,
    #[serde(default)]
    sh_val_28: f32,
    #[serde(default)]
    sh_val_29: f32,
    #[serde(default)]
    sh_val_30: f32,
    #[serde(default)]
    sh_val_31: f32,
    #[serde(default)]
    sh_val_32: f32,
    #[serde(default)]
    sh_val_33: f32,
    #[serde(default)]
    sh_val_34: f32,
    #[serde(default)]
    sh_val_35: f32,
    #[serde(default)]
    sh_val_36: f32,
    #[serde(default)]
    sh_val_37: f32,
    #[serde(default)]
    sh_val_38: f32,
    #[serde(default)]
    sh_val_39: f32,
    #[serde(default)]
    sh_val_40: f32,
    #[serde(default)]
    sh_val_41: f32,
    #[serde(default)]
    sh_val_42: f32,
    #[serde(default)]
    sh_val_43: f32,
    #[serde(default)]
    sh_val_44: f32,
    #[serde(default)]
    sh_val_45: f32,
    #[serde(default)]
    sh_val_46: f32,
    #[serde(default)]
    sh_val_47: f32,
}

#[derive(Deserialize)]
#[allow(unused)]
struct SplatPly {
    vertex: Vec<GaussianSplat>,
}

fn generate_test_data(num_splats: usize) -> Vec<u8> {
    let mut header = String::from("ply\nformat binary_little_endian 1.0\n");
    header.push_str(&format!("element vertex {num_splats}\n"));
    header.push_str("property float x\n");
    header.push_str("property float y\n");
    header.push_str("property float z\n");
    header.push_str("property float rot_x\n");
    header.push_str("property float rot_y\n");
    header.push_str("property float rot_z\n");
    header.push_str("property float rot_w\n");
    header.push_str("property float scale_x\n");
    header.push_str("property float scale_y\n");
    header.push_str("property float scale_z\n");
    for i in 0..48 {
        header.push_str(&format!("property float sh_val_{i}\n"));
    }
    header.push_str("end_header\n");

    let mut data = header.into_bytes();

    // Generate binary data
    for i in 0..num_splats {
        let i_f = i as f32;
        // Position
        data.extend_from_slice(&(i_f * 0.1).to_le_bytes());
        data.extend_from_slice(&((i_f * 0.13) % 10.0).to_le_bytes());
        data.extend_from_slice(&((i_f * 0.17) % 10.0).to_le_bytes());
        // Rotation
        data.extend_from_slice(&0.707f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.707f32.to_le_bytes());
        // Scale
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        // SH coefficients
        for j in 0..48 {
            let val = if j == 0 {
                0.5
            } else {
                (j as f32 * 0.01 * i_f.sin()).clamp(-0.1, 0.1)
            };
            data.extend_from_slice(&val.to_le_bytes());
        }
    }
    data
}

fn bench_gaussian_splat(c: &mut Criterion) {
    let num_splats = 10000;
    let test_data = generate_test_data(num_splats);

    let mut group = c.benchmark_group("gaussian_splat");
    group.throughput(Throughput::Elements(num_splats as u64));

    group.bench_function("gaussian_splat", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&test_data));
            let reader = BufReader::new(cursor);
            let ply: SplatPly = serde_ply::from_reader(reader).unwrap();
            black_box(ply.vertex)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_gaussian_splat);
criterion_main!(benches);
