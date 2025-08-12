use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use serde::Deserialize;

use std::io::{BufReader, Cursor};

#[derive(Deserialize)]
#[allow(unused)]
struct GaussianSplat {
    x: f32,
    y: f32,
    z: f32,

    scale_0: f32,
    scale_1: f32,
    scale_2: f32,

    opacity: f32,

    rot_0: f32,
    rot_1: f32,
    rot_2: f32,
    rot_3: f32,

    #[serde(default)]
    f_dc_0: f32,
    #[serde(default)]
    f_dc_1: f32,
    #[serde(default)]
    f_dc_2: f32,

    #[serde(default)]
    f_rest_0: f32,
    #[serde(default)]
    f_rest_1: f32,
    #[serde(default)]
    f_rest_2: f32,
    #[serde(default)]
    f_rest_3: f32,
    #[serde(default)]
    f_rest_4: f32,
    #[serde(default)]
    f_rest_5: f32,
    #[serde(default)]
    f_rest_6: f32,
    #[serde(default)]
    f_rest_7: f32,
    #[serde(default)]
    f_rest_8: f32,
    #[serde(default)]
    f_rest_9: f32,
    #[serde(default)]
    f_rest_10: f32,
    #[serde(default)]
    f_rest_11: f32,
    #[serde(default)]
    f_rest_12: f32,
    #[serde(default)]
    f_rest_13: f32,
    #[serde(default)]
    f_rest_14: f32,
    #[serde(default)]
    f_rest_15: f32,
    #[serde(default)]
    f_rest_16: f32,
    #[serde(default)]
    f_rest_17: f32,
    #[serde(default)]
    f_rest_18: f32,
    #[serde(default)]
    f_rest_19: f32,
    #[serde(default)]
    f_rest_20: f32,
    #[serde(default)]
    f_rest_21: f32,
    #[serde(default)]
    f_rest_22: f32,
    #[serde(default)]
    f_rest_23: f32,
    #[serde(default)]
    f_rest_24: f32,
    #[serde(default)]
    f_rest_25: f32,
    #[serde(default)]
    f_rest_26: f32,
    #[serde(default)]
    f_rest_27: f32,
    #[serde(default)]
    f_rest_28: f32,
    #[serde(default)]
    f_rest_29: f32,
    #[serde(default)]
    f_rest_30: f32,
    #[serde(default)]
    f_rest_31: f32,
    #[serde(default)]
    f_rest_32: f32,
    #[serde(default)]
    f_rest_33: f32,
    #[serde(default)]
    f_rest_34: f32,
    #[serde(default)]
    f_rest_35: f32,
    #[serde(default)]
    f_rest_36: f32,
    #[serde(default)]
    f_rest_37: f32,
    #[serde(default)]
    f_rest_38: f32,
    #[serde(default)]
    f_rest_39: f32,
    #[serde(default)]
    f_rest_40: f32,
    #[serde(default)]
    f_rest_41: f32,
    #[serde(default)]
    f_rest_42: f32,
    #[serde(default)]
    f_rest_43: f32,
    #[serde(default)]
    f_rest_44: f32,
}

#[derive(Deserialize)]
#[allow(unused)]
struct SplatPly {
    vertex: Vec<GaussianSplat>,
}

fn generate_test_data(num_splats: usize) -> Vec<u8> {
    let header = format!(
        r#"ply
format binary_little_endian 1.0
comment Exported from Brush
comment Vertical axis: y
element vertex {num_splats}
property float x
property float y
property float z
property float scale_0
property float scale_1
property float scale_2
property float opacity
property float rot_0
property float rot_1
property float rot_2
property float rot_3
property float f_dc_0
property float f_dc_1
property float f_dc_2
property float f_rest_0
property float f_rest_1
property float f_rest_2
property float f_rest_3
property float f_rest_4
property float f_rest_5
property float f_rest_6
property float f_rest_7
property float f_rest_8
property float f_rest_9
property float f_rest_10
property float f_rest_11
property float f_rest_12
property float f_rest_13
property float f_rest_14
property float f_rest_15
property float f_rest_16
property float f_rest_17
property float f_rest_18
property float f_rest_19
property float f_rest_20
property float f_rest_21
property float f_rest_22
property float f_rest_23
property float f_rest_24
property float f_rest_25
property float f_rest_26
property float f_rest_27
property float f_rest_28
property float f_rest_29
property float f_rest_30
property float f_rest_31
property float f_rest_32
property float f_rest_33
property float f_rest_34
property float f_rest_35
property float f_rest_36
property float f_rest_37
property float f_rest_38
property float f_rest_39
property float f_rest_40
property float f_rest_41
property float f_rest_42
property float f_rest_43
property float f_rest_44
end_header
"#
    );

    let mut data = header.as_bytes().to_vec();

    // Generate binary data
    for i in 0..num_splats {
        let i_f = i as f32;
        // Position
        data.extend_from_slice(&(i_f * 0.1).to_le_bytes());
        data.extend_from_slice(&((i_f * 0.13) % 10.0).to_le_bytes());
        data.extend_from_slice(&((i_f * 0.17) % 10.0).to_le_bytes());
        // Scale
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        // Opacity
        data.extend_from_slice(&(0.5f32).to_le_bytes());
        // Rotation
        data.extend_from_slice(&0.707f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.707f32.to_le_bytes());

        // SH DC & rest
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
    let num_splats = 100000;
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
