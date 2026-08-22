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
        (
            "phase_energy_reconcile_movement.wgsl",
            include_str!("../src/phase_energy_reconcile_movement.wgsl"),
        ),
        (
            "air_flow_scale.wgsl",
            include_str!("../src/air_flow_scale.wgsl"),
        ),
        (
            "air_transport_commit.wgsl",
            include_str!("../src/air_transport_commit.wgsl"),
        ),
        (
            "thermal_stability_scale.wgsl",
            include_str!("../src/thermal_stability_scale.wgsl"),
        ),
        (
            "unified_thermal_commit.wgsl",
            include_str!("../src/unified_thermal_commit.wgsl"),
        ),
        (
            "phase_transition.wgsl",
            include_str!("../src/phase_transition.wgsl"),
        ),
        (
            "phase_context_propose.wgsl",
            include_str!("../src/phase_context_propose.wgsl"),
        ),
        (
            "phase_energy_hygiene_identity.wgsl",
            include_str!("../src/phase_energy_hygiene_identity.wgsl"),
        ),
        ("decay.wgsl", include_str!("../src/decay.wgsl")),
        (
            "ignition_exposure_propose.wgsl",
            include_str!("../src/ignition_exposure_propose.wgsl"),
        ),
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
        (
            "pressure_activity_propose.wgsl",
            include_str!("../src/pressure_activity_propose.wgsl"),
        ),
        ("rupture.wgsl", include_str!("../src/rupture.wgsl")),
        (
            "activity_propose.wgsl",
            include_str!("../src/activity_propose.wgsl"),
        ),
        (
            "phase_activity_propose.wgsl",
            include_str!("../src/phase_activity_propose.wgsl"),
        ),
        (
            "activity_reduce.wgsl",
            include_str!("../src/activity_reduce.wgsl"),
        ),
        (
            "environment_activity_propose.wgsl",
            include_str!("../src/environment_activity_propose.wgsl"),
        ),
        (
            "ignition_activity_propose.wgsl",
            include_str!("../src/ignition_activity_propose.wgsl"),
        ),
    ];

    for (name, source) in shaders {
        parse(name, source);
    }
}

#[test]
fn te4i_passes_stay_within_the_eight_storage_binding_limit() {
    for (name, source) in [
        (
            "ignition_exposure_propose.wgsl",
            include_str!("../src/ignition_exposure_propose.wgsl"),
        ),
        ("combustion.wgsl", include_str!("../src/combustion.wgsl")),
        (
            "ignition_activity_propose.wgsl",
            include_str!("../src/ignition_activity_propose.wgsl"),
        ),
    ] {
        let module = naga::front::wgsl::parse_str(source).unwrap();
        let storage_bindings = module
            .global_variables
            .iter()
            .filter(|(_, variable)| matches!(variable.space, naga::AddressSpace::Storage { .. }))
            .count();
        assert!(
            storage_bindings <= 8,
            "{name} declares {storage_bindings} storage bindings"
        );
    }
}

#[test]
fn te5r1_pressure_update_stays_air_independent_but_rupture_reads_air_energy() {
    let pressure = include_str!("../src/pressure.wgsl");
    assert!(!pressure.contains("air_mass") && !pressure.contains("air_energy"));
    let rupture = include_str!("../src/rupture.wgsl");
    assert!(rupture.contains("air_energy_current"));
    assert!(!rupture.contains("air_mass_current"));
}

#[test]
fn te5r1_pressure_activity_has_one_production_owner() {
    let base = include_str!("../src/activity_propose.wgsl");
    let pressure = include_str!("../src/pressure_activity_propose.wgsl");
    assert!(!base.contains("const ACTIVITY_PRESSURE"));
    assert!(!base.contains("PRESSURE_BIT"));
    assert!(pressure.contains("const PRESSURE_BIT:u32=4u"));
    assert_eq!(pressure.matches("|=PRESSURE_BIT").count(), 1);
    assert!(!pressure.contains("chunk_state"));
}

#[test]
fn te5r1_changed_passes_stay_within_eight_storage_bindings() {
    for (name, source) in [
        ("pressure.wgsl", include_str!("../src/pressure.wgsl")),
        (
            "pressure_activity_propose.wgsl",
            include_str!("../src/pressure_activity_propose.wgsl"),
        ),
        ("rupture.wgsl", include_str!("../src/rupture.wgsl")),
        (
            "air_flow_scale.wgsl",
            include_str!("../src/air_flow_scale.wgsl"),
        ),
        (
            "air_transport_commit.wgsl",
            include_str!("../src/air_transport_commit.wgsl"),
        ),
        (
            "activity_propose.wgsl",
            include_str!("../src/activity_propose.wgsl"),
        ),
        (
            "environment_activity_propose.wgsl",
            include_str!("../src/environment_activity_propose.wgsl"),
        ),
    ] {
        let module = naga::front::wgsl::parse_str(source).unwrap();
        let count = module
            .global_variables
            .iter()
            .filter(|(_, variable)| matches!(variable.space, naga::AddressSpace::Storage { .. }))
            .count();
        assert!(count <= 8, "{name} declares {count} storage bindings");
    }
}

#[test]
fn te2_passes_stay_within_the_eight_storage_binding_limit() {
    for (name, source) in [
        (
            "air_flow_scale.wgsl",
            include_str!("../src/air_flow_scale.wgsl"),
        ),
        (
            "air_transport_commit.wgsl",
            include_str!("../src/air_transport_commit.wgsl"),
        ),
        (
            "thermal_stability_scale.wgsl",
            include_str!("../src/thermal_stability_scale.wgsl"),
        ),
        (
            "unified_thermal_commit.wgsl",
            include_str!("../src/unified_thermal_commit.wgsl"),
        ),
        (
            "environment_activity_propose.wgsl",
            include_str!("../src/environment_activity_propose.wgsl"),
        ),
    ] {
        let module = naga::front::wgsl::parse_str(source).unwrap();
        let storage_bindings = module
            .global_variables
            .iter()
            .filter(|(_, variable)| matches!(variable.space, naga::AddressSpace::Storage { .. }))
            .count();
        assert!(
            storage_bindings <= 8,
            "{name} declares {storage_bindings} storage bindings"
        );
    }
}

#[test]
fn te3_passes_stay_within_the_eight_storage_binding_limit() {
    for (name, source) in [
        (
            "phase_energy_reconcile_movement.wgsl",
            include_str!("../src/phase_energy_reconcile_movement.wgsl"),
        ),
        (
            "phase_context_propose.wgsl",
            include_str!("../src/phase_context_propose.wgsl"),
        ),
        (
            "phase_transition.wgsl",
            include_str!("../src/phase_transition.wgsl"),
        ),
        (
            "phase_energy_hygiene_identity.wgsl",
            include_str!("../src/phase_energy_hygiene_identity.wgsl"),
        ),
        (
            "phase_activity_propose.wgsl",
            include_str!("../src/phase_activity_propose.wgsl"),
        ),
    ] {
        let module = naga::front::wgsl::parse_str(source).unwrap();
        let storage_bindings = module
            .global_variables
            .iter()
            .filter(|(_, variable)| matches!(variable.space, naga::AddressSpace::Storage { .. }))
            .count();
        assert!(
            storage_bindings <= 8,
            "{name} declares {storage_bindings} storage bindings"
        );
    }
}
