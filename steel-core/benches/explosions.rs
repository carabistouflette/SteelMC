#![expect(missing_docs, reason = "benchmarks")]
use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use glam::DVec3;
use steel_core::world::explosion_benchmark_support::{
    populate_pigs, run_e2e_tnt_chain_detonation, run_entity_aabb_query, run_mass_detonation,
    run_single_explosion, setup_benchmark_world, spawn_stationary_tnt, tick_stationary_tnt,
};

fn bench_e2e_1000_tnt_chain(c: &mut Criterion) {
    let world = setup_benchmark_world("e2e_tnt_chain_bench");
    let mut group = c.benchmark_group("e2e_gameplay/1000_tnt_chain_detonation");
    group.throughput(Throughput::Elements(1000));
    group.sample_size(10);

    group.bench_function("1000_tnt_chain", |b| {
        b.iter(|| {
            let affected = run_e2e_tnt_chain_detonation(&world, 1000);
            black_box(affected)
        });
    });
    group.finish();
}

fn bench_stationary_tnt_tick(c: &mut Criterion) {
    const TNT_COUNT: usize = 2_000;
    const TICKS_PER_ITERATION: usize = 20;

    let world = setup_benchmark_world("stationary_tnt_tick_bench");
    let entities = spawn_stationary_tnt(&world, TNT_COUNT);
    let mut group = c.benchmark_group("entity/stationary_tnt_tick");
    group.throughput(Throughput::Elements(
        (TNT_COUNT * TICKS_PER_ITERATION) as u64,
    ));

    group.bench_function("2000_tnt_x20_ticks", |b| {
        b.iter(|| {
            let ticks = tick_stationary_tnt(&entities, TICKS_PER_ITERATION);
            black_box(ticks)
        });
    });
    group.finish();
}

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

fn bench_entity_aabb_query(c: &mut Criterion) {
    const PIG_COUNT: usize = 200;

    let world = setup_benchmark_world("entity_query_bench");
    let spawned = populate_pigs(&world, PIG_COUNT);
    assert_eq!(spawned, PIG_COUNT);

    let mut group = c.benchmark_group("entity_query");
    group.throughput(Throughput::Elements(1));

    group.bench_function("empty_volume_100_matches_none", |b| {
        b.iter(|| {
            // High above the floor: exercises the full query path with zero matches.
            black_box(run_entity_aabb_query(
                &world,
                [-12.0, 80.0, -12.0, 12.0, 96.0, 12.0],
                true,
            ))
        });
    });

    group.bench_function("populated_volume_200", |b| {
        b.iter(|| {
            // Covers the entire spawn strip so every pig intersects the box.
            black_box(run_entity_aabb_query(
                &world,
                [-18.0, 60.0, -18.0, 18.0, 70.0, 18.0],
                true,
            ))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_e2e_1000_tnt_chain,
    bench_stationary_tnt_tick,
    bench_entity_aabb_query,
    bench_single_explosion_radius_4,
    bench_mass_detonation_100_tnt,
);
criterion_main!(benches);
