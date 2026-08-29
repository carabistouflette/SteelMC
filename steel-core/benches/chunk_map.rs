#![expect(missing_docs, reason = "benchmarks")]
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use glam::DVec3;
use std::hint::black_box;
use std::sync::{Arc, Weak};
use steel_core::bootstrap::init_globals_once;
use steel_core::chunk::chunk_ticket_manager::{ChunkTicket, ChunkTicketLevel, ChunkTicketManager};
use steel_core::chunk::paletted_container::PalettedContainer;
use steel_core::entity::{
    Entity, EntityBase, EntityOwnership, EntityVisibility, SharedEntity, WorldEntityManager,
};
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::vanilla_entities;
use steel_utils::downcast::{DowncastType, DowncastTypeKey};
use steel_utils::{BlockStateId, ChunkPos, WorldAabb};
use uuid::Uuid;

struct BenchEntity {
    base: EntityBase,
}

impl BenchEntity {
    fn shared(id: i32, position: DVec3) -> SharedEntity {
        let uuid = Uuid::from_u128(id as u128);
        Arc::new(Self {
            base: EntityBase::with_uuid(
                id,
                uuid,
                position,
                vanilla_entities::ITEM.dimensions,
                Weak::new(),
            ),
        })
    }
}

// SAFETY: BenchEntity is a private benchmark type with a unique key.
unsafe impl DowncastType for BenchEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:bench_entity");
}

impl Entity for BenchEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        &vanilla_entities::ITEM
    }
}

fn bench_ticket_propagation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ticket_propagation");

    group.bench_function("single_strong_ticket_propagation", |b| {
        b.iter(|| {
            let mut manager = ChunkTicketManager::new();
            manager.add_ticket(
                ChunkPos::new(0, 0),
                ChunkTicket::loading(ChunkTicketLevel::STRONGEST),
            );
            let changes = manager.run_all_updates();
            black_box(changes.len())
        });
    });

    group.bench_function("overlapping_players_16x16_grid", |b| {
        let player_positions: Vec<ChunkPos> = (-4..=4)
            .flat_map(|x| (-4..=4).map(move |z| ChunkPos::new(x * 2, z * 2)))
            .collect();

        b.iter(|| {
            let mut manager = ChunkTicketManager::new();
            for (i, &pos) in player_positions.iter().enumerate() {
                let level = if i % 2 == 0 {
                    ChunkTicketLevel::BLOCK_TICKING_CHUNK
                } else {
                    ChunkTicketLevel::ENTITY_TICKING_CHUNK
                };
                manager.add_ticket(pos, ChunkTicket::loading(level));
            }
            let changes = manager.run_all_updates();
            black_box(changes.len())
        });
    });

    group.bench_function("incremental_ticket_update_single_move", |b| {
        let old_pos = ChunkPos::new(0, 0);
        let new_pos = ChunkPos::new(1, 0);
        let ticket = ChunkTicket::loading(ChunkTicketLevel::STRONGEST);

        b.iter(|| {
            let mut manager = ChunkTicketManager::new();
            for x in -5..=5 {
                for z in -5..=5 {
                    manager.add_ticket(
                        ChunkPos::new(x, z),
                        ChunkTicket::loading(ChunkTicketLevel::ENTITY_TICKING_CHUNK),
                    );
                }
            }
            manager.run_all_updates();
            manager.remove_ticket(old_pos, ticket);
            manager.add_ticket(new_pos, ticket);
            let changes = manager.run_all_updates();
            black_box(changes.len())
        });
    });

    group.finish();
}

fn bench_spatial_entity_manager(c: &mut Criterion) {
    init_globals_once();
    let mut group = c.benchmark_group("spatial_entity_manager");

    let manager = Arc::new(WorldEntityManager::new());

    // Load a 10x10 chunk grid (100 chunks, 160x160 blocks)
    for x in -5..5 {
        for z in -5..5 {
            let pos = ChunkPos::new(x, z);
            manager.on_chunk_loaded(pos);
            manager.update_chunk_visibility(pos, EntityVisibility::Ticking);
        }
    }

    // Populate with 1,000 entities distributed across the area
    let entity_count = 1000;
    for i in 0..entity_count {
        let x = (f64::from(i % 160)) - 80.0;
        let y = 64.0 + f64::from((i / 160) % 10);
        let z = (f64::from((i * 7) % 160)) - 80.0;
        let pos = DVec3::new(x, y, z);
        let entity = BenchEntity::shared(i + 1, pos);
        manager
            .add_live_entity(entity, EntityOwnership::ManagerOwned)
            .expect("valid entity addition");
    }

    // Benchmark broadphase AABB queries of different sizes
    let query_sizes = [
        ("point_1x1x1", 1.0),
        ("medium_8x8x8", 8.0),
        ("chunk_16x16x16", 16.0),
        ("large_48x48x48", 48.0),
    ];

    for (name, extent) in query_sizes {
        let aabb = WorldAabb::of_size(DVec3::new(0.0, 65.0, 0.0), extent, 10.0, extent);

        group.bench_with_input(
            BenchmarkId::new("get_entities_in_aabb", name),
            &aabb,
            |b, aabb| {
                b.iter(|| {
                    let entities = manager.get_entities_in_aabb(black_box(aabb));
                    black_box(entities.len())
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("has_entity_matching", name),
            &aabb,
            |b, aabb| {
                b.iter(|| {
                    let has_match = manager.has_entity_in_aabb_matching(black_box(aabb), |_| true);
                    black_box(has_match)
                });
            },
        );
    }

    // Benchmark entity move commit across spatial cell boundary
    let move_entity_id = 42;
    let base_pos = DVec3::new(0.0, 64.0, 0.0);
    let target_pos = DVec3::new(5.5, 64.0, 5.5); // Crosses ENTITY_SPATIAL_CELL_SIZE (4.0)

    group.bench_function("commit_move_cross_cell", |b| {
        let mut toggle = false;
        b.iter(|| {
            let pos = if toggle { base_pos } else { target_pos };
            toggle = !toggle;
            let result = manager.commit_move(move_entity_id, pos);
            black_box(result)
        });
    });

    group.finish();
}

fn bench_paletted_container_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("paletted_container_write");

    let homogeneous: PalettedContainer<BlockStateId, 16> =
        PalettedContainer::Homogeneous(BlockStateId(1));

    let mut cube16 = Box::new([[[BlockStateId(0); 16]; 16]; 16]);
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                cube16[y][z][x] = BlockStateId(((x + z + y) % 16) as u16);
            }
        }
    }
    let palette16 = PalettedContainer::from_cube(cube16);

    let mut cube64 = Box::new([[[BlockStateId(0); 16]; 16]; 16]);
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                cube64[y][z][x] = BlockStateId(((x + z * 16 + y * 256) % 64) as u16);
            }
        }
    }
    let palette64 = PalettedContainer::from_cube(cube64);

    let mut cube256 = Box::new([[[BlockStateId(0); 16]; 16]; 16]);
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                cube256[y][z][x] = BlockStateId(((x + z * 16 + y * 256) % 256) as u16);
            }
        }
    }
    let palette256 = PalettedContainer::from_cube(cube256);

    let cases = [
        ("homogeneous", &homogeneous),
        ("16_entries", &palette16),
        ("64_entries", &palette64),
        ("256_entries", &palette256),
    ];

    for (name, container) in cases {
        group.bench_with_input(
            BenchmarkId::new("write", name),
            container,
            |b, container| {
                let mut sink = Vec::with_capacity(4096);
                b.iter(|| {
                    sink.clear();
                    let _ = container.write(&mut sink);
                    black_box(sink.len())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_ticket_propagation,
    bench_spatial_entity_manager,
    bench_paletted_container_write
);
criterion_main!(benches);
