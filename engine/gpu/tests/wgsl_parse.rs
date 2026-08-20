//! GPU-free WGSL syntax regression test.
//!
//! `cargo check` only sees WGSL through `include_str!` and therefore does not
//! parse shader syntax. Keep this test in CI so reserved keywords and other
//! parser-level WGSL failures are caught before running on the RTX 5090.

fn parse(name: &str, source: &str) {
    naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|err| panic!("WGSL parse failed for {name}: {err}"));
}

#[test]
fn all_production_wgsl_parses_without_a_gpu() {
    let shaders = [
        (
            "movement_propose.wgsl",
            include_str!("../src/movement_propose.wgsl"),
        ),
        (
            "movement_claim.wgsl",
            include_str!("../src/movement_claim.wgsl"),
        ),
        (
            "movement_commit.wgsl",
            include_str!("../src/movement_commit.wgsl"),
        ),
        (
            "material_flag_hygiene.wgsl",
            include_str!("../src/material_flag_hygiene.wgsl"),
        ),
        (
            "environment_reconcile_movement.wgsl",
            include_str!("../src/environment_reconcile_movement.wgsl"),
        ),
        ("thermal.wgsl", include_str!("../src/thermal.wgsl")),
        (
            "phase_transition.wgsl",
            include_str!("../src/phase_transition.wgsl"),
        ),
        ("decay.wgsl", include_str!("../src/decay.wgsl")),
        ("combustion.wgsl", include_str!("../src/combustion.wgsl")),
        ("smoke_claim.wgsl", include_str!("../src/smoke_claim.wgsl")),
        (
            "smoke_commit.wgsl",
            include_str!("../src/smoke_commit.wgsl"),
        ),
        (
            "expansion_claim.wgsl",
            include_str!("../src/expansion_claim.wgsl"),
        ),
        (
            "expansion_spawn_commit.wgsl",
            include_str!("../src/expansion_spawn_commit.wgsl"),
        ),
        (
            "expansion_pressure.wgsl",
            include_str!("../src/expansion_pressure.wgsl"),
        ),
        (
            "environment_receiver_claim.wgsl",
            include_str!("../src/environment_receiver_claim.wgsl"),
        ),
        (
            "environment_blocked_expansion_pressure.wgsl",
            include_str!("../src/environment_blocked_expansion_pressure.wgsl"),
        ),
        (
            "environment_reconcile_spawn.wgsl",
            include_str!("../src/environment_reconcile_spawn.wgsl"),
        ),
        (
            "environment_reconcile_identity.wgsl",
            include_str!("../src/environment_reconcile_identity.wgsl"),
        ),
        ("pressure.wgsl", include_str!("../src/pressure.wgsl")),
        ("rupture.wgsl", include_str!("../src/rupture.wgsl")),
        (
            "activity_propose.wgsl",
            include_str!("../src/activity_propose.wgsl"),
        ),
        (
            "activity_reduce.wgsl",
            include_str!("../src/activity_reduce.wgsl"),
        ),
    ];

    for (name, source) in shaders {
        parse(name, source);
    }
}

#[test]
fn te1_keeps_air_out_of_thermal_and_pressure_coupling() {
    for (name, source) in [
        ("thermal.wgsl", include_str!("../src/thermal.wgsl")),
        ("pressure.wgsl", include_str!("../src/pressure.wgsl")),
        ("rupture.wgsl", include_str!("../src/rupture.wgsl")),
    ] {
        assert!(
            !source.contains("air_mass") && !source.contains("air_energy"),
            "{name} must not couple Matter thermal/pressure physics to Air in TE-1"
        );
    }
}
