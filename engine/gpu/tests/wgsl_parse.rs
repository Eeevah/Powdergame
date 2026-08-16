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
        ("pressure.wgsl", include_str!("../src/pressure.wgsl")),
    ];

    for (name, source) in shaders {
        parse(name, source);
    }
}
