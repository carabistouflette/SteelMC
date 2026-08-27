//! Isolated workload driver for counter-level differential profiling.
//!
//! Executes a fixed explosion workload with zero Criterion overhead so hardware
//! counters attribute entirely to the measured code path.

#[cfg(feature = "benchmark-support")]
fn main() {
    use glam::DVec3;
    use steel_core::world::explosion_benchmark_support::{
        run_single_explosion, setup_benchmark_world,
    };

    const WARMUP_EXPLOSIONS: usize = 25;
    const MEASURED_EXPLOSIONS: usize = 300;

    let world = setup_benchmark_world("parity_driver");
    for _ in 0..WARMUP_EXPLOSIONS {
        run_single_explosion(&world, DVec3::new(0.5, 64.5, 0.5));
    }
    eprintln!("warmup complete");
    for _ in 0..MEASURED_EXPLOSIONS {
        run_single_explosion(&world, DVec3::new(0.5, 64.5, 0.5));
    }
}

#[cfg(not(feature = "benchmark-support"))]
fn main() {}
