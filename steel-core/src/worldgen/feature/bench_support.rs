//! Benchmark-only wrappers around the sculk patch read and placement paths.
//!
//! These let an external criterion bench exercise the exact source shapes the
//! sculk patch feature uses, identically on every branch under comparison,
//! without touching gameplay code.

use std::sync::Arc;

use super::prelude::*;
use super::runner::FeatureDecorationRunner;
use crate::chunk::chunk_generation_task::StaticCache2D;
use crate::chunk::chunk_holder::ChunkHolder;
use crate::chunk::chunk_pyramid::ChunkStep;
use crate::worldgen::generator::ChunkGenerator as _;
use crate::worldgen::generator::context::WorldGenContext;
use crate::worldgen::region::WorldGenRegion;
use steel_utils::ChunkPos;

/// Reads `pos` and `pos.below()` the way the sculk patch spread predicate does
/// on this branch: one batched region read acquiring the section lock once.
#[must_use]
pub fn read_pair(region: &WorldGenRegion<'_>, pos: BlockPos) -> [BlockStateId; 2] {
    let below = pos.below();
    region.block_states_for([pos, below])
}

/// Places vanilla sculk patches at each origin and returns how many placements
/// succeeded. Each origin is placed with a fresh `WorldgenRandom` seeded from
/// `seed`, so the placement sequence is deterministic and comparable across
/// branches. Config values are the vanilla `minecraft:sculk_patch` settings.
pub fn place_sculk_patch_stress(
    context: &WorldGenContext,
    step: &ChunkStep,
    cache: &StaticCache2D<Arc<ChunkHolder>>,
    center: ChunkPos,
    origins: &[BlockPos],
    seed: u64,
) -> u64 {
    let config = SculkPatchConfiguration {
        charge_count: 10,
        amount_per_charge: 32,
        spread_attempts: 64,
        growth_rounds: 0,
        spread_rounds: 1,
        extra_rare_growths: IntProvider::Uniform {
            min_inclusive: 1,
            max_inclusive: 3,
        },
        catalyst_chance: 0.5,
    };

    let world_seed = context.world().seed();
    let region_random = context
        .generator
        .create_worldgen_region_random(world_seed, center);
    let mut region = WorldGenRegion::new(context, step, cache, center, region_random);

    let mut placed = 0_u64;
    for &origin in origins {
        let mut random = WorldgenRandom::from_seed(seed);
        if FeatureDecorationRunner::place_sculk_patch_feature(
            &mut region,
            &REGISTRY,
            &mut random,
            &config,
            origin,
        ) {
            placed += 1;
        }
    }
    placed
}

/// Scans fixed columns top-down for the first air-above-solid position — the
/// cave-floor shape a vanilla sculk patch spreads from. Column `y` values are
/// ignored; `y_top`/`y_bottom` bound the scan. Columns that find nothing fall
/// back to their given position, which the feature rejects during placement.
/// Deterministic for identical chunk content, so comparable across branches.
#[must_use]
pub fn sculk_bench_origins(
    region: &WorldGenRegion<'_>,
    columns: &[BlockPos],
    y_top: i32,
    y_bottom: i32,
) -> Vec<BlockPos> {
    columns
        .iter()
        .map(|column| {
            let mut y = y_top;
            while y >= y_bottom {
                let pos = BlockPos::new(column.x(), y, column.z());
                if region.block_state(pos).is_air() && !region.block_state(pos.below()).is_air() {
                    return pos;
                }
                y -= 1;
            }
            *column
        })
        .collect()
}
