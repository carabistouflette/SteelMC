#![expect(missing_docs, reason = "benchmarks")]
use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use glam::DVec3;
use steel_core::world::explosion_benchmark_support::{
    run_mass_detonation, run_single_explosion, setup_benchmark_world,
};

fn bench_single_explosion_radius_4(c: &mut Criterion) {
    let world = setup_benchmark_world("single_explosion_bench");
    let mut group = c.benchmark_group("explosion/single_radius_4");
    group.throughput(Throughput::Elements(1));

    group.bench_function("explode", |b| {
        b.iter(|| {
            let center = DVec3::new(0.5, 64.5, 0.5);
            let affected = run_single_explosion(&world, center);
            black_box(affected)
        });
    });
    group.finish();
}

fn bench_mass_detonation_100_tnt(c: &mut Criterion) {
    let world = setup_benchmark_world("mass_detonation_bench");
    let mut group = c.benchmark_group("explosion/mass_detonation_100");
    group.throughput(Throughput::Elements(100));

    group.bench_function("100_explosions", |b| {
        b.iter(|| {
            let affected = run_mass_detonation(&world, 100);
            black_box(affected)
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_single_explosion_radius_4,
    bench_mass_detonation_100_tnt,
);
criterion_main!(benches);
