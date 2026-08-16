from pathlib import Path
import textwrap

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content, encoding="utf-8", newline="\n")


def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected exactly one anchor, found {count}: {old[:80]!r}")
    write(path, text.replace(old, new, 1))


def insert_before(path: str, marker: str, insertion: str) -> None:
    replace_once(path, marker, insertion + marker)


def create_new(path: str, content: str) -> None:
    target = ROOT / path
    if target.exists():
        raise RuntimeError(f"{path}: refusing to overwrite existing file")
    target.write_text(textwrap.dedent(content).lstrip(), encoding="utf-8", newline="\n")


# ---------------------------------------------------------------------------
# Core phase grammar: data-driven Matter yield + confinement pressure metadata
# ---------------------------------------------------------------------------
phase = "engine/core/src/phase.rs"
replace_once(
    phase,
    "//! G4-B — Phase transition: temperature-based 1:1 self transitions.\n",
    "//! G4-B/G5-B — Phase transition with data-driven expansion metadata.\n",
)
replace_once(
    phase,
    "pub const WATER_BOIL_ABOVE: f32 = 60.0;\n",
    "pub const WATER_BOIL_ABOVE: f32 = 60.0;\n"
    "\n"
    "/// G5-B baseline: ordinary phase rules are identity-yield (1 cell in → 1 out).\n"
    "pub const PHASE_IDENTITY_MATTER_YIELD: u32 = 1;\n"
    "/// G5-B minimum sufficient expansion: boiling Water requests one extra Steam cell.\n"
    "pub const WATER_BOIL_MATTER_YIELD: u32 = 2;\n"
    "/// Pressure impulse created when the extra boiling yield cannot acquire space.\n"
    "/// Gameplay scalar, not SI pressure.\n"
    "pub const WATER_BOIL_BLOCKED_PRESSURE: f32 = 100.0;\n"
    "/// Current G5-B ownership path supports at most one additional Matter cell.\n"
    "pub const MAX_PHASE_MATTER_YIELD: u32 = 2;\n",
)
replace_once(
    phase,
    "pub struct PhaseTransition {\n"
    "    pub condition: TemperatureCondition,\n"
    "    pub threshold: f32,\n"
    "    pub target_material: u32,\n"
    "}\n",
    "pub struct PhaseTransition {\n"
    "    pub condition: TemperatureCondition,\n"
    "    pub threshold: f32,\n"
    "    pub target_material: u32,\n"
    "    /// Total Matter cells requested by this transition, including self.\n"
    "    pub matter_yield: u32,\n"
    "    /// Pressure added at the source when requested extra yield is unresolved.\n"
    "    pub blocked_pressure: f32,\n"
    "}\n",
)
replace_once(
    phase,
    "pub struct PhaseGpuDescriptor {\n"
    "    pub below_target: u32,\n"
    "    pub above_target: u32,\n"
    "    pub below_threshold: f32,\n"
    "    pub above_threshold: f32,\n"
    "}\n",
    "pub struct PhaseGpuDescriptor {\n"
    "    pub below_target: u32,\n"
    "    pub above_target: u32,\n"
    "    pub below_yield: u32,\n"
    "    pub above_yield: u32,\n"
    "    pub below_threshold: f32,\n"
    "    pub above_threshold: f32,\n"
    "    pub below_blocked_pressure: f32,\n"
    "    pub above_blocked_pressure: f32,\n"
    "}\n",
)
insert_before(
    phase,
    "/// Pure reference: selects the phase target for `material_id` at\n",
    "/// Full selected phase effect used by G5-B expansion/confinement.\n"
    "#[derive(Debug, Clone, Copy, PartialEq)]\n"
    "pub struct PhaseEffect {\n"
    "    pub target_material: u32,\n"
    "    pub matter_yield: u32,\n"
    "    pub blocked_pressure: f32,\n"
    "}\n"
    "\n"
    "/// Selects the first matching Material-owned phase effect.\n"
    "pub fn select_phase_effect(material_id: u32, temperature: f32) -> Option<PhaseEffect> {\n"
    "    let rules = registry_lookup(material_id)?.phase_transitions;\n"
    "    let t = sanitize_temperature(temperature);\n"
    "    for rule in rules {\n"
    "        let hit = match rule.condition {\n"
    "            TemperatureCondition::Below => t < rule.threshold,\n"
    "            TemperatureCondition::Above => t > rule.threshold,\n"
    "        };\n"
    "        if hit {\n"
    "            return Some(PhaseEffect {\n"
    "                target_material: rule.target_material,\n"
    "                matter_yield: rule.matter_yield,\n"
    "                blocked_pressure: rule.blocked_pressure,\n"
    "            });\n"
    "        }\n"
    "    }\n"
    "    None\n"
    "}\n"
    "\n",
)
replace_once(
    phase,
    "pub fn select_phase_transition(material_id: u32, temperature: f32) -> Option<u32> {\n"
    "    let rules = registry_lookup(material_id)?.phase_transitions;\n"
    "    let t = sanitize_temperature(temperature);\n"
    "    for rule in rules {\n"
    "        let hit = match rule.condition {\n"
    "            TemperatureCondition::Below => t < rule.threshold,\n"
    "            TemperatureCondition::Above => t > rule.threshold,\n"
    "        };\n"
    "        if hit {\n"
    "            return Some(rule.target_material);\n"
    "        }\n"
    "    }\n"
    "    None\n"
    "}\n",
    "pub fn select_phase_transition(material_id: u32, temperature: f32) -> Option<u32> {\n"
    "    select_phase_effect(material_id, temperature).map(|effect| effect.target_material)\n"
    "}\n",
)
replace_once(
    phase,
    "    let none = PhaseGpuDescriptor {\n"
    "        below_target: NO_PHASE_TARGET,\n"
    "        above_target: NO_PHASE_TARGET,\n"
    "        below_threshold: 0.0,\n"
    "        above_threshold: 0.0,\n"
    "    };\n",
    "    let none = PhaseGpuDescriptor {\n"
    "        below_target: NO_PHASE_TARGET,\n"
    "        above_target: NO_PHASE_TARGET,\n"
    "        below_yield: PHASE_IDENTITY_MATTER_YIELD,\n"
    "        above_yield: PHASE_IDENTITY_MATTER_YIELD,\n"
    "        below_threshold: 0.0,\n"
    "        above_threshold: 0.0,\n"
    "        below_blocked_pressure: 0.0,\n"
    "        above_blocked_pressure: 0.0,\n"
    "    };\n",
)
replace_once(
    phase,
    "                    desc.below_target = rule.target_material;\n"
    "                    desc.below_threshold = rule.threshold;\n"
    "                    below_seen = true;\n",
    "                    desc.below_target = rule.target_material;\n"
    "                    desc.below_yield = rule.matter_yield;\n"
    "                    desc.below_threshold = rule.threshold;\n"
    "                    desc.below_blocked_pressure = rule.blocked_pressure;\n"
    "                    below_seen = true;\n",
)
replace_once(
    phase,
    "                    desc.above_target = rule.target_material;\n"
    "                    desc.above_threshold = rule.threshold;\n"
    "                    above_seen = true;\n",
    "                    desc.above_target = rule.target_material;\n"
    "                    desc.above_yield = rule.matter_yield;\n"
    "                    desc.above_threshold = rule.threshold;\n"
    "                    desc.above_blocked_pressure = rule.blocked_pressure;\n"
    "                    above_seen = true;\n",
)
insert_before(
    phase,
    "    #[test]\n    fn ice_thermal_properties_are_sane() {\n",
    "    #[test]\n"
    "    fn boiling_effect_requests_expansion_and_confinement_pressure() {\n"
    "        let effect = select_phase_effect(MATERIAL_WATER, 70.0).unwrap();\n"
    "        assert_eq!(effect.target_material, MATERIAL_STEAM);\n"
    "        assert_eq!(effect.matter_yield, WATER_BOIL_MATTER_YIELD);\n"
    "        assert_eq!(effect.blocked_pressure, WATER_BOIL_BLOCKED_PRESSURE);\n"
    "    }\n"
    "\n"
    "    #[test]\n"
    "    fn non_expanding_phase_rules_keep_identity_yield() {\n"
    "        for (material, t) in [\n"
    "            (MATERIAL_WATER, -30.0),\n"
    "            (MATERIAL_ICE, 0.0),\n"
    "            (MATERIAL_STEAM, 30.0),\n"
    "        ] {\n"
    "            let effect = select_phase_effect(material, t).unwrap();\n"
    "            assert_eq!(effect.matter_yield, PHASE_IDENTITY_MATTER_YIELD);\n"
    "            assert_eq!(effect.blocked_pressure, 0.0);\n"
    "        }\n"
    "    }\n"
    "\n"
    "    #[test]\n"
    "    fn phase_descriptor_carries_g5b_metadata() {\n"
    "        let table = phase_descriptor_table();\n"
    "        let water = table[MATERIAL_WATER as usize];\n"
    "        assert_eq!(water.above_yield, WATER_BOIL_MATTER_YIELD);\n"
    "        assert_eq!(water.above_blocked_pressure, WATER_BOIL_BLOCKED_PRESSURE);\n"
    "        assert_eq!(water.below_yield, PHASE_IDENTITY_MATTER_YIELD);\n"
    "        assert_eq!(water.below_blocked_pressure, 0.0);\n"
    "    }\n"
    "\n"
    "    #[test]\n"
    "    fn registered_phase_yields_fit_g5b_single_extra_cell_path() {\n"
    "        for material in crate::material::MATERIAL_REGISTRY {\n"
    "            for rule in material.phase_transitions {\n"
    "                assert!(rule.matter_yield >= 1);\n"
    "                assert!(rule.matter_yield <= MAX_PHASE_MATTER_YIELD);\n"
    "                assert!(rule.blocked_pressure.is_finite());\n"
    "                assert!(rule.blocked_pressure >= 0.0);\n"
    "                if rule.matter_yield == PHASE_IDENTITY_MATTER_YIELD {\n"
    "                    assert_eq!(rule.blocked_pressure, 0.0);\n"
    "                }\n"
    "            }\n"
    "        }\n"
    "    }\n"
    "\n",
)

# ---------------------------------------------------------------------------
# Material-owned phase metadata
# ---------------------------------------------------------------------------
material = "engine/core/src/material.rs"
replace_once(
    material,
    "                target_material: MATERIAL_ICE,\n"
    "            },\n"
    "            PhaseTransition {\n"
    "                condition: TemperatureCondition::Above,\n"
    "                threshold: crate::phase::WATER_BOIL_ABOVE,\n"
    "                target_material: MATERIAL_STEAM,\n"
    "            },\n",
    "                target_material: MATERIAL_ICE,\n"
    "                matter_yield: crate::phase::PHASE_IDENTITY_MATTER_YIELD,\n"
    "                blocked_pressure: 0.0,\n"
    "            },\n"
    "            PhaseTransition {\n"
    "                condition: TemperatureCondition::Above,\n"
    "                threshold: crate::phase::WATER_BOIL_ABOVE,\n"
    "                target_material: MATERIAL_STEAM,\n"
    "                matter_yield: crate::phase::WATER_BOIL_MATTER_YIELD,\n"
    "                blocked_pressure: crate::phase::WATER_BOIL_BLOCKED_PRESSURE,\n"
    "            },\n",
)
replace_once(
    material,
    "        phase_transitions: &[PhaseTransition {\n"
    "            condition: TemperatureCondition::Below,\n"
    "            threshold: crate::phase::STEAM_CONDENSE_BELOW,\n"
    "            target_material: MATERIAL_WATER,\n"
    "        }],\n",
    "        phase_transitions: &[PhaseTransition {\n"
    "            condition: TemperatureCondition::Below,\n"
    "            threshold: crate::phase::STEAM_CONDENSE_BELOW,\n"
    "            target_material: MATERIAL_WATER,\n"
    "            matter_yield: crate::phase::PHASE_IDENTITY_MATTER_YIELD,\n"
    "            blocked_pressure: 0.0,\n"
    "        }],\n",
)
replace_once(
    material,
    "        phase_transitions: &[PhaseTransition {\n"
    "            condition: TemperatureCondition::Above,\n"
    "            threshold: crate::phase::ICE_MELT_ABOVE,\n"
    "            target_material: MATERIAL_WATER,\n"
    "        }],\n",
    "        phase_transitions: &[PhaseTransition {\n"
    "            condition: TemperatureCondition::Above,\n"
    "            threshold: crate::phase::ICE_MELT_ABOVE,\n"
    "            target_material: MATERIAL_WATER,\n"
    "            matter_yield: crate::phase::PHASE_IDENTITY_MATTER_YIELD,\n"
    "            blocked_pressure: 0.0,\n"
    "        }],\n",
)

# Public core exports.
lib = "engine/core/src/lib.rs"
replace_once(
    lib,
    "pub use phase::{\n"
    "    is_phase_candidate, phase_descriptor_table, select_phase_transition, PhaseGpuDescriptor,\n"
    "    PhaseTransition, TemperatureCondition, ICE_MELT_ABOVE, NO_PHASE_TARGET, STEAM_CONDENSE_BELOW,\n"
    "    WATER_BOIL_ABOVE, WATER_FREEZE_BELOW,\n"
    "};\n",
    "pub use phase::{\n"
    "    is_phase_candidate, phase_descriptor_table, select_phase_effect, select_phase_transition,\n"
    "    PhaseEffect, PhaseGpuDescriptor, PhaseTransition, TemperatureCondition, ICE_MELT_ABOVE,\n"
    "    MAX_PHASE_MATTER_YIELD, NO_PHASE_TARGET, PHASE_IDENTITY_MATTER_YIELD,\n"
    "    STEAM_CONDENSE_BELOW, WATER_BOIL_ABOVE, WATER_BOIL_BLOCKED_PRESSURE,\n"
    "    WATER_BOIL_MATTER_YIELD, WATER_FREEZE_BELOW,\n"
    "};\n",
)

# ---------------------------------------------------------------------------
# Phase shader now emits one local expansion proposal for yield=2 transitions
# ---------------------------------------------------------------------------
write(
    "engine/gpu/src/phase_transition.wgsl",
    textwrap.dedent(r'''
    // G4-B + G5-B — Phase self-transition plus expansion proposal.
    //
    // The phase identity transform remains Write Self. If the selected
    // Material-owned rule requests matter_yield=2, the same invocation also
    // writes only proposal[self], choosing at most one local EMPTY target.
    // Destination ownership is resolved by the following expansion claim pass.

    struct Params {
        cell_count: u32,
        threads_x: u32,
        width: u32,
        height: u32,
    };

    struct PhaseDesc {
        below_target: u32,
        above_target: u32,
        below_yield: u32,
        above_yield: u32,
        below_threshold: f32,
        above_threshold: f32,
        below_blocked_pressure: f32,
        above_blocked_pressure: f32,
    };

    @group(0) @binding(0) var<uniform> params: Params;
    @group(0) @binding(1) var<storage, read> material_current: array<u32>;
    @group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
    @group(0) @binding(3) var<storage, read> phase_table: array<PhaseDesc, 16>;
    @group(0) @binding(4) var<storage, read_write> material_next: array<u32>;
    @group(0) @binding(5) var<storage, read_write> proposal: array<u32>;

    const EMPTY: u32 = 0u;
    const NO_PHASE_TARGET: u32 = 0xFFFFFFFFu;
    const NO_PROPOSAL: u32 = 0u;
    const BLOCKED_EXPANSION: u32 = 0xFFFFFFFFu;
    const TEMPERATURE_REFERENCE: f32 = 0.0;

    fn sanitize_temperature(t: f32) -> f32 {
        if (t != t) {
            return TEMPERATURE_REFERENCE;
        }
        if (t > 1.0e20 || t < -1.0e20) {
            return TEMPERATURE_REFERENCE;
        }
        return t;
    }

    fn in_domain(x: i32, y: i32) -> bool {
        return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
    }

    fn candidate(x: i32, y: i32) -> u32 {
        if (!in_domain(x, y)) {
            return NO_PROPOSAL;
        }
        let idx = u32(y) * params.width + u32(x);
        if (material_current[idx] == EMPTY) {
            return idx + 1u;
        }
        return NO_PROPOSAL;
    }

    // Local 8-neighbor First-Match. Upward cells are preferred so boiling
    // expansion composes naturally with the following GAS movement without
    // any long-range scan or special boiler code.
    fn find_expansion_target(index: u32) -> u32 {
        let x = i32(index % params.width);
        let y = i32(index / params.width);
        var p = candidate(x, y - 1);
        if (p != NO_PROPOSAL) { return p; }
        p = candidate(x - 1, y - 1);
        if (p != NO_PROPOSAL) { return p; }
        p = candidate(x + 1, y - 1);
        if (p != NO_PROPOSAL) { return p; }
        p = candidate(x - 1, y);
        if (p != NO_PROPOSAL) { return p; }
        p = candidate(x + 1, y);
        if (p != NO_PROPOSAL) { return p; }
        p = candidate(x - 1, y + 1);
        if (p != NO_PROPOSAL) { return p; }
        p = candidate(x + 1, y + 1);
        if (p != NO_PROPOSAL) { return p; }
        p = candidate(x, y + 1);
        if (p != NO_PROPOSAL) { return p; }
        return BLOCKED_EXPANSION;
    }

    @compute @workgroup_size(64, 1, 1)
    fn phase_main(@builtin(global_invocation_id) gid: vec3<u32>) {
        let index = gid.y * params.threads_x + gid.x;
        if (index >= params.cell_count) {
            return;
        }

        proposal[index] = NO_PROPOSAL;
        let mat = material_current[index];
        if (mat == EMPTY || mat >= 16u) {
            material_next[index] = mat;
            return;
        }

        let desc = phase_table[mat];
        let t = sanitize_temperature(temperature_current[index]);
        var next_mat = mat;
        var matter_yield = 1u;
        if (desc.below_target != NO_PHASE_TARGET && t < desc.below_threshold) {
            next_mat = desc.below_target;
            matter_yield = desc.below_yield;
        } else if (desc.above_target != NO_PHASE_TARGET && t > desc.above_threshold) {
            next_mat = desc.above_target;
            matter_yield = desc.above_yield;
        }

        material_next[index] = next_mat;
        if (next_mat != mat && matter_yield > 1u) {
            // G5-B baseline supports one additional cell (yield=2). Unknown
            // larger yields fail closed into confinement pressure rather than
            // silently writing multiple neighbors.
            if (matter_yield == 2u) {
                proposal[index] = find_expansion_target(index);
            } else {
                proposal[index] = BLOCKED_EXPANSION;
            }
        }
    }
    ''').lstrip(),
)

# ---------------------------------------------------------------------------
# New GPU ownership passes
# ---------------------------------------------------------------------------
create_new(
    "engine/gpu/src/expansion_claim.wgsl",
    r'''
    // G5-B — phase expansion destination Claim/Resolve.
    // Each EMPTY destination reads only its 8-neighborhood and chooses the
    // smallest source index whose proposal targets this cell. claim[c]=source+1.

    struct Params {
        cell_count: u32,
        threads_x: u32,
        width: u32,
        height: u32,
    };

    @group(0) @binding(0) var<uniform> params: Params;
    @group(0) @binding(1) var<storage, read> material_current: array<u32>;
    @group(0) @binding(2) var<storage, read> proposal: array<u32>;
    @group(0) @binding(3) var<storage, read_write> claim: array<u32>;

    const EMPTY: u32 = 0u;
    const NO_CLAIM: u32 = 0u;
    const NO_SOURCE: u32 = 0xFFFFFFFFu;

    @compute @workgroup_size(64, 1, 1)
    fn expansion_claim_main(@builtin(global_invocation_id) gid: vec3<u32>) {
        let c = gid.y * params.threads_x + gid.x;
        if (c >= params.cell_count) {
            return;
        }
        claim[c] = NO_CLAIM;
        if (material_current[c] != EMPTY) {
            return;
        }

        let x = i32(c % params.width);
        let y = i32(c / params.width);
        var best = NO_SOURCE;
        var dy: i32 = -1;
        while (dy <= 1) {
            var dx: i32 = -1;
            while (dx <= 1) {
                if (!(dx == 0 && dy == 0)) {
                    let nx = x + dx;
                    let ny = y + dy;
                    if (nx >= 0 && ny >= 0 && nx < i32(params.width) && ny < i32(params.height)) {
                        let s = u32(ny) * params.width + u32(nx);
                        if (proposal[s] == c + 1u && s < best) {
                            best = s;
                        }
                    }
                }
                dx = dx + 1;
            }
            dy = dy + 1;
        }

        if (best != NO_SOURCE) {
            claim[c] = best + 1u;
        }
    }
    ''',
)

create_new(
    "engine/gpu/src/expansion_spawn_commit.wgsl",
    r'''
    // G5-B — winning expansion destination commits one extra Matter cell.
    // Reads the source's already-computed phase result and writes only self.

    struct Params {
        cell_count: u32,
        threads_x: u32,
        width: u32,
        height: u32,
    };

    @group(0) @binding(0) var<uniform> params: Params;
    @group(0) @binding(1) var<storage, read> material_current: array<u32>;
    @group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
    @group(0) @binding(3) var<storage, read> claim: array<u32>;
    @group(0) @binding(4) var<storage, read_write> material_next: array<u32>;
    @group(0) @binding(5) var<storage, read_write> temperature_next: array<f32>;
    @group(0) @binding(6) var<storage, read_write> flags_next: array<u32>;

    const EMPTY: u32 = 0u;

    @compute @workgroup_size(64, 1, 1)
    fn expansion_spawn_commit_main(@builtin(global_invocation_id) gid: vec3<u32>) {
        let c = gid.y * params.threads_x + gid.x;
        if (c >= params.cell_count) {
            return;
        }
        let winner = claim[c];
        if (winner == 0u || material_current[c] != EMPTY) {
            return;
        }
        let source = winner - 1u;
        material_next[c] = material_next[source];
        temperature_next[c] = temperature_current[source];
        flags_next[c] = 0u;
    }
    ''',
)

create_new(
    "engine/gpu/src/expansion_pressure.wgsl",
    r'''
    // G5-B — unresolved phase expansion becomes scalar Pressure at source.
    // Successful claims add no pressure. Blocked requests and claim losers
    // receive the Material-owned blocked_pressure impulse. Write Self only.

    struct Params {
        cell_count: u32,
        threads_x: u32,
        width: u32,
        height: u32,
    };

    struct PhaseDesc {
        below_target: u32,
        above_target: u32,
        below_yield: u32,
        above_yield: u32,
        below_threshold: f32,
        above_threshold: f32,
        below_blocked_pressure: f32,
        above_blocked_pressure: f32,
    };

    struct PhaseEffect {
        active: u32,
        matter_yield: u32,
        blocked_pressure: f32,
    };

    @group(0) @binding(0) var<uniform> params: Params;
    @group(0) @binding(1) var<storage, read> material_current: array<u32>;
    @group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
    @group(0) @binding(3) var<storage, read> phase_table: array<PhaseDesc, 16>;
    @group(0) @binding(4) var<storage, read> proposal: array<u32>;
    @group(0) @binding(5) var<storage, read> claim: array<u32>;
    @group(0) @binding(6) var<storage, read> pressure_current: array<f32>;
    @group(0) @binding(7) var<storage, read_write> pressure_next: array<f32>;

    const EMPTY: u32 = 0u;
    const NO_PHASE_TARGET: u32 = 0xFFFFFFFFu;
    const NO_PROPOSAL: u32 = 0u;
    const BLOCKED_EXPANSION: u32 = 0xFFFFFFFFu;
    const PRESSURE_REFERENCE: f32 = 0.0;
    const PRESSURE_MAX: f32 = 1.0e6;

    fn sanitize_temperature(t: f32) -> f32 {
        if (t != t || t > 1.0e20 || t < -1.0e20) {
            return 0.0;
        }
        return t;
    }

    fn sanitize_pressure(p: f32) -> f32 {
        if (p != p || p > 1.0e20 || p < -1.0e20) {
            return PRESSURE_REFERENCE;
        }
        return clamp(p, PRESSURE_REFERENCE, PRESSURE_MAX);
    }

    fn selected_effect(mat: u32, t: f32) -> PhaseEffect {
        var effect = PhaseEffect(0u, 1u, 0.0);
        if (mat == EMPTY || mat >= 16u) {
            return effect;
        }
        let desc = phase_table[mat];
        if (desc.below_target != NO_PHASE_TARGET && t < desc.below_threshold) {
            effect.active = 1u;
            effect.matter_yield = desc.below_yield;
            effect.blocked_pressure = desc.below_blocked_pressure;
        } else if (desc.above_target != NO_PHASE_TARGET && t > desc.above_threshold) {
            effect.active = 1u;
            effect.matter_yield = desc.above_yield;
            effect.blocked_pressure = desc.above_blocked_pressure;
        }
        return effect;
    }

    @compute @workgroup_size(64, 1, 1)
    fn expansion_pressure_main(@builtin(global_invocation_id) gid: vec3<u32>) {
        let c = gid.y * params.threads_x + gid.x;
        if (c >= params.cell_count) {
            return;
        }

        let p0 = sanitize_pressure(pressure_current[c]);
        let effect = selected_effect(material_current[c], sanitize_temperature(temperature_current[c]));
        var impulse = 0.0;

        if (effect.active != 0u && effect.matter_yield > 1u) {
            let request = proposal[c];
            var succeeded = false;
            if (request != NO_PROPOSAL && request != BLOCKED_EXPANSION) {
                let destination = request - 1u;
                if (destination < params.cell_count && claim[destination] == c + 1u) {
                    succeeded = true;
                }
            }
            if (!succeeded) {
                impulse = max(effect.blocked_pressure, 0.0);
            }
        }

        pressure_next[c] = sanitize_pressure(p0 + impulse);
    }
    ''',
)

# ---------------------------------------------------------------------------
# Simulation wiring
# ---------------------------------------------------------------------------
sim = "engine/gpu/src/simulation.rs"
replace_once(
    sim,
    "/// Phase descriptor table: 16 descriptors × 16 bytes.\nconst PHASE_TABLE_SIZE: u64 = 256;\n",
    "/// Phase descriptor table: 16 descriptors × 32 bytes (G5-B yield/confinement metadata).\n"
    "const PHASE_TABLE_SIZE: u64 = 512;\n",
)
replace_once(
    sim,
    "    phase_pipeline: wgpu::ComputePipeline,\n    decay_pipeline: wgpu::ComputePipeline,\n",
    "    phase_pipeline: wgpu::ComputePipeline,\n"
    "    expansion_claim_pipeline: wgpu::ComputePipeline,\n"
    "    expansion_spawn_commit_pipeline: wgpu::ComputePipeline,\n"
    "    expansion_pressure_pipeline: wgpu::ComputePipeline,\n"
    "    decay_pipeline: wgpu::ComputePipeline,\n",
)
replace_once(
    sim,
    "    phase_bind_group: wgpu::BindGroup,\n    decay_bind_group: wgpu::BindGroup,\n",
    "    phase_bind_group: wgpu::BindGroup,\n"
    "    expansion_claim_bind_group: wgpu::BindGroup,\n"
    "    expansion_spawn_commit_bind_group: wgpu::BindGroup,\n"
    "    expansion_pressure_bind_group: wgpu::BindGroup,\n"
    "    decay_bind_group: wgpu::BindGroup,\n",
)
replace_once(
    sim,
    "        let shader_phase = context\n"
    "            .device\n"
    "            .create_shader_module(wgpu::ShaderModuleDescriptor {\n"
    "                label: Some(\"powdergame-g4b-phase\"),\n"
    "                source: wgpu::ShaderSource::Wgsl(include_str!(\"phase_transition.wgsl\").into()),\n"
    "            });\n",
    "        let shader_phase = context\n"
    "            .device\n"
    "            .create_shader_module(wgpu::ShaderModuleDescriptor {\n"
    "                label: Some(\"powdergame-g4b-g5b-phase\"),\n"
    "                source: wgpu::ShaderSource::Wgsl(include_str!(\"phase_transition.wgsl\").into()),\n"
    "            });\n"
    "        let shader_expansion_claim = context\n"
    "            .device\n"
    "            .create_shader_module(wgpu::ShaderModuleDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-claim\"),\n"
    "                source: wgpu::ShaderSource::Wgsl(include_str!(\"expansion_claim.wgsl\").into()),\n"
    "            });\n"
    "        let shader_expansion_spawn_commit = context\n"
    "            .device\n"
    "            .create_shader_module(wgpu::ShaderModuleDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-spawn-commit\"),\n"
    "                source: wgpu::ShaderSource::Wgsl(include_str!(\"expansion_spawn_commit.wgsl\").into()),\n"
    "            });\n"
    "        let shader_expansion_pressure = context\n"
    "            .device\n"
    "            .create_shader_module(wgpu::ShaderModuleDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-pressure\"),\n"
    "                source: wgpu::ShaderSource::Wgsl(include_str!(\"expansion_pressure.wgsl\").into()),\n"
    "            });\n",
)
replace_once(
    sim,
    "        let phase_layout =\n"
    "            context\n"
    "                .device\n"
    "                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {\n"
    "                    label: Some(\"powdergame-g4b-phase-bgl\"),\n"
    "                    entries: &[\n"
    "                        buffer_entry(0, &BindingKind::Uniform),\n"
    "                        buffer_entry(1, &BindingKind::Read), // material_current\n"
    "                        buffer_entry(2, &BindingKind::Read), // temperature_current\n"
    "                        buffer_entry(3, &BindingKind::Read), // phase_table\n"
    "                        buffer_entry(4, &BindingKind::ReadWrite), // material_next\n"
    "                    ],\n"
    "                });\n",
    "        let phase_layout =\n"
    "            context\n"
    "                .device\n"
    "                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {\n"
    "                    label: Some(\"powdergame-g4b-g5b-phase-bgl\"),\n"
    "                    entries: &[\n"
    "                        buffer_entry(0, &BindingKind::Uniform),\n"
    "                        buffer_entry(1, &BindingKind::Read), // material_current\n"
    "                        buffer_entry(2, &BindingKind::Read), // temperature_current\n"
    "                        buffer_entry(3, &BindingKind::Read), // phase_table\n"
    "                        buffer_entry(4, &BindingKind::ReadWrite), // material_next\n"
    "                        buffer_entry(5, &BindingKind::ReadWrite), // expansion proposal\n"
    "                    ],\n"
    "                });\n"
    "        let expansion_claim_layout = context.device.create_bind_group_layout(\n"
    "            &wgpu::BindGroupLayoutDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-claim-bgl\"),\n"
    "                entries: &[\n"
    "                    buffer_entry(0, &BindingKind::Uniform),\n"
    "                    buffer_entry(1, &BindingKind::Read), // material_current\n"
    "                    buffer_entry(2, &BindingKind::Read), // proposal\n"
    "                    buffer_entry(3, &BindingKind::ReadWrite), // claim\n"
    "                ],\n"
    "            },\n"
    "        );\n"
    "        let expansion_spawn_commit_layout = context.device.create_bind_group_layout(\n"
    "            &wgpu::BindGroupLayoutDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-spawn-commit-bgl\"),\n"
    "                entries: &[\n"
    "                    buffer_entry(0, &BindingKind::Uniform),\n"
    "                    buffer_entry(1, &BindingKind::Read), // material_current\n"
    "                    buffer_entry(2, &BindingKind::Read), // temperature_current\n"
    "                    buffer_entry(3, &BindingKind::Read), // claim\n"
    "                    buffer_entry(4, &BindingKind::ReadWrite), // material_next\n"
    "                    buffer_entry(5, &BindingKind::ReadWrite), // temperature_next\n"
    "                    buffer_entry(6, &BindingKind::ReadWrite), // flags_next\n"
    "                ],\n"
    "            },\n"
    "        );\n"
    "        let expansion_pressure_layout = context.device.create_bind_group_layout(\n"
    "            &wgpu::BindGroupLayoutDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-pressure-bgl\"),\n"
    "                entries: &[\n"
    "                    buffer_entry(0, &BindingKind::Uniform),\n"
    "                    buffer_entry(1, &BindingKind::Read), // material_current\n"
    "                    buffer_entry(2, &BindingKind::Read), // temperature_current\n"
    "                    buffer_entry(3, &BindingKind::Read), // phase_table\n"
    "                    buffer_entry(4, &BindingKind::Read), // proposal\n"
    "                    buffer_entry(5, &BindingKind::Read), // claim\n"
    "                    buffer_entry(6, &BindingKind::Read), // pressure_current\n"
    "                    buffer_entry(7, &BindingKind::ReadWrite), // pressure_next\n"
    "                ],\n"
    "            },\n"
    "        );\n",
)
replace_once(
    sim,
    "        let phase_pipeline = make_pipeline(\n"
    "            \"powdergame-g4b-phase\",\n"
    "            &phase_layout,\n"
    "            &shader_phase,\n"
    "            \"phase_main\",\n"
    "        );\n",
    "        let phase_pipeline = make_pipeline(\n"
    "            \"powdergame-g4b-g5b-phase\",\n"
    "            &phase_layout,\n"
    "            &shader_phase,\n"
    "            \"phase_main\",\n"
    "        );\n"
    "        let expansion_claim_pipeline = make_pipeline(\n"
    "            \"powdergame-g5b-expansion-claim\",\n"
    "            &expansion_claim_layout,\n"
    "            &shader_expansion_claim,\n"
    "            \"expansion_claim_main\",\n"
    "        );\n"
    "        let expansion_spawn_commit_pipeline = make_pipeline(\n"
    "            \"powdergame-g5b-expansion-spawn-commit\",\n"
    "            &expansion_spawn_commit_layout,\n"
    "            &shader_expansion_spawn_commit,\n"
    "            \"expansion_spawn_commit_main\",\n"
    "        );\n"
    "        let expansion_pressure_pipeline = make_pipeline(\n"
    "            \"powdergame-g5b-expansion-pressure\",\n"
    "            &expansion_pressure_layout,\n"
    "            &shader_expansion_pressure,\n"
    "            \"expansion_pressure_main\",\n"
    "        );\n",
)
replace_once(
    sim,
    "        // G4-B phase descriptor table (16 × 16 bytes; Material data, not\n"
    "        // per-cell state). Compiled from each Material's ordered rules.\n"
    "        let mut phase_data = [0u8; PHASE_TABLE_SIZE as usize];\n"
    "        for (i, desc) in phase_descriptor_table().iter().enumerate() {\n"
    "            let off = i * 16;\n"
    "            phase_data[off..off + 4].copy_from_slice(&desc.below_target.to_ne_bytes());\n"
    "            phase_data[off + 4..off + 8].copy_from_slice(&desc.above_target.to_ne_bytes());\n"
    "            phase_data[off + 8..off + 12].copy_from_slice(&desc.below_threshold.to_ne_bytes());\n"
    "            phase_data[off + 12..off + 16].copy_from_slice(&desc.above_threshold.to_ne_bytes());\n"
    "        }\n",
    "        // G4-B/G5-B phase descriptor table (16 × 32 bytes; Material data,\n"
    "        // not per-cell state): targets + matter yield + thresholds +\n"
    "        // confinement pressure. No per-cell expansion buffer is added.\n"
    "        let mut phase_data = [0u8; PHASE_TABLE_SIZE as usize];\n"
    "        for (i, desc) in phase_descriptor_table().iter().enumerate() {\n"
    "            let off = i * 32;\n"
    "            phase_data[off..off + 4].copy_from_slice(&desc.below_target.to_ne_bytes());\n"
    "            phase_data[off + 4..off + 8].copy_from_slice(&desc.above_target.to_ne_bytes());\n"
    "            phase_data[off + 8..off + 12].copy_from_slice(&desc.below_yield.to_ne_bytes());\n"
    "            phase_data[off + 12..off + 16].copy_from_slice(&desc.above_yield.to_ne_bytes());\n"
    "            phase_data[off + 16..off + 20].copy_from_slice(&desc.below_threshold.to_ne_bytes());\n"
    "            phase_data[off + 20..off + 24].copy_from_slice(&desc.above_threshold.to_ne_bytes());\n"
    "            phase_data[off + 24..off + 28]\n"
    "                .copy_from_slice(&desc.below_blocked_pressure.to_ne_bytes());\n"
    "            phase_data[off + 28..off + 32]\n"
    "                .copy_from_slice(&desc.above_blocked_pressure.to_ne_bytes());\n"
    "        }\n",
)
replace_once(
    sim,
    "                    wgpu::BindGroupEntry {\n"
    "                        binding: 4,\n"
    "                        resource: world.material_next.as_entire_binding(),\n"
    "                    },\n"
    "                ],\n"
    "            });\n"
    "        let decay_bind_group = context\n",
    "                    wgpu::BindGroupEntry {\n"
    "                        binding: 4,\n"
    "                        resource: world.material_next.as_entire_binding(),\n"
    "                    },\n"
    "                    wgpu::BindGroupEntry {\n"
    "                        binding: 5,\n"
    "                        resource: world.proposal.as_entire_binding(),\n"
    "                    },\n"
    "                ],\n"
    "            });\n"
    "        let expansion_claim_bind_group = context.device.create_bind_group(\n"
    "            &wgpu::BindGroupDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-claim-bg\"),\n"
    "                layout: &expansion_claim_layout,\n"
    "                entries: &[\n"
    "                    wgpu::BindGroupEntry { binding: 0, resource: params.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 1, resource: world.material_current.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 2, resource: world.proposal.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 3, resource: world.claim.as_entire_binding() },\n"
    "                ],\n"
    "            },\n"
    "        );\n"
    "        let expansion_spawn_commit_bind_group = context.device.create_bind_group(\n"
    "            &wgpu::BindGroupDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-spawn-commit-bg\"),\n"
    "                layout: &expansion_spawn_commit_layout,\n"
    "                entries: &[\n"
    "                    wgpu::BindGroupEntry { binding: 0, resource: params.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 1, resource: world.material_current.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 2, resource: world.temperature_current.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 3, resource: world.claim.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 4, resource: world.material_next.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 5, resource: world.temperature_next.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 6, resource: world.flags_next.as_entire_binding() },\n"
    "                ],\n"
    "            },\n"
    "        );\n"
    "        let expansion_pressure_bind_group = context.device.create_bind_group(\n"
    "            &wgpu::BindGroupDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-pressure-bg\"),\n"
    "                layout: &expansion_pressure_layout,\n"
    "                entries: &[\n"
    "                    wgpu::BindGroupEntry { binding: 0, resource: params.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 1, resource: world.material_current.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 2, resource: world.temperature_current.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 3, resource: phase_table_buf.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 4, resource: world.proposal.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 5, resource: world.claim.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 6, resource: world.pressure_current.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 7, resource: world.pressure_next.as_entire_binding() },\n"
    "                ],\n"
    "            },\n"
    "        );\n"
    "        let decay_bind_group = context\n",
)
replace_once(
    sim,
    "            phase_pipeline,\n            decay_pipeline,\n",
    "            phase_pipeline,\n"
    "            expansion_claim_pipeline,\n"
    "            expansion_spawn_commit_pipeline,\n"
    "            expansion_pressure_pipeline,\n"
    "            decay_pipeline,\n",
)
replace_once(
    sim,
    "            phase_bind_group,\n            decay_bind_group,\n",
    "            phase_bind_group,\n"
    "            expansion_claim_bind_group,\n"
    "            expansion_spawn_commit_bind_group,\n"
    "            expansion_pressure_bind_group,\n"
    "            decay_bind_group,\n",
)
replace_once(
    sim,
    "        {\n"
    "            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {\n"
    "                label: Some(\"powdergame-g4b-phase-pass\"),\n"
    "                timestamp_writes: None,\n"
    "            });\n"
    "            dispatch(&mut pass, &self.phase_pipeline, &self.phase_bind_group);\n"
    "        }\n"
    "        encoder.copy_buffer_to_buffer(\n"
    "            &self.world.material_next,\n"
    "            0,\n"
    "            &self.world.material_current,\n"
    "            0,\n"
    "            self.world.layout.material_bytes,\n"
    "        );\n",
    "        {\n"
    "            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {\n"
    "                label: Some(\"powdergame-g4b-g5b-phase-pass\"),\n"
    "                timestamp_writes: None,\n"
    "            });\n"
    "            dispatch(&mut pass, &self.phase_pipeline, &self.phase_bind_group);\n"
    "        }\n"
    "        {\n"
    "            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-claim-pass\"),\n"
    "                timestamp_writes: None,\n"
    "            });\n"
    "            dispatch(&mut pass, &self.expansion_claim_pipeline, &self.expansion_claim_bind_group);\n"
    "        }\n"
    "        {\n"
    "            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-spawn-commit-pass\"),\n"
    "                timestamp_writes: None,\n"
    "            });\n"
    "            dispatch(\n"
    "                &mut pass,\n"
    "                &self.expansion_spawn_commit_pipeline,\n"
    "                &self.expansion_spawn_commit_bind_group,\n"
    "            );\n"
    "        }\n"
    "        {\n"
    "            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {\n"
    "                label: Some(\"powdergame-g5b-expansion-pressure-pass\"),\n"
    "                timestamp_writes: None,\n"
    "            });\n"
    "            dispatch(\n"
    "                &mut pass,\n"
    "                &self.expansion_pressure_pipeline,\n"
    "                &self.expansion_pressure_bind_group,\n"
    "            );\n"
    "        }\n"
    "        // Phase identity + any won expansion spawn become authoritative\n"
    "        // together. Unresolved expansion pressure is visible to the G5-A\n"
    "        // propagation pass later in the same tick.\n"
    "        encoder.copy_buffer_to_buffer(\n"
    "            &self.world.material_next,\n"
    "            0,\n"
    "            &self.world.material_current,\n"
    "            0,\n"
    "            self.world.layout.material_bytes,\n"
    "        );\n"
    "        encoder.copy_buffer_to_buffer(\n"
    "            &self.world.temperature_next,\n"
    "            0,\n"
    "            &self.world.temperature_current,\n"
    "            0,\n"
    "            self.world.layout.temperature_bytes,\n"
    "        );\n"
    "        encoder.copy_buffer_to_buffer(\n"
    "            &self.world.flags_next,\n"
    "            0,\n"
    "            &self.world.flags_current,\n"
    "            0,\n"
    "            self.world.layout.flags_bytes,\n"
    "        );\n"
    "        encoder.copy_buffer_to_buffer(\n"
    "            &self.world.pressure_next,\n"
    "            0,\n"
    "            &self.world.pressure_current,\n"
    "            0,\n"
    "            self.world.layout.pressure_bytes,\n"
    "        );\n",
)

# Keep the high-level pipeline contract accurate.
replace_once(
    sim,
    "    /// → phase transition (self-write) → copy material Next→Current\n",
    "    /// → phase transition + expansion proposal → expansion claim/commit\n"
    "    /// → unresolved expansion → pressure impulse → copy phase state Current\n",
)
replace_once(
    sim,
    "    /// runs last on settled Matter. Expansion generation / rupture remain\n"
    "    /// G5-B/G5-C.\n",
    "    /// runs last on settled Matter. G5-B expansion/confinement feeds it;\n"
    "    /// structural stress/rupture remains G5-C.\n",
)

# ---------------------------------------------------------------------------
# GPU-free WGSL parser includes all new production shaders
# ---------------------------------------------------------------------------
wgsl_test = "engine/gpu/tests/wgsl_parse.rs"
replace_once(
    wgsl_test,
    "        (\"pressure.wgsl\", include_str!(\"../src/pressure.wgsl\")),\n",
    "        (\n"
    "            \"expansion_claim.wgsl\",\n"
    "            include_str!(\"../src/expansion_claim.wgsl\"),\n"
    "        ),\n"
    "        (\n"
    "            \"expansion_spawn_commit.wgsl\",\n"
    "            include_str!(\"../src/expansion_spawn_commit.wgsl\"),\n"
    "        ),\n"
    "        (\n"
    "            \"expansion_pressure.wgsl\",\n"
    "            include_str!(\"../src/expansion_pressure.wgsl\"),\n"
    "        ),\n"
    "        (\"pressure.wgsl\", include_str!(\"../src/pressure.wgsl\")),\n",
)

# ---------------------------------------------------------------------------
# G5-B real-GPU integration tests
# ---------------------------------------------------------------------------
create_new(
    "engine/gpu/tests/expansion.rs",
    r'''
    //! G5-B — phase expansion / confinement → Pressure GPU integration tests.
    //!
    //! Requires production Windows + RTX 5090 + DX12 through `Simulation::new`.

    use powdergame_core::{
        WorldConfig, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_STEAM, MATERIAL_STONE,
        MATERIAL_WATER, WATER_BOIL_BLOCKED_PRESSURE,
    };
    use powdergame_gpu::Simulation;

    fn make_sim(config: WorldConfig) -> Simulation {
        pollster::block_on(Simulation::new(config)).expect("DX12 + RTX 5090 simulation init")
    }

    fn eight_by_eight() -> Simulation {
        make_sim(WorldConfig::new(8, 8, 8).unwrap())
    }

    fn set_mat(sim: &Simulation, x: i64, y: i64, material: u32) {
        sim.world
            .write_material(&sim.context.queue, x, y, material)
            .expect("material edit");
    }

    fn set_t(sim: &Simulation, x: i64, y: i64, value: f32) {
        sim.world
            .write_temperature(&sim.context.queue, x, y, value)
            .expect("temperature edit");
    }

    fn cell(sim: &Simulation, x: i64, y: i64) -> u32 {
        sim.world
            .read_material_cell(&sim.context.device, &sim.context.queue, x, y)
            .expect("material readback")
    }

    fn temp(sim: &Simulation, x: i64, y: i64) -> f32 {
        sim.world
            .read_temperature_cell(&sim.context.device, &sim.context.queue, x, y)
            .expect("temperature readback")
    }

    fn pressure(sim: &Simulation, x: i64, y: i64) -> f32 {
        sim.world
            .read_pressure_cell(&sim.context.device, &sim.context.queue, x, y)
            .expect("pressure readback")
    }

    fn clear_region(sim: &Simulation, x0: i64, y0: i64, x1: i64, y1: i64) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                set_mat(sim, x, y, MATERIAL_EMPTY);
            }
        }
    }

    fn seal_eight(sim: &Simulation, x: i64, y: i64) {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx != 0 || dy != 0 {
                    set_mat(sim, x + dx, y + dy, MATERIAL_STONE);
                }
            }
        }
    }

    #[test]
    fn boiling_with_space_spawns_second_steam_without_pressure() {
        let mut sim = eight_by_eight();
        clear_region(&sim, 1, 1, 6, 6);
        seal_eight(&sim, 3, 3);
        set_mat(&sim, 3, 2, MATERIAL_EMPTY); // first expansion candidate (up)
        set_mat(&sim, 3, 3, MATERIAL_WATER);
        set_t(&sim, 3, 3, 1000.0);

        sim.tick().expect("tick");

        assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM);
        assert_eq!(cell(&sim, 3, 2), MATERIAL_STEAM);
        assert_eq!(pressure(&sim, 3, 3), 0.0);
        assert_eq!(pressure(&sim, 3, 2), 0.0);
        let source_t = temp(&sim, 3, 3);
        let spawn_t = temp(&sim, 3, 2);
        assert!(source_t > 60.0);
        assert!((source_t - spawn_t).abs() < 1.0e-3, "source={source_t} spawn={spawn_t}");
    }

    #[test]
    fn fully_confined_boiling_generates_pressure_instead_of_extra_matter() {
        let mut sim = eight_by_eight();
        clear_region(&sim, 1, 1, 6, 6);
        seal_eight(&sim, 3, 3);
        set_mat(&sim, 3, 3, MATERIAL_WATER);
        set_t(&sim, 3, 3, 1000.0);

        sim.tick().expect("tick");

        assert_eq!(cell(&sim, 3, 3), MATERIAL_STEAM);
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx != 0 || dy != 0 {
                    assert_ne!(cell(&sim, 3 + dx, 3 + dy), MATERIAL_STEAM);
                }
            }
        }
        let p = pressure(&sim, 3, 3);
        assert!(
            (p - WATER_BOIL_BLOCKED_PRESSURE).abs() < 1.0e-3,
            "blocked pressure={p}"
        );
    }

    #[test]
    fn competing_expansions_have_one_winner_and_loser_becomes_pressure() {
        let mut sim = eight_by_eight();
        clear_region(&sim, 1, 1, 6, 6);

        // Two Water cells can only expand into shared up-diagonal (4,3).
        for (x, y) in [
            (3, 3), (2, 3), // A: up, up-left blocked; up-right is target
            (5, 3),         // B: up blocked; up-left is target
            (2, 4), (4, 4), (6, 4),
            (2, 5), (3, 5), (4, 5), (5, 5), (6, 5),
        ] {
            set_mat(&sim, x, y, MATERIAL_STONE);
        }
        set_mat(&sim, 4, 3, MATERIAL_EMPTY);
        set_mat(&sim, 3, 4, MATERIAL_WATER); // lower row-major source index wins
        set_mat(&sim, 5, 4, MATERIAL_WATER);
        set_t(&sim, 3, 4, 1000.0);
        set_t(&sim, 5, 4, 1000.0);

        sim.tick().expect("tick");

        assert_eq!(cell(&sim, 3, 4), MATERIAL_STEAM);
        assert_eq!(cell(&sim, 5, 4), MATERIAL_STEAM);
        assert_eq!(cell(&sim, 4, 3), MATERIAL_STEAM, "exactly one destination winner");
        assert_eq!(pressure(&sim, 3, 4), 0.0, "smallest source won claim");
        let loser_p = pressure(&sim, 5, 4);
        assert!(
            (loser_p - WATER_BOIL_BLOCKED_PRESSURE).abs() < 1.0e-3,
            "claim loser pressure={loser_p}"
        );
    }

    #[test]
    fn expansion_can_cross_a_64_cell_chunk_boundary() {
        let mut sim = make_sim(WorldConfig::new(16, 128, 64).unwrap());
        clear_region(&sim, 5, 61, 11, 67);
        seal_eight(&sim, 8, 64);
        set_mat(&sim, 8, 63, MATERIAL_EMPTY); // up target lies in previous y chunk
        set_mat(&sim, 8, 64, MATERIAL_WATER);
        set_t(&sim, 8, 64, 1000.0);

        sim.tick().expect("tick");

        assert_eq!(cell(&sim, 8, 64), MATERIAL_STEAM);
        assert_eq!(cell(&sim, 8, 63), MATERIAL_STEAM);
        assert_eq!(pressure(&sim, 8, 64), 0.0);
    }

    #[test]
    fn one_to_one_phase_transition_creates_no_expansion_pressure() {
        let mut sim = eight_by_eight();
        clear_region(&sim, 1, 1, 6, 6);
        set_mat(&sim, 3, 3, MATERIAL_ICE);
        set_t(&sim, 3, 3, 100.0);

        sim.tick().expect("tick");

        assert_eq!(cell(&sim, 3, 3), MATERIAL_WATER);
        assert_eq!(pressure(&sim, 3, 3), 0.0);
        let materials = sim
            .world
            .read_material_all(&sim.context.device, &sim.context.queue)
            .expect("material readback");
        assert_eq!(materials.iter().filter(|&&m| m == MATERIAL_WATER).count(), 1);
    }
    ''',
)

# ---------------------------------------------------------------------------
# Update two G4 phase regression fixtures whose old assumptions were
# intentionally superseded by G5-B expansion semantics.
# ---------------------------------------------------------------------------
phase_gpu = "engine/gpu/tests/phase.rs"
replace_once(
    phase_gpu,
    "//! Temperature is preserved across the 1:1 transform (latent heat is out of\n"
    "//! scope). 1 Water cell → 1 Steam cell: no expansion/spawn (G5).\n",
    "//! Temperature is preserved across the source transform (latent heat is out\n"
    "//! of scope). G5-B extends boiling with a data-driven extra Steam request;\n"
    "//! sealed fixtures still isolate the original phase identity contract.\n",
)
replace_once(
    phase_gpu,
    "    assert_eq!(cell(&sim, 3, 3), MATERIAL_EMPTY, \"source vacated\");\n"
    "    assert_eq!(\n"
    "        temp(&sim, 3, 3),\n"
    "        TEMPERATURE_REFERENCE,\n"
    "        \"no ghost heat at the source\"\n"
    "    );\n"
    "    assert_eq!(\n"
    "        cell(&sim, 3, 4),\n"
    "        MATERIAL_STEAM,\n"
    "        \"water moved down, then boiled at the destination\"\n"
    "    );\n"
    "    assert_eq!(count_material(&sim, MATERIAL_STEAM), 1);\n"
    "    assert_eq!(count_material(&sim, MATERIAL_WATER), 0);\n"
    "    assert_eq!(matter_count(&sim), before);\n"
    "    let dest_t = temp(&sim, 3, 4);\n"
    "    assert!(\n"
    "        (dest_t - 80.0).abs() < 1.0e-3,\n"
    "        \"the hot state must be carried to the new cell; got {dest_t}\"\n"
    "    );\n",
    "    assert_eq!(\n"
    "        cell(&sim, 3, 4),\n"
    "        MATERIAL_STEAM,\n"
    "        \"water moved down, then boiled at the destination\"\n"
    "    );\n"
    "    assert_eq!(\n"
    "        cell(&sim, 3, 3),\n"
    "        MATERIAL_STEAM,\n"
    "        \"G5-B expansion reuses the newly vacated source cell\"\n"
    "    );\n"
    "    assert_eq!(count_material(&sim, MATERIAL_STEAM), 2);\n"
    "    assert_eq!(count_material(&sim, MATERIAL_WATER), 0);\n"
    "    assert_eq!(matter_count(&sim), before + 1);\n"
    "    let dest_t = temp(&sim, 3, 4);\n"
    "    let spawn_t = temp(&sim, 3, 3);\n"
    "    assert!(\n"
    "        (dest_t - 80.0).abs() < 1.0e-3,\n"
    "        \"the hot state must be carried to the new cell; got {dest_t}\"\n"
    "    );\n"
    "    assert!((spawn_t - dest_t).abs() < 1.0e-3);\n",
)
replace_once(
    phase_gpu,
    "    // Stone ring below/lateral: down (3,4), down-diagonals (2,4),(4,4),\n"
    "    // laterals (2,3),(4,3). Up (3,2) stays EMPTY for the Steam to rise.\n"
    "    set(&sim, 3, 4, MATERIAL_STONE);\n"
    "    set(&sim, 2, 4, MATERIAL_STONE);\n"
    "    set(&sim, 4, 4, MATERIAL_STONE);\n"
    "    set(&sim, 2, 3, MATERIAL_STONE);\n"
    "    set(&sim, 4, 3, MATERIAL_STONE);\n"
    "    let before = matter_count(&sim);\n",
    "    // Seal all eight neighbors on tick 1 so G5-B expansion is deliberately\n"
    "    // confined; after boiling we open only (3,2) to test GAS movement.\n"
    "    box_seal(&sim, 3, 3);\n"
    "    let before = matter_count(&sim);\n",
)
replace_once(
    phase_gpu,
    "    // Tick 2: the GAS identity actually rises one cell.\n"
    "    sim.tick().expect(\"tick\");\n",
    "    // Open one cell only after the blocked boiling tick; this keeps the\n"
    "    // historical MovementClass adoption test independent of G5-B spawn.\n"
    "    set(&sim, 3, 2, MATERIAL_EMPTY);\n"
    "    let after_open = matter_count(&sim);\n"
    "\n"
    "    // Tick 2: the GAS identity actually rises one cell.\n"
    "    sim.tick().expect(\"tick\");\n",
)
replace_once(
    phase_gpu,
    "    assert_eq!(\n"
    "        matter_count(&sim),\n"
    "        before,\n"
    "        \"matter conserved across both ticks\"\n"
    "    );\n"
    "}\n"
    "\n"
    "// ── Chunk boundary ──────────────────────────────────────────────────────\n",
    "    assert_eq!(\n"
    "        matter_count(&sim),\n"
    "        after_open,\n"
    "        \"opening the fixture changes the authored Stone count, but movement conserves Matter\"\n"
    "    );\n"
    "    assert_eq!(after_open + 1, before);\n"
    "}\n"
    "\n"
    "// ── Chunk boundary ──────────────────────────────────────────────────────\n",
)

print("G5-B expansion/confinement implementation applied")
