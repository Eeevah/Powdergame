from pathlib import Path
import textwrap

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: str, old: str, new: str) -> None:
    p = ROOT / path
    text = p.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected exactly one anchor, found {count}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8")


def write_new(path: str, content: str) -> None:
    p = ROOT / path
    if p.exists():
        current = p.read_text(encoding="utf-8")
        if current == content:
            return
        raise RuntimeError(f"{path}: refusing to overwrite unexpected existing file")
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content, encoding="utf-8")


PRESSURE_RS = r'''//! G5-A pressure field baseline — CPU reference rule.
//!
//! Pressure is a spatial per-cell `f32` field (`SIMULATION_SPEC` §15), not
//! Matter-owned state. The baseline is deliberately small:
//! - scalar pressure only (no pressure velocity vector),
//! - 4-neighbor local propagation,
//! - only LIQUID/GAS Matter acts as a pressure medium,
//! - EMPTY/Void and STATIC/POWDER do not secretly transmit pressure,
//! - no arbitrary time decay: an isolated pressured medium retains pressure,
//! - finite/non-negative sanitization prevents NaN/Infinity runaway.
//!
//! G5-B will generate pressure from blocked phase expansion. G5-C will use
//! pressure gradients to influence Matter and stress/rupture structures.

use crate::material::{movement_class, MovementClass};

/// Neutral pressure for cells that cannot host the field.
pub const PRESSURE_REFERENCE: f32 = 0.0;

/// Explicit 4-neighbor diffusion coefficient. Must stay <= 0.25 for the
/// symmetric four-neighbor explicit update to avoid overshoot.
pub const PRESSURE_DIFFUSION_RATE: f32 = 0.20;

/// Gameplay safety clamp, not a physical unit.
pub const PRESSURE_MAX: f32 = 1.0e6;

/// One orthogonal pressure sample. `None` represents Void/out-of-domain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressureNeighbor {
    pub material: u32,
    pub pressure: f32,
}

/// Pressure propagates only through actual Liquid/Gas Matter in G5-A.
pub fn is_pressure_medium(material: u32) -> bool {
    matches!(
        movement_class(material),
        Some(MovementClass::Liquid | MovementClass::Gas)
    )
}

/// Collapses invalid pressure to the neutral value and bounds valid values.
pub fn sanitize_pressure(value: f32) -> f32 {
    if !value.is_finite() {
        PRESSURE_REFERENCE
    } else {
        value.clamp(PRESSURE_REFERENCE, PRESSURE_MAX)
    }
}

/// One Read-Neighbors / Write-Self pressure update.
///
/// Only pressure-media neighbors participate. There is no implicit loss term;
/// a sealed isolated Liquid/Gas cell therefore keeps its pressure exactly.
pub fn pressure_step(
    self_material: u32,
    self_pressure: f32,
    neighbors: [Option<PressureNeighbor>; 4],
) -> f32 {
    if !is_pressure_medium(self_material) {
        return PRESSURE_REFERENCE;
    }

    let self_p = sanitize_pressure(self_pressure);
    let mut acc = 0.0f32;
    for neighbor in neighbors.into_iter().flatten() {
        if !is_pressure_medium(neighbor.material) {
            continue;
        }
        let neighbor_p = sanitize_pressure(neighbor.pressure);
        acc += neighbor_p - self_p;
    }

    sanitize_pressure(self_p + PRESSURE_DIFFUSION_RATE * acc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{
        MATERIAL_EMPTY, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER,
    };

    fn right(material: u32, pressure: f32) -> [Option<PressureNeighbor>; 4] {
        [
            None,
            None,
            Some(PressureNeighbor { material, pressure }),
            None,
        ]
    }

    #[test]
    fn only_liquid_and_gas_are_pressure_media() {
        assert!(is_pressure_medium(MATERIAL_WATER));
        assert!(is_pressure_medium(MATERIAL_STEAM));
        assert!(!is_pressure_medium(MATERIAL_STONE));
        assert!(!is_pressure_medium(MATERIAL_EMPTY));
    }

    #[test]
    fn pressure_moves_down_gradient_without_spontaneous_loss() {
        let hot = pressure_step(MATERIAL_WATER, 100.0, right(MATERIAL_WATER, 0.0));
        let cold = pressure_step(MATERIAL_WATER, 0.0, right(MATERIAL_WATER, 100.0));
        assert!((hot - 80.0).abs() < 1.0e-5, "hot={hot}");
        assert!((cold - 20.0).abs() < 1.0e-5, "cold={cold}");
        assert!(((hot + cold) - 100.0).abs() < 1.0e-5);
    }

    #[test]
    fn isolated_pressure_does_not_decay_with_time() {
        let next = pressure_step(MATERIAL_STEAM, 42.0, [None, None, None, None]);
        assert_eq!(next, 42.0);
    }

    #[test]
    fn empty_and_static_do_not_transmit_pressure() {
        let through_empty = pressure_step(MATERIAL_WATER, 12.0, right(MATERIAL_EMPTY, 100.0));
        let through_stone = pressure_step(MATERIAL_WATER, 12.0, right(MATERIAL_STONE, 100.0));
        assert_eq!(through_empty, 12.0);
        assert_eq!(through_stone, 12.0);
        assert_eq!(pressure_step(MATERIAL_EMPTY, 99.0, [None; 4]), PRESSURE_REFERENCE);
        assert_eq!(pressure_step(MATERIAL_STONE, 99.0, [None; 4]), PRESSURE_REFERENCE);
    }

    #[test]
    fn four_neighbor_update_is_stable() {
        let zero = Some(PressureNeighbor {
            material: MATERIAL_WATER,
            pressure: 0.0,
        });
        let next = pressure_step(MATERIAL_WATER, 100.0, [zero; 4]);
        assert!((next - 20.0).abs() < 1.0e-5, "next={next}");
    }

    #[test]
    fn invalid_values_are_sanitized() {
        assert_eq!(sanitize_pressure(f32::NAN), PRESSURE_REFERENCE);
        assert_eq!(sanitize_pressure(f32::INFINITY), PRESSURE_REFERENCE);
        assert_eq!(sanitize_pressure(f32::NEG_INFINITY), PRESSURE_REFERENCE);
        assert_eq!(sanitize_pressure(-4.0), PRESSURE_REFERENCE);
        assert_eq!(sanitize_pressure(PRESSURE_MAX * 2.0), PRESSURE_MAX);
    }
}
'''

PRESSURE_WGSL = r'''// G5-A scalar pressure baseline. Read Neighbors → Write Self.
// Constants must match engine/core/src/pressure.rs.
// EMPTY/Void and STATIC/POWDER are not hidden pressure media.

struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> pressure_current: array<f32>;
@group(0) @binding(3) var<storage, read_write> pressure_next: array<f32>;
@group(0) @binding(4) var<storage, read> movement_class_table: array<u32>;

const EMPTY: u32 = 0u;
const TABLE_LEN: u32 = 16u;
const CLASS_LIQUID: u32 = 2u;
const CLASS_GAS: u32 = 3u;
const PRESSURE_REFERENCE: f32 = 0.0;
const PRESSURE_DIFFUSION_RATE: f32 = 0.20;
const PRESSURE_MAX: f32 = 1.0e6;

fn sanitize_pressure(value: f32) -> f32 {
    if (value != value) {
        return PRESSURE_REFERENCE;
    }
    if (value > 1.0e20 || value < -1.0e20) {
        return PRESSURE_REFERENCE;
    }
    return clamp(value, PRESSURE_REFERENCE, PRESSURE_MAX);
}

fn in_domain(x: i32, y: i32) -> bool {
    return x >= 0 && y >= 0 && x < i32(params.width) && y < i32(params.height);
}

fn index_of(x: i32, y: i32) -> u32 {
    return u32(y) * params.width + u32(x);
}

fn is_pressure_medium(material: u32) -> bool {
    if (material == EMPTY || material >= TABLE_LEN) {
        return false;
    }
    let class = movement_class_table[material];
    return class == CLASS_LIQUID || class == CLASS_GAS;
}

fn accumulate(self_p: f32, nx: i32, ny: i32) -> f32 {
    if (!in_domain(nx, ny)) {
        return 0.0;
    }
    let nidx = index_of(nx, ny);
    if (!is_pressure_medium(material_current[nidx])) {
        return 0.0;
    }
    return sanitize_pressure(pressure_current[nidx]) - self_p;
}

@compute @workgroup_size(64, 1, 1)
fn pressure_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let index = gid.y * params.threads_x + gid.x;
    if (index >= params.cell_count) {
        return;
    }

    if (!is_pressure_medium(material_current[index])) {
        pressure_next[index] = PRESSURE_REFERENCE;
        return;
    }

    let self_p = sanitize_pressure(pressure_current[index]);
    let x = i32(index % params.width);
    let y = i32(index / params.width);

    var acc = 0.0;
    acc += accumulate(self_p, x, y - 1);
    acc += accumulate(self_p, x, y + 1);
    acc += accumulate(self_p, x - 1, y);
    acc += accumulate(self_p, x + 1, y);

    pressure_next[index] = sanitize_pressure(self_p + PRESSURE_DIFFUSION_RATE * acc);
}
'''

PRESSURE_GPU_TEST = r'''//! G5-A — scalar pressure field GPU semantic/invariant tests.
//!
//! These tests require the production Windows + RTX 5090 + DX12 path.
//! GitHub CI compiles them; the reference machine executes them for final
//! technical validation. G5-B expansion generation and G5-C rupture are out
//! of scope here.

use powdergame_core::{
    WorldConfig, MATERIAL_EMPTY, MATERIAL_STONE, MATERIAL_WATER, PRESSURE_REFERENCE,
};
use powdergame_gpu::Simulation;

fn make_sim(config: WorldConfig) -> Simulation {
    pollster::block_on(Simulation::new(config)).expect("DX12 + RTX 5090 simulation init")
}

fn eight_by_eight() -> Simulation {
    make_sim(WorldConfig::new(8, 8, 8).unwrap())
}

fn cell(sim: &Simulation, x: i64, y: i64) -> u32 {
    sim.world
        .read_material_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("material readback")
}

fn pressure(sim: &Simulation, x: i64, y: i64) -> f32 {
    sim.world
        .read_pressure_cell(&sim.context.device, &sim.context.queue, x, y)
        .expect("pressure readback")
}

fn all_pressure(sim: &Simulation) -> Vec<f32> {
    sim.world
        .read_pressure_all(&sim.context.device, &sim.context.queue)
        .expect("pressure readback")
}

fn set_mat(sim: &Simulation, x: i64, y: i64, material: u32) {
    sim.world
        .write_material(&sim.context.queue, x, y, material)
        .expect("material edit");
}

fn set_pressure(sim: &Simulation, x: i64, y: i64, value: f32) {
    sim.world
        .write_pressure(&sim.context.queue, x, y, value)
        .expect("pressure edit");
}

fn box_water_pair(sim: &Simulation) {
    // Two Water cells at (3,3)/(4,3), all of their liquid movement exits blocked.
    for (x, y) in [(2, 3), (5, 3), (2, 4), (3, 4), (4, 4), (5, 4)] {
        set_mat(sim, x, y, MATERIAL_STONE);
    }
    set_mat(sim, 3, 3, MATERIAL_WATER);
    set_mat(sim, 4, 3, MATERIAL_WATER);
}

#[test]
fn pressure_propagates_between_adjacent_liquid_cells() {
    let mut sim = eight_by_eight();
    box_water_pair(&sim);
    set_pressure(&sim, 3, 3, 100.0);
    set_pressure(&sim, 4, 3, 0.0);

    sim.tick().expect("tick");

    let left = pressure(&sim, 3, 3);
    let right = pressure(&sim, 4, 3);
    assert!(left < 100.0 && left > 0.0, "left={left}");
    assert!(right > 0.0 && right < left, "right={right}, left={left}");
    assert!(((left + right) - 100.0).abs() < 1.0e-3, "sum={}", left + right);
}

#[test]
fn isolated_pressure_has_no_time_decay() {
    let mut sim = eight_by_eight();
    set_mat(&sim, 3, 3, MATERIAL_WATER);
    for (x, y) in [(2, 3), (4, 3), (2, 4), (3, 4), (4, 4)] {
        set_mat(&sim, x, y, MATERIAL_STONE);
    }
    set_pressure(&sim, 3, 3, 42.0);

    for _ in 0..120 {
        sim.tick().expect("tick");
    }

    let p = pressure(&sim, 3, 3);
    assert!((p - 42.0).abs() < 1.0e-4, "pressure decayed without a sink: {p}");
}

#[test]
fn non_medium_cells_clear_pressure() {
    let mut sim = eight_by_eight();
    set_pressure(&sim, 3, 3, 50.0); // EMPTY
    set_mat(&sim, 4, 3, MATERIAL_STONE);
    set_pressure(&sim, 4, 3, 50.0);

    sim.tick().expect("tick");

    assert_eq!(pressure(&sim, 3, 3), PRESSURE_REFERENCE);
    assert_eq!(pressure(&sim, 4, 3), PRESSURE_REFERENCE);
}

#[test]
fn material_edit_clears_stale_spatial_pressure() {
    let sim = eight_by_eight();
    set_mat(&sim, 3, 3, MATERIAL_WATER);
    set_pressure(&sim, 3, 3, 25.0);
    assert_eq!(pressure(&sim, 3, 3), 25.0);

    set_mat(&sim, 3, 3, MATERIAL_STONE);
    assert_eq!(pressure(&sim, 3, 3), PRESSURE_REFERENCE);
}

#[test]
fn pressure_crosses_chunk_boundary() {
    let mut sim = make_sim(WorldConfig::new(128, 16, 64).unwrap());
    // Narrow two-cell liquid chamber across x=63/64.
    for (x, y) in [(62, 8), (65, 8), (62, 9), (63, 9), (64, 9), (65, 9)] {
        set_mat(&sim, x, y, MATERIAL_STONE);
    }
    set_mat(&sim, 63, 8, MATERIAL_WATER);
    set_mat(&sim, 64, 8, MATERIAL_WATER);
    set_pressure(&sim, 63, 8, 40.0);

    sim.tick().expect("tick");

    assert!(pressure(&sim, 63, 8) < 40.0);
    assert!(pressure(&sim, 64, 8) > 0.0);
}

#[test]
fn void_exit_vents_pressure_with_departing_medium() {
    let mut sim = eight_by_eight();
    // Replace the editable bottom boundary cell with Water. Its first down
    // movement target is Void, so the Matter exits before the pressure pass.
    set_mat(&sim, 4, 7, MATERIAL_WATER);
    set_pressure(&sim, 4, 7, 80.0);

    sim.tick().expect("tick");

    assert_eq!(cell(&sim, 4, 7), MATERIAL_EMPTY);
    assert_eq!(pressure(&sim, 4, 7), PRESSURE_REFERENCE);
}

#[test]
fn pressure_world_stays_finite_and_non_negative() {
    let mut sim = eight_by_eight();
    box_water_pair(&sim);
    set_pressure(&sim, 3, 3, 1.0e6);

    for _ in 0..200 {
        sim.tick().expect("tick");
    }

    for (i, p) in all_pressure(&sim).into_iter().enumerate() {
        assert!(p.is_finite(), "pressure[{i}] non-finite: {p}");
        assert!(p >= 0.0, "pressure[{i}] negative: {p}");
    }
}

#[test]
fn write_pressure_rejects_non_finite() {
    let sim = eight_by_eight();
    let err = sim
        .world
        .write_pressure(&sim.context.queue, 3, 3, f32::NAN)
        .expect_err("NaN must be rejected");
    assert!(format!("{err}").contains("invalid pressure"));
}
'''

write_new("engine/core/src/pressure.rs", PRESSURE_RS)
write_new("engine/gpu/src/pressure.wgsl", PRESSURE_WGSL)
write_new("engine/gpu/tests/pressure.rs", PRESSURE_GPU_TEST)

# Core public contract.
replace_once(
    "engine/core/src/lib.rs",
    "pub mod phase;\npub mod thermal;",
    "pub mod phase;\npub mod pressure;\npub mod thermal;",
)
replace_once(
    "engine/core/src/lib.rs",
    "    WATER_BOIL_ABOVE, WATER_FREEZE_BELOW,\n};\npub use thermal::{",
    "    WATER_BOIL_ABOVE, WATER_FREEZE_BELOW,\n};\npub use pressure::{\n"
    "    is_pressure_medium, pressure_step, sanitize_pressure, PressureNeighbor,\n"
    "    PRESSURE_DIFFUSION_RATE, PRESSURE_MAX, PRESSURE_REFERENCE,\n"
    "};\npub use thermal::{",
)

# GPU error contract for validated edit hook.
replace_once(
    "engine/gpu/src/context.rs",
    "    /// A temperature edit is not a finite f32.\n    InvalidTemperature(f32),\n    /// Other error with a message.",
    "    /// A temperature edit is not a finite f32.\n    InvalidTemperature(f32),\n"
    "    /// A pressure edit is not a finite f32.\n    InvalidPressure(f32),\n    /// Other error with a message.",
)
replace_once(
    "engine/gpu/src/context.rs",
    "            GpuError::InvalidTemperature(value) => {\n"
    "                write!(f, \"invalid temperature {value}: must be a finite f32\")\n"
    "            }\n            GpuError::InvalidMaterialValue(value) => {",
    "            GpuError::InvalidTemperature(value) => {\n"
    "                write!(f, \"invalid temperature {value}: must be a finite f32\")\n"
    "            }\n"
    "            GpuError::InvalidPressure(value) => {\n"
    "                write!(f, \"invalid pressure {value}: must be a finite f32\")\n"
    "            }\n            GpuError::InvalidMaterialValue(value) => {",
)

# GPU world read/write hooks and stale-field cleanup on authoring edits.
replace_once(
    "engine/gpu/src/world.rs",
    "    FLAGS_ELEM_SIZE, MATERIAL_ELEM_SIZE, MATERIAL_EMPTY, TEMPERATURE_ELEM_SIZE,\n"
    "    TEMPERATURE_REFERENCE,\n};",
    "    FLAGS_ELEM_SIZE, MATERIAL_ELEM_SIZE, MATERIAL_EMPTY, PRESSURE_ELEM_SIZE,\n"
    "    PRESSURE_REFERENCE, TEMPERATURE_ELEM_SIZE, TEMPERATURE_REFERENCE,\n};",
)
replace_once(
    "engine/gpu/src/world.rs",
    "        queue.write_buffer(&self.flags_current, f_off, &zero_flags);\n"
    "        queue.write_buffer(&self.flags_next, f_off, &zero_flags);\n"
    "        if value == MATERIAL_EMPTY {",
    "        queue.write_buffer(&self.flags_current, f_off, &zero_flags);\n"
    "        queue.write_buffer(&self.flags_next, f_off, &zero_flags);\n"
    "        // Pressure is spatial, not Matter-owned. An explicit authoring\n"
    "        // identity replacement must never inherit stale field state.\n"
    "        let zero_pressure = PRESSURE_REFERENCE.to_ne_bytes();\n"
    "        let p_off = index * PRESSURE_ELEM_SIZE;\n"
    "        queue.write_buffer(&self.pressure_current, p_off, &zero_pressure);\n"
    "        queue.write_buffer(&self.pressure_next, p_off, &zero_pressure);\n"
    "        if value == MATERIAL_EMPTY {",
)
replace_once(
    "engine/gpu/src/world.rs",
    "        queue.write_buffer(&self.temperature_current, offset, &bytes);\n"
    "        queue.write_buffer(&self.temperature_next, offset, &bytes);\n"
    "        Ok(())\n    }\n}\n\n/// Copies `size` bytes out of `source` at `offset` and maps them back to CPU.",
    "        queue.write_buffer(&self.temperature_current, offset, &bytes);\n"
    "        queue.write_buffer(&self.temperature_next, offset, &bytes);\n"
    "        Ok(())\n    }\n\n"
    "    /// Reads one cell's scalar pressure (diagnostic/test helper).\n"
    "    pub fn read_pressure_cell(\n"
    "        &self,\n        device: &wgpu::Device,\n        queue: &wgpu::Queue,\n"
    "        x: i64,\n        y: i64,\n    ) -> Result<f32, GpuError> {\n"
    "        let index = self\n            .domain\n            .index(x, y)\n"
    "            .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;\n"
    "        let offset = index * PRESSURE_ELEM_SIZE;\n"
    "        let bytes = read_back_bytes(\n            device,\n            queue,\n"
    "            &self.pressure_current,\n            offset,\n            PRESSURE_ELEM_SIZE,\n        )?;\n"
    "        Ok(f32::from_ne_bytes(bytes[..4].try_into().unwrap()))\n    }\n\n"
    "    /// Reads the entire scalar pressure Current buffer (test helper).\n"
    "    pub fn read_pressure_all(\n        &self,\n        device: &wgpu::Device,\n"
    "        queue: &wgpu::Queue,\n    ) -> Result<Vec<f32>, GpuError> {\n"
    "        let bytes = read_back_bytes(\n            device,\n            queue,\n"
    "            &self.pressure_current,\n            0,\n            self.layout.pressure_bytes,\n        )?;\n"
    "        let mut cells = Vec::with_capacity(bytes.len() / 4);\n"
    "        for chunk in bytes.chunks_exact(4) {\n"
    "            cells.push(f32::from_ne_bytes(chunk.try_into().unwrap()));\n        }\n"
    "        Ok(cells)\n    }\n\n"
    "    /// Edit/test hook: sets scalar pressure on Current and Next.\n"
    "    /// Non-finite values are rejected; the simulation pass later clears\n"
    "    /// pressure from cells that are not Liquid/Gas pressure media.\n"
    "    pub fn write_pressure(\n        &self,\n        queue: &wgpu::Queue,\n"
    "        x: i64,\n        y: i64,\n        value: f32,\n    ) -> Result<(), GpuError> {\n"
    "        if !value.is_finite() {\n            return Err(GpuError::InvalidPressure(value));\n        }\n"
    "        let index = self\n            .domain\n            .index(x, y)\n"
    "            .ok_or(GpuError::CoordinateOutOfBounds { x, y })?;\n"
    "        let offset = index * PRESSURE_ELEM_SIZE;\n        let bytes = value.to_ne_bytes();\n"
    "        queue.write_buffer(&self.pressure_current, offset, &bytes);\n"
    "        queue.write_buffer(&self.pressure_next, offset, &bytes);\n        Ok(())\n    }\n}\n\n"
    "/// Copies `size` bytes out of `source` at `offset` and maps them back to CPU.",
)

# Simulation pipeline integration.
replace_once(
    "engine/gpu/src/simulation.rs",
    "//! winner per destination. Pressure is a spatial field (G5) and is never\n"
    "//! transported on movement edges.\n//!\n"
    "//! Causal order per tick: movement (Matter carries Temperature + flags) →\n"
    "//! thermal conduction → phase transition → combustion → smoke spawn.\n"
    "//! Expansion / Pressure are not implemented (G5).",
    "//! winner per destination. Pressure is a spatial field (G5) and is never\n"
    "//! transported on movement edges.\n//!\n"
    "//! G5-A adds scalar pressure propagation after Matter/phase/combustion settle:\n"
    "//! Liquid/Gas cells exchange pressure with 4-neighbor Liquid/Gas cells via\n"
    "//! Read Neighbors / Write Self. EMPTY/Static/Powder do not transmit it.\n//!\n"
    "//! Causal order per tick: movement (Matter carries Temperature + flags) →\n"
    "//! thermal conduction → phase transition → combustion → smoke spawn → pressure.\n"
    "//! Blocked expansion generation and rupture remain G5-B/G5-C.",
)
replace_once(
    "engine/gpu/src/simulation.rs",
    "    smoke_claim_pipeline: wgpu::ComputePipeline,\n    smoke_commit_pipeline: wgpu::ComputePipeline,",
    "    smoke_claim_pipeline: wgpu::ComputePipeline,\n    smoke_commit_pipeline: wgpu::ComputePipeline,\n"
    "    pressure_pipeline: wgpu::ComputePipeline,",
)
replace_once(
    "engine/gpu/src/simulation.rs",
    "    smoke_claim_bind_group: wgpu::BindGroup,\n    smoke_commit_bind_group: wgpu::BindGroup,\n    marker: wgpu::Buffer,",
    "    smoke_claim_bind_group: wgpu::BindGroup,\n    smoke_commit_bind_group: wgpu::BindGroup,\n"
    "    pressure_bind_group: wgpu::BindGroup,\n    marker: wgpu::Buffer,",
)
replace_once(
    "engine/gpu/src/simulation.rs",
    "        // Bind group layouts.\n",
    "        let shader_pressure = context\n"
    "            .device\n            .create_shader_module(wgpu::ShaderModuleDescriptor {\n"
    "                label: Some(\"powdergame-g5a-pressure\"),\n"
    "                source: wgpu::ShaderSource::Wgsl(include_str!(\"pressure.wgsl\").into()),\n"
    "            });\n\n        // Bind group layouts.\n",
)
replace_once(
    "engine/gpu/src/simulation.rs",
    "        let make_pipeline = |label: &str,",
    "        let pressure_layout =\n            context\n                .device\n"
    "                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {\n"
    "                    label: Some(\"powdergame-g5a-pressure-bgl\"),\n"
    "                    entries: &[\n"
    "                        buffer_entry(0, &BindingKind::Uniform),\n"
    "                        buffer_entry(1, &BindingKind::Read), // material_current\n"
    "                        buffer_entry(2, &BindingKind::Read), // pressure_current\n"
    "                        buffer_entry(3, &BindingKind::ReadWrite), // pressure_next\n"
    "                        buffer_entry(4, &BindingKind::Read), // movement_class_table\n"
    "                    ],\n                });\n\n        let make_pipeline = |label: &str,",
)
replace_once(
    "engine/gpu/src/simulation.rs",
    "        // Params uniform: cell_count, threads_x, width, height.\n",
    "        let pressure_pipeline = make_pipeline(\n"
    "            \"powdergame-g5a-pressure\",\n            &pressure_layout,\n"
    "            &shader_pressure,\n            \"pressure_main\",\n        );\n\n"
    "        // Params uniform: cell_count, threads_x, width, height.\n",
)
replace_once(
    "engine/gpu/src/simulation.rs",
    "\n        Ok(Self {\n            context,",
    "\n        let pressure_bind_group = context\n            .device\n"
    "            .create_bind_group(&wgpu::BindGroupDescriptor {\n"
    "                label: Some(\"powdergame-g5a-pressure-bg\"),\n"
    "                layout: &pressure_layout,\n                entries: &[\n"
    "                    wgpu::BindGroupEntry { binding: 0, resource: params.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 1, resource: world.material_current.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 2, resource: world.pressure_current.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 3, resource: world.pressure_next.as_entire_binding() },\n"
    "                    wgpu::BindGroupEntry { binding: 4, resource: class_table.as_entire_binding() },\n"
    "                ],\n            });\n\n        Ok(Self {\n            context,",
)
replace_once(
    "engine/gpu/src/simulation.rs",
    "            smoke_claim_pipeline,\n            smoke_commit_pipeline,\n            propose_bind_group,",
    "            smoke_claim_pipeline,\n            smoke_commit_pipeline,\n            pressure_pipeline,\n"
    "            propose_bind_group,",
)
replace_once(
    "engine/gpu/src/simulation.rs",
    "            smoke_claim_bind_group,\n            smoke_commit_bind_group,\n\n            marker,",
    "            smoke_claim_bind_group,\n            smoke_commit_bind_group,\n"
    "            pressure_bind_group,\n\n            marker,",
)
replace_once(
    "engine/gpu/src/simulation.rs",
    "    /// → copy material/temperature/flags Next→Current\n    /// ```",
    "    /// → copy material/temperature/flags Next→Current\n"
    "    /// → scalar pressure 4-neighbor propagation → copy pressure Next→Current\n    /// ```",
)
replace_once(
    "engine/gpu/src/simulation.rs",
    "    /// runs, then Smoke spawns with ownership. Expansion / Pressure are\n"
    "    /// not implemented (G5).",
    "    /// runs, then Smoke spawns with ownership. G5-A pressure propagation\n"
    "    /// runs last on settled Matter. Expansion generation / rupture remain\n"
    "    /// G5-B/G5-C.",
)
replace_once(
    "engine/gpu/src/simulation.rs",
    "        self.context.queue.submit([encoder.finish()]);\n        self.tick_count += 1;",
    "        // G5-A: spatial scalar pressure. It is deliberately not carried on\n"
    "        // movement ownership edges; the settled Matter map decides where the\n"
    "        // field can exist and which 4-neighbor cells exchange it.\n"
    "        {\n            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {\n"
    "                label: Some(\"powdergame-g5a-pressure-pass\"),\n"
    "                timestamp_writes: None,\n            });\n"
    "            dispatch(&mut pass, &self.pressure_pipeline, &self.pressure_bind_group);\n        }\n"
    "        encoder.copy_buffer_to_buffer(\n            &self.world.pressure_next,\n            0,\n"
    "            &self.world.pressure_current,\n            0,\n            self.world.layout.pressure_bytes,\n        );\n\n"
    "        self.context.queue.submit([encoder.finish()]);\n        self.tick_count += 1;",
)

print("G5-A pressure implementation applied")
