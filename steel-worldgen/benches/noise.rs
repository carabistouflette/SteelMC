#![feature(portable_simd)]
#![expect(missing_docs, reason = "benchmarks")]

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::simd::f64x4;
use steel_utils::random::RandomSource;
use steel_utils::random::legacy_random::LegacyRandom;
use steel_utils::random::xoroshiro::Xoroshiro;
use steel_worldgen::noise::{BlendedNoise, ImprovedNoise, NormalNoise};

fn bench_improved_noise(c: &mut Criterion) {
    let mut rng = Xoroshiro::from_seed(12345);
    let improved = ImprovedNoise::new(&mut rng);

    let mut group = c.benchmark_group("improved_noise");

    group.bench_function("scalar_3d", |b| {
        let (x, y, z) = (12.34, 56.78, 90.12);
        b.iter(|| black_box(improved.noise(black_box(x), black_box(y), black_box(z))));
    });

    group.bench_function("scalar_xz_2d", |b| {
        let (x, z) = (12.34, 90.12);
        b.iter(|| black_box(improved.noise_xz(black_box(x), black_box(z))));
    });

    let xs = f64x4::from_array([12.34, 13.34, 14.34, 15.34]);
    let ys = f64x4::from_array([56.78, 57.78, 58.78, 59.78]);
    let zs = f64x4::from_array([90.12, 91.12, 92.12, 93.12]);

    group.throughput(Throughput::Elements(4));
    group.bench_function("simd_4x_f64", |b| {
        b.iter(|| {
            black_box(improved.noise_simd::<f64, 4>(black_box(xs), black_box(ys), black_box(zs)))
        });
    });

    group.finish();
}

fn bench_normal_noise(c: &mut Criterion) {
    let mut rng = RandomSource::Xoroshiro(Xoroshiro::from_seed(54321));
    let amplitudes = [1.0, 1.0, 0.5, 0.25];
    let normal = NormalNoise::create_from_random(&mut rng, -3, &amplitudes);

    let mut group = c.benchmark_group("normal_noise");

    group.bench_function("scalar_3d", |b| {
        let (x, y, z) = (42.5, 64.0, 128.5);
        b.iter(|| black_box(normal.get_value(black_box(x), black_box(y), black_box(z))));
    });

    group.bench_function("scalar_xz_2d", |b| {
        let (x, z) = (42.5, 128.5);
        b.iter(|| black_box(normal.get_value_xz(black_box(x), black_box(z))));
    });

    let xs = f64x4::from_array([42.5, 43.5, 44.5, 45.5]);
    let ys = f64x4::from_array([64.0, 64.0, 64.0, 64.0]);
    let zs = f64x4::from_array([128.5, 129.5, 130.5, 131.5]);

    group.throughput(Throughput::Elements(4));
    group.bench_function("simd_4x_f64", |b| {
        b.iter(|| {
            black_box(normal.get_value_simd::<f64, 4>(black_box(xs), black_box(ys), black_box(zs)))
        });
    });

    group.bench_function("simd_y_4x_fixed_xz", |b| {
        let (x, z) = (42.5, 128.5);
        b.iter(|| black_box(normal.get_value_y_4x(black_box(x), black_box(ys), black_box(z))));
    });

    group.finish();
}

fn bench_blended_noise(c: &mut Criterion) {
    let mut rng = RandomSource::Legacy(LegacyRandom::from_seed(99999));
    let blended = BlendedNoise::new(&mut rng, 0.25, 0.375, 80.0, 60.0, 8.0);

    let mut group = c.benchmark_group("blended_noise");

    group.bench_function("scalar_3d", |b| {
        let (x, y, z) = (100.0, 70.0, 200.0);
        b.iter(|| black_box(blended.compute(black_box(x), black_box(y), black_box(z))));
    });

    let (x, z) = (100.0, 200.0);
    let ys = [70.0, 74.0, 78.0, 82.0];

    group.throughput(Throughput::Elements(4));
    group.bench_function("simd_4x_f64_column", |b| {
        b.iter(|| black_box(blended.compute_simd::<4>(black_box(x), black_box(ys), black_box(z))));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_improved_noise,
    bench_normal_noise,
    bench_blended_noise
);
criterion_main!(benches);
