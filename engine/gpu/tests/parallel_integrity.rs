//! G6 — Parallel Integrity: Write Ownership & Contention Integrity Tests.
//!
//! Verifies on the authoritative GPU simulation path (Windows + RTX 5090 + DX12):
//! - G6-A: Every production pass obeys its structural write contract (Read Neighbors, Write Self
//!   or bounded Propose -> Resolve/Claim -> Commit).
//! - G6-B: Ownership contention (movement, expansion, smoke spawn, scratch reuse, and mixed stress)
//!   preserves cell uniqueness, conservation, and world integrity without corruption.

use powdergame_core::{
    WorldConfig, FLAG_COMBUSTING, FLAG_FLAME_EVENT, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL,
    MATERIAL_SAND, MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
};
use powdergame_gpu::Simulation;

fn make_sim(config: WorldConfig) -> Simulation {
    pollster::block_on(Simulation::new(config)).expect("DX12 + RTX 5090 simulation init")
}

fn cell(sim: &Simulation, x: i64, y: i64) -> u32 {
    sim.world
        .read_material_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("cell readback")
}

fn pressure(sim: &Simulation, x: i64, y: i64) -> f32 {
    sim.world
        .read_pressure_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("pressure readback")
}

fn flags(sim: &Simulation, x: i64, y: i64) -> u32 {
    sim.world
        .read_flags_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("flags readback")
}

fn set(sim: &Simulation, x: i64, y: i64, id: u32) {
    sim.world
        .write_material(&sim.context.queue, x, y, id)
        .expect("material edit")
}

fn set_t(sim: &Simulation, x: i64, y: i64, t: f32) {
    sim.world
        .write_temperature(&sim.context.queue, x, y, t)
        .expect("temperature edit")
}

fn set_f(sim: &Simulation, x: i64, y: i64, f: u32) {
    sim.world
        .write_flags(&sim.context.queue, x, y, f)
        .expect("flags edit")
}

fn count_material(sim: &Simulation, id: u32) -> usize {
    let mats = sim
        .world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("read all materials");
    mats.iter().filter(|&&m| m == id).count()
}

// ─────────────────────────────────────────────────────────────────────────────
// G6-A: Structural Write Contract & WGSL Binding Safety Tests
// ─────────────────────────────────────────────────────────────────────────────

struct PassContract {
    name: &'static str,
    source: &'static str,
    expected_readwrite_bindings: &'static [&'static str],
}

#[test]
fn test_all_production_wgsl_write_contracts_and_binding_safety() {
    let contracts = [
        PassContract {
            name: "movement_propose.wgsl",
            source: include_str!("../src/movement_propose.wgsl"),
            expected_readwrite_bindings: &["proposal", "marker"],
        },
        PassContract {
            name: "movement_claim.wgsl",
            source: include_str!("../src/movement_claim.wgsl"),
            expected_readwrite_bindings: &["claim"],
        },
        PassContract {
            name: "movement_commit.wgsl",
            source: include_str!("../src/movement_commit.wgsl"),
            expected_readwrite_bindings: &["material_next", "temperature_next", "flags_next"],
        },
        PassContract {
            name: "thermal.wgsl",
            source: include_str!("../src/thermal.wgsl"),
            expected_readwrite_bindings: &["temperature_next"],
        },
        PassContract {
            name: "phase_transition.wgsl",
            source: include_str!("../src/phase_transition.wgsl"),
            expected_readwrite_bindings: &["material_next", "proposal", "cell_activity"],
        },
        PassContract {
            name: "expansion_claim.wgsl",
            source: include_str!("../src/expansion_claim.wgsl"),
            expected_readwrite_bindings: &["claim"],
        },
        PassContract {
            name: "expansion_spawn_commit.wgsl",
            source: include_str!("../src/expansion_spawn_commit.wgsl"),
            expected_readwrite_bindings: &["material_next", "temperature_next", "flags_next"],
        },
        PassContract {
            name: "expansion_pressure.wgsl",
            source: include_str!("../src/expansion_pressure.wgsl"),
            expected_readwrite_bindings: &["pressure_next"],
        },
        PassContract {
            name: "decay.wgsl",
            source: include_str!("../src/decay.wgsl"),
            expected_readwrite_bindings: &["material_next", "flags_next", "temperature_next"],
        },
        PassContract {
            name: "combustion.wgsl",
            source: include_str!("../src/combustion.wgsl"),
            expected_readwrite_bindings: &[
                "temperature_next",
                "flags_next",
                "proposal",
                "material_next",
            ],
        },
        PassContract {
            name: "smoke_claim.wgsl",
            source: include_str!("../src/smoke_claim.wgsl"),
            expected_readwrite_bindings: &["claim"],
        },
        PassContract {
            name: "smoke_commit.wgsl",
            source: include_str!("../src/smoke_commit.wgsl"),
            expected_readwrite_bindings: &["temperature_next", "material_next"],
        },
        PassContract {
            name: "pressure.wgsl",
            source: include_str!("../src/pressure.wgsl"),
            expected_readwrite_bindings: &["pressure_next"],
        },
        PassContract {
            name: "rupture.wgsl",
            source: include_str!("../src/rupture.wgsl"),
            expected_readwrite_bindings: &["material_next", "temperature_next", "flags_next"],
        },
        PassContract {
            name: "activity_propose.wgsl",
            source: include_str!("../src/activity_propose.wgsl"),
            expected_readwrite_bindings: &["cell_activity"],
        },
        PassContract {
            name: "activity_reduce.wgsl",
            source: include_str!("../src/activity_reduce.wgsl"),
            expected_readwrite_bindings: &[
                "chunk_activity",
                "chunk_changed_this_tick",
                "chunk_stable_ticks",
            ],
        },
    ];

    for contract in &contracts {
        let module = naga::front::wgsl::parse_str(contract.source)
            .unwrap_or_else(|err| panic!("WGSL parse failed for {}: {}", contract.name, err));

        let mut actual_readwrite = Vec::new();
        for (_handle, global) in module.global_variables.iter() {
            if let Some(name) = &global.name {
                match global.space {
                    naga::AddressSpace::Storage { access } => {
                        if access.contains(naga::StorageAccess::STORE) {
                            actual_readwrite.push(name.as_str());
                        }
                    }
                    naga::AddressSpace::WorkGroup => {
                        panic!(
                            "{}: unexpected workgroup shared variable '{}' (G6 forbids workgroup coordination)",
                            contract.name, name
                        );
                    }
                    _ => {}
                }
            }
        }

        // Verify that every actual read_write binding is authorized in expected list.
        for actual in &actual_readwrite {
            assert!(
                contract.expected_readwrite_bindings.contains(actual),
                "{}: unauthorized writable storage binding '{}'. Expected only {:?}",
                contract.name,
                actual,
                contract.expected_readwrite_bindings
            );
        }

        // Verify that every expected read_write binding actually exists.
        for expected in contract.expected_readwrite_bindings {
            assert!(
                actual_readwrite.contains(expected),
                "{}: missing expected writable storage binding '{}'. Actual: {:?}",
                contract.name,
                expected,
                actual_readwrite
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// G6-B: Movement Contention Integrity Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_movement_many_sources_one_empty_target_exactly_one_winner() {
    let mut sim = make_sim(WorldConfig::new(8, 8, 8).unwrap());

    // Enclose world with Stone so only (3, 4) is EMPTY.
    for x in 1..=6 {
        for y in 1..=6 {
            set(&sim, x, y, MATERIAL_STONE);
        }
    }

    // Shared target EMPTY at (3, 4).
    set(&sim, 3, 4, MATERIAL_EMPTY);

    // Surrounding 3 Sand particles: (3, 3) [above], (2, 3) [up-left], (4, 3) [up-right].
    // Since all other neighbors are Stone, all 3 Sand particles can ONLY target (3, 4).
    set(&sim, 3, 3, MATERIAL_SAND);
    set(&sim, 2, 3, MATERIAL_SAND);
    set(&sim, 4, 3, MATERIAL_SAND);

    assert_eq!(count_material(&sim, MATERIAL_SAND), 3);
    assert_eq!(cell(&sim, 3, 4), MATERIAL_EMPTY);

    sim.tick().expect("tick 1");

    // Exactly one winner lands at (3, 4).
    assert_eq!(cell(&sim, 3, 4), MATERIAL_SAND);

    // Total Sand count is strictly conserved.
    assert_eq!(count_material(&sim, MATERIAL_SAND), 3);

    // Exactly 2 of the original sources still contain Sand.
    let remaining_sources = [cell(&sim, 3, 3), cell(&sim, 2, 3), cell(&sim, 4, 3)]
        .iter()
        .filter(|&&m| m == MATERIAL_SAND)
        .count();
    assert_eq!(remaining_sources, 2);
}

#[test]
fn test_movement_chain_cell_joins_at_most_one_edge() {
    let mut sim = make_sim(WorldConfig::new(8, 8, 8).unwrap());

    // Column: (3, 2) Sand, (3, 3) Sand, (3, 4) EMPTY.
    // (3, 3) proposes to move to (3, 4).
    // (3, 2) sees (3, 3) is occupied during propose phase, so (3, 2) cannot move straight down.
    // Side walls prevent diagonal exits.
    set(&sim, 2, 2, MATERIAL_STONE);
    set(&sim, 4, 2, MATERIAL_STONE);
    set(&sim, 2, 3, MATERIAL_STONE);
    set(&sim, 4, 3, MATERIAL_STONE);
    set(&sim, 2, 4, MATERIAL_STONE);
    set(&sim, 4, 4, MATERIAL_STONE);

    set(&sim, 3, 2, MATERIAL_SAND);
    set(&sim, 3, 3, MATERIAL_SAND);
    set(&sim, 3, 4, MATERIAL_EMPTY);

    assert_eq!(count_material(&sim, MATERIAL_SAND), 2);

    sim.tick().expect("tick 1");

    // (3, 3) moved to (3, 4). (3, 2) stayed at (3, 2). (3, 3) became EMPTY.
    assert_eq!(cell(&sim, 3, 2), MATERIAL_SAND);
    assert_eq!(cell(&sim, 3, 3), MATERIAL_EMPTY);
    assert_eq!(cell(&sim, 3, 4), MATERIAL_SAND);
    assert_eq!(count_material(&sim, MATERIAL_SAND), 2);
}

#[test]
fn test_movement_contention_across_chunk_boundary_single_winner() {
    let mut sim = make_sim(WorldConfig::new(128, 128, 64).unwrap());

    // Chunk boundary between x=63 and x=64.
    // Build a stone funnel where the ONLY reachable empty cell for both sources is (63, 20).
    for x in 60..=67 {
        for y in 18..=22 {
            set(&sim, x, y, MATERIAL_STONE);
        }
    }

    // Shared target at (63, 20).
    set(&sim, 63, 20, MATERIAL_EMPTY);

    // Source A at (63, 19) [chunk 0], Source B at (64, 19) [chunk 1].
    set(&sim, 63, 19, MATERIAL_SAND);
    set(&sim, 64, 19, MATERIAL_SAND);

    assert_eq!(count_material(&sim, MATERIAL_SAND), 2);

    sim.tick().expect("tick 1");

    // Exactly one winner at (63, 20).
    assert_eq!(cell(&sim, 63, 20), MATERIAL_SAND);
    assert_eq!(count_material(&sim, MATERIAL_SAND), 2);

    let remaining_sources = [cell(&sim, 63, 19), cell(&sim, 64, 19)]
        .iter()
        .filter(|&&m| m == MATERIAL_SAND)
        .count();
    assert_eq!(remaining_sources, 1);
}

#[test]
fn test_movement_repeated_contention_long_run_preserves_world_integrity() {
    let mut sim = make_sim(WorldConfig::new(32, 32, 16).unwrap());

    // Build a closed V-hopper containing 30 Sand and 30 Water particles.
    for y in 5..=25 {
        set(&sim, 5, y, MATERIAL_STONE);
        set(&sim, 26, y, MATERIAL_STONE);
    }
    for x in 5..=26 {
        set(&sim, x, 25, MATERIAL_STONE);
    }

    let mut initial_sand = 0;
    let mut initial_water = 0;
    for y in 6..=11 {
        for x in 6..=15 {
            set(&sim, x, y, MATERIAL_SAND);
            initial_sand += 1;
        }
        for x in 16..=25 {
            set(&sim, x, y, MATERIAL_WATER);
            set_t(&sim, x, y, 20.0);
            initial_water += 1;
        }
    }

    assert_eq!(count_material(&sim, MATERIAL_SAND), initial_sand);
    assert_eq!(count_material(&sim, MATERIAL_WATER), initial_water);

    // Run 200 ticks of heavy parallel contention and density swaps.
    for _ in 0..200 {
        sim.tick().expect("continuous tick");
    }

    // Verify exact conservation in closed container.
    assert_eq!(count_material(&sim, MATERIAL_SAND), initial_sand);
    assert_eq!(count_material(&sim, MATERIAL_WATER), initial_water);
}

// ─────────────────────────────────────────────────────────────────────────────
// G6-B: Expansion & Scratch Reuse Contention Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_expansion_contention_many_boiling_sources_one_empty_target() {
    let mut sim = make_sim(WorldConfig::new(16, 16, 16).unwrap());

    // Enclose world with Stone.
    for x in 1..=14 {
        for y in 1..=14 {
            set(&sim, x, y, MATERIAL_STONE);
        }
    }

    // Shared EMPTY destination at (8, 8).
    set(&sim, 8, 8, MATERIAL_EMPTY);

    // 3 boiling Water sources at (8, 9), (7, 8), (9, 8) at T=75.0 (> 60.0 boil threshold).
    set(&sim, 8, 9, MATERIAL_WATER);
    set_t(&sim, 8, 9, 75.0);

    set(&sim, 7, 8, MATERIAL_WATER);
    set_t(&sim, 7, 8, 75.0);

    set(&sim, 9, 8, MATERIAL_WATER);
    set_t(&sim, 9, 8, 75.0);

    assert_eq!(count_material(&sim, MATERIAL_WATER), 3);
    assert_eq!(count_material(&sim, MATERIAL_STEAM), 0);

    sim.tick().expect("tick 1");

    // All 3 sources transform into Steam via 1:1 self-phase transition.
    // In addition, exactly ONE source wins the extra expansion spawn at (8, 8).
    assert_eq!(cell(&sim, 8, 8), MATERIAL_STEAM);
    assert_eq!(cell(&sim, 8, 9), MATERIAL_STEAM);
    assert_eq!(cell(&sim, 7, 8), MATERIAL_STEAM);
    assert_eq!(cell(&sim, 9, 8), MATERIAL_STEAM);

    // Total Steam in world must be exactly 4 (3 sources + 1 spawn).
    assert_eq!(count_material(&sim, MATERIAL_STEAM), 4);
    assert_eq!(count_material(&sim, MATERIAL_WATER), 0);

    // The losing sources must have generated confinement pressure (blocked_pressure = 100.0).
    let pressures = [
        pressure(&sim, 8, 9),
        pressure(&sim, 7, 8),
        pressure(&sim, 9, 8),
    ];
    let sources_with_confinement_pressure =
        pressures.iter().filter(|&&p| p >= 100.0 || p > 0.0).count();
    assert!(
        sources_with_confinement_pressure >= 2,
        "losing boiling sources must receive confinement pressure"
    );
}

#[test]
fn test_expansion_scratch_reuse_after_movement() {
    let mut sim = make_sim(WorldConfig::new(32, 32, 16).unwrap());

    // 1. Setup movement in top-left (Sand falling) to dirty proposal/claim scratch.
    set(&sim, 4, 4, MATERIAL_SAND);
    set(&sim, 4, 5, MATERIAL_EMPTY);

    // 2. Setup boiling Water enclosed in a stone capsule in bottom-right (2 sources, 1 target).
    for x in 18..=22 {
        for y in 18..=22 {
            set(&sim, x, y, MATERIAL_STONE);
        }
    }
    set(&sim, 20, 20, MATERIAL_EMPTY);
    set(&sim, 20, 21, MATERIAL_WATER);
    set_t(&sim, 20, 21, 75.0);
    set(&sim, 19, 20, MATERIAL_WATER);
    set_t(&sim, 19, 20, 75.0);

    sim.tick().expect("tick 1");

    // Verify movement completed.
    assert_eq!(cell(&sim, 4, 5), MATERIAL_SAND);

    // Verify expansion completed cleanly without stale movement scratch interference.
    assert_eq!(cell(&sim, 20, 20), MATERIAL_STEAM);
    assert_eq!(cell(&sim, 20, 21), MATERIAL_STEAM);
    assert_eq!(cell(&sim, 19, 20), MATERIAL_STEAM);
}

// ─────────────────────────────────────────────────────────────────────────────
// G6-B: Smoke Spawn Contention & Scratch Reuse Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_smoke_spawn_contention_multiple_burning_sources_one_empty_target() {
    let mut sim = make_sim(WorldConfig::new(16, 16, 16).unwrap());

    // Enclose with Stone.
    for x in 1..=14 {
        for y in 1..=14 {
            set(&sim, x, y, MATERIAL_STONE);
        }
    }

    // Shared EMPTY destination at (8, 7).
    set(&sim, 8, 7, MATERIAL_EMPTY);

    // 3 burning Wood sources at (8, 8), (7, 8), (9, 8) at T=150.0.
    for &x in &[7, 8, 9] {
        set(&sim, x, 8, MATERIAL_WOOD);
        set_t(&sim, x, 8, 150.0);
        set_f(&sim, x, 8, FLAG_COMBUSTING | FLAG_FLAME_EVENT);
    }

    assert_eq!(count_material(&sim, MATERIAL_SMOKE), 0);

    sim.tick().expect("tick 1");

    // Destination receives exactly one Smoke.
    assert_eq!(cell(&sim, 8, 7), MATERIAL_SMOKE);
    assert_eq!(count_material(&sim, MATERIAL_SMOKE), 1);

    // Destination Smoke decay age begins at zero (bits 16..27 = 0).
    let smoke_flags = flags(&sim, 8, 7);
    let decay_age = (smoke_flags >> 16) & 0x0FFF;
    assert_eq!(decay_age, 0, "newly spawned smoke must start with age 0");

    // All 3 Wood sources remain Wood.
    for &x in &[7, 8, 9] {
        assert_eq!(cell(&sim, x, 8), MATERIAL_WOOD);
    }
}

#[test]
fn test_smoke_scratch_reuse_after_movement_and_expansion() {
    let mut sim = make_sim(WorldConfig::new(32, 32, 16).unwrap());

    // 1. Movement active in region A.
    set(&sim, 4, 4, MATERIAL_SAND);
    set(&sim, 4, 5, MATERIAL_EMPTY);

    // 2. Expansion active in region B (enclosed in Stone).
    for x in 10..=14 {
        for y in 10..=14 {
            set(&sim, x, y, MATERIAL_STONE);
        }
    }
    set(&sim, 12, 12, MATERIAL_EMPTY);
    set(&sim, 12, 13, MATERIAL_WATER);
    set_t(&sim, 12, 13, 75.0);

    // 3. Smoke spawn active in region C (enclosed in Stone).
    for x in 22..=26 {
        for y in 22..=26 {
            set(&sim, x, y, MATERIAL_STONE);
        }
    }
    set(&sim, 24, 23, MATERIAL_EMPTY);
    set(&sim, 24, 24, MATERIAL_WOOD);
    set_t(&sim, 24, 24, 150.0);
    set_f(&sim, 24, 24, FLAG_COMBUSTING | FLAG_FLAME_EVENT);
    set(&sim, 23, 24, MATERIAL_WOOD);
    set_t(&sim, 23, 24, 150.0);
    set_f(&sim, 23, 24, FLAG_COMBUSTING | FLAG_FLAME_EVENT);

    sim.tick().expect("tick 1");

    // Movement verified.
    assert_eq!(cell(&sim, 4, 5), MATERIAL_SAND);

    // Expansion verified.
    assert_eq!(cell(&sim, 12, 12), MATERIAL_STEAM);

    // Smoke verified: exactly one smoke at (24, 23).
    assert_eq!(cell(&sim, 24, 23), MATERIAL_SMOKE);
    assert_eq!(count_material(&sim, MATERIAL_SMOKE), 1);
}

// ─────────────────────────────────────────────────────────────────────────────
// G6-B: Heavy Mixed Integrity Stress Test
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_mixed_integrity_stress_long_run() {
    let mut sim = make_sim(WorldConfig::new(64, 64, 32).unwrap());

    // Region 1 (x: 4..16, y: 4..24): Sand & Water hopper.
    for y in 4..=24 {
        set(&sim, 4, y, MATERIAL_STONE);
        set(&sim, 16, y, MATERIAL_STONE);
    }
    for x in 4..=16 {
        set(&sim, x, 24, MATERIAL_STONE);
    }
    for y in 5..=10 {
        for x in 5..=10 {
            set(&sim, x, y, MATERIAL_SAND);
        }
        for x in 11..=15 {
            set(&sim, x, y, MATERIAL_WATER);
            set_t(&sim, x, y, 20.0);
        }
    }

    // Region 2 (x: 20..32, y: 4..24): Oil & Water density column.
    for y in 4..=24 {
        set(&sim, 20, y, MATERIAL_STONE);
        set(&sim, 32, y, MATERIAL_STONE);
    }
    for x in 20..=32 {
        set(&sim, x, 24, MATERIAL_STONE);
    }
    for y in 5..=10 {
        for x in 21..=31 {
            set(&sim, x, y, MATERIAL_WATER);
            set_t(&sim, x, y, 20.0);
        }
    }
    for y in 11..=16 {
        for x in 21..=31 {
            set(&sim, x, y, MATERIAL_OIL);
            set_t(&sim, x, y, 20.0);
        }
    }

    // Region 3 (x: 36..48, y: 4..24): Burning Wood generating Smoke.
    for y in 4..=24 {
        set(&sim, 36, y, MATERIAL_STONE);
        set(&sim, 48, y, MATERIAL_STONE);
    }
    for x in 36..=48 {
        set(&sim, x, 24, MATERIAL_STONE);
    }
    for y in 20..=22 {
        for x in 38..=46 {
            set(&sim, x, y, MATERIAL_WOOD);
            set_t(&sim, x, y, 120.0);
            set_f(&sim, x, y, FLAG_COMBUSTING | FLAG_FLAME_EVENT);
        }
    }

    // Region 4 (x: 4..16, y: 32..56): Phase & Pressure boiler (Water + heater + Wood relief plug).
    for y in 32..=56 {
        set(&sim, 4, y, MATERIAL_STONE);
        set(&sim, 16, y, MATERIAL_STONE);
    }
    for x in 4..=16 {
        set(&sim, x, 56, MATERIAL_STONE);
        if (8..=12).contains(&x) {
            set(&sim, x, 32, MATERIAL_WOOD); // relief plug
        } else {
            set(&sim, x, 32, MATERIAL_STONE);
        }
    }
    // Floor heater at T=200.0.
    for x in 5..=15 {
        set(&sim, x, 55, MATERIAL_STONE);
        set_t(&sim, x, 55, 200.0);
    }
    // Water at T=55.0.
    for y in 40..=54 {
        for x in 5..=15 {
            set(&sim, x, y, MATERIAL_WATER);
            set_t(&sim, x, y, 55.0);
        }
    }

    // Region 5 (x: 20..32, y: 32..56): Ice melting zone.
    for y in 32..=56 {
        set(&sim, 20, y, MATERIAL_STONE);
        set(&sim, 32, y, MATERIAL_STONE);
    }
    for x in 20..=32 {
        set(&sim, x, 56, MATERIAL_STONE);
        set(&sim, x, 32, MATERIAL_STONE);
    }
    for y in 40..=50 {
        for x in 22..=30 {
            set(&sim, x, y, MATERIAL_ICE);
            set_t(&sim, x, y, 10.0); // Above melting threshold -10.0
        }
    }

    // Run for 300 ticks under full parallel load.
    for _ in 0..300 {
        sim.tick().expect("continuous parallel tick");
    }

    // Full-world integrity audit.
    let mats = sim
        .world
        .read_material_all(&sim.context.device, &sim.context.queue)
        .expect("read all materials");
    let temps = sim
        .world
        .read_temperature_all(&sim.context.device, &sim.context.queue)
        .expect("read all temperatures");
    let pressures = sim
        .world
        .read_pressure_all(&sim.context.device, &sim.context.queue)
        .expect("read all pressures");
    let flag_vals = sim
        .world
        .read_flags_all(&sim.context.device, &sim.context.queue)
        .expect("read all flags");

    assert_eq!(mats.len(), 64 * 64);

    for i in 0..mats.len() {
        let m = mats[i];
        let t = temps[i];
        let p = pressures[i];
        let f = flag_vals[i];

        // 1. Material ID valid (< 16).
        assert!(m < 16, "invalid material id {} at index {}", m, i);

        // 2. Temperature finite.
        assert!(
            t.is_finite() && (-100.0..=2000.0).contains(&t),
            "non-finite or runaway temperature {} at index {}",
            t,
            i
        );

        // 3. Pressure finite and non-negative.
        assert!(
            p.is_finite() && (0.0..=1.0e6).contains(&p),
            "non-finite or negative pressure {} at index {}",
            p,
            i
        );

        // 4. EMPTY hygiene.
        if m == MATERIAL_EMPTY {
            assert_eq!(
                t, 0.0,
                "EMPTY cell at index {} must have reference temperature 0.0, got {}",
                i, t
            );
            assert_eq!(
                f, 0,
                "EMPTY cell at index {} must have flags 0, got {}",
                i, f
            );
            assert_eq!(
                p, 0.0,
                "EMPTY cell at index {} must have pressure 0.0, got {}",
                i, p
            );
        }
    }
}

#[test]
fn test_production_hash_contention_varies_with_tick_but_preserves_single_edge() {
    let mut left_wins = 0;
    let mut right_wins = 0;

    for seed in 0..64u64 {
        let mut sim = make_sim(WorldConfig {
            width: 64,
            height: 64,
            chunk_size: 64,
        });
        sim.tick_count = seed;

        // Symmetric horizontal contention: Left at (31, 32), Right at (33, 32), Target at (32, 32).
        // Sand falling diagonally down into (32, 32):
        // Left sand at (31, 31), Right sand at (33, 31), Target at (32, 32)
        // Block (31, 32) and (33, 32) with Stone so down is blocked and they must take down-diagonals.
        set(&sim, 31, 32, MATERIAL_STONE);
        set(&sim, 33, 32, MATERIAL_STONE);
        set(&sim, 30, 31, MATERIAL_STONE);
        set(&sim, 34, 31, MATERIAL_STONE);
        set(&sim, 30, 32, MATERIAL_STONE);
        set(&sim, 34, 32, MATERIAL_STONE);

        set(&sim, 31, 31, MATERIAL_SAND);
        set(&sim, 33, 31, MATERIAL_SAND);

        sim.tick().expect("production tick");

        let dest = cell(&sim, 32, 32);
        assert_eq!(
            dest, MATERIAL_SAND,
            "contested target must receive exactly one Sand"
        );

        let left_src = cell(&sim, 31, 31);
        let right_src = cell(&sim, 33, 31);

        if left_src == MATERIAL_EMPTY && right_src == MATERIAL_SAND {
            left_wins += 1;
        } else if right_src == MATERIAL_EMPTY && left_src == MATERIAL_SAND {
            right_wins += 1;
        } else {
            panic!(
                "exactly one source must move: left_src={}, right_src={}",
                left_src, right_src
            );
        }

        assert_eq!(
            count_material(&sim, MATERIAL_SAND),
            2,
            "sand count strictly conserved"
        );
    }

    assert!(
        left_wins > 0 && right_wins > 0,
        "both contenders must win across tick seeds in production (got left={}, right={})",
        left_wins,
        right_wins
    );
}

#[test]
fn test_production_same_seed_determinism() {
    let run = |seed: u64| -> Vec<u32> {
        let mut sim = make_sim(WorldConfig {
            width: 64,
            height: 64,
            chunk_size: 64,
        });
        sim.tick_count = seed;

        set(&sim, 31, 32, MATERIAL_STONE);
        set(&sim, 33, 32, MATERIAL_STONE);
        set(&sim, 31, 31, MATERIAL_SAND);
        set(&sim, 33, 31, MATERIAL_SAND);

        sim.tick().expect("tick");
        sim.world
            .read_material_all(&sim.context.device, &sim.context.queue)
            .expect("read all")
    };

    let run1 = run(42);
    let run2 = run(42);
    assert_eq!(
        run1, run2,
        "identical initial state and seed must produce bit-exact results"
    );
}
