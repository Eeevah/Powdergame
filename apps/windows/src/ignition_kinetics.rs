//! TE-4I user-testable ignition-kinetics candidate.

use powdergame_core::{
    combustion_descriptor, fuel_progress, ignition_context, ignition_exposure, AirState,
    EmptyEnvironmentSeed, FLAG_COMBUSTING, FLAG_FLAME_EVENT, MATERIAL_BOUNDARY_BLOCK,
    MATERIAL_EMPTY, MATERIAL_OIL, MATERIAL_STONE, MATERIAL_WOOD, TEMPERATURE_REFERENCE,
};
use powdergame_gpu::{GpuError, Simulation};

pub(crate) const TE4_TITLE: &str = "Powdergame TE-4 Ignition Kinetics";
pub(crate) const TE4_WORLD_WIDTH: u32 = 256;
pub(crate) const TE4_WORLD_HEIGHT: u32 = 192;
pub(crate) const TE4_CHUNK_SIZE: u32 = 64;
pub(crate) const TE4_TPS: u32 = 60;
const SAMPLE_INTERVAL: u64 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IgnitionKineticsScene {
    SpikeVsSustained,
    FlameVsInert,
    SurfaceFront,
    AirVacuumSmoke,
}

impl IgnitionKineticsScene {
    pub(crate) fn from_number(number: u8) -> Option<Self> {
        Some(match number {
            1 => Self::SpikeVsSustained,
            2 => Self::FlameVsInert,
            3 => Self::SurfaceFront,
            4 => Self::AirVacuumSmoke,
            _ => return None,
        })
    }
    pub(crate) fn number(self) -> u8 {
        match self {
            Self::SpikeVsSustained => 1,
            Self::FlameVsInert => 2,
            Self::SurfaceFront => 3,
            Self::AirVacuumSmoke => 4,
        }
    }
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::SpikeVsSustained => "Spike versus sustained heat",
            Self::FlameVsInert => "Flame bonus versus inert heat",
            Self::SurfaceFront => "Surface-first connected fuel",
            Self::AirVacuumSmoke => "Air / Vacuum / self-Smoke",
        }
    }
    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::SpikeVsSustained => "Short spikes decay; sustained Oil reaches dose before Wood.",
            Self::FlameVsInert => {
                "Previous-tick flame accelerates but never bypasses own threshold."
            }
            Self::SurfaceFront => {
                "A real heated surface advances into a connected two-dimensional fuel bed."
            }
            Self::AirVacuumSmoke => {
                "Atmosphere/LowPressure qualify; Vacuum does not; self-Smoke removes sole Air."
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IgnitionSampleRow {
    pub label: &'static str,
    pub cell: (u32, u32),
    pub material: u32,
    pub temperature: f32,
    pub exposure: u32,
    pub thermal_rate: u32,
    pub previous_flames: u32,
    pub air_access: bool,
    pub burning: bool,
    pub fuel_progress: u32,
    pub fuel_duration: u32,
    pub gross_q_this_tick: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IgnitionSample {
    pub generation: u64,
    pub sequence: u64,
    pub sample_tick: u64,
    pub rows: Vec<IgnitionSampleRow>,
    pub oil_count: usize,
    pub wood_count: usize,
    pub burning_count: usize,
    pub newly_ignited_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum IgnitionDiagnosticState {
    Sampling {
        generation: u64,
        sequence: u64,
        simulation_tick: u64,
    },
    Fresh(IgnitionSample),
    Failed {
        generation: u64,
        sequence: u64,
        simulation_tick: u64,
        message: String,
    },
}

#[derive(Clone, Copy)]
struct SampleRequest {
    generation: u64,
    sequence: u64,
    simulation_tick: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct IgnitionHudData {
    pub scene: IgnitionKineticsScene,
    pub playing: bool,
    pub fast: u32,
    pub simulation_tick: u64,
    pub diagnostic: IgnitionDiagnosticState,
    pub details_visible: bool,
}

pub(crate) struct IgnitionKineticsState {
    scene: IgnitionKineticsScene,
    diagnostic: IgnitionDiagnosticState,
    generation: u64,
    next_sequence: u64,
    details_visible: bool,
}

impl IgnitionKineticsState {
    pub(crate) fn new(simulation: &mut Simulation) -> Result<Self, GpuError> {
        let mut state = Self {
            scene: IgnitionKineticsScene::SpikeVsSustained,
            diagnostic: IgnitionDiagnosticState::Sampling {
                generation: 0,
                sequence: 0,
                simulation_tick: 0,
            },
            generation: 0,
            next_sequence: 1,
            details_visible: true,
        };
        state.reset(simulation)?;
        Ok(state)
    }
    pub(crate) fn select_scene(
        &mut self,
        simulation: &mut Simulation,
        scene: IgnitionKineticsScene,
    ) -> Result<(), GpuError> {
        self.scene = scene;
        self.reset(simulation)
    }
    pub(crate) fn reset(&mut self, simulation: &mut Simulation) -> Result<(), GpuError> {
        self.generation = self.generation.wrapping_add(1).max(1);
        self.diagnostic = IgnitionDiagnosticState::Sampling {
            generation: self.generation,
            sequence: 0,
            simulation_tick: simulation.tick_count,
        };
        stage_scene(simulation, self.scene)?;
        self.next_sequence = 1;
        self.sample_now(simulation);
        Ok(())
    }
    pub(crate) fn tick(
        &mut self,
        simulation: &mut Simulation,
        force_sample: bool,
    ) -> Result<(), GpuError> {
        simulation.tick()?;
        if force_sample || simulation.tick_count.is_multiple_of(SAMPLE_INTERVAL) {
            self.sample_now(simulation);
        }
        Ok(())
    }
    pub(crate) fn toggle_details(&mut self) -> bool {
        self.details_visible = !self.details_visible;
        self.details_visible
    }
    pub(crate) fn hud_data(
        &self,
        playing: bool,
        fast: u32,
        simulation_tick: u64,
    ) -> IgnitionHudData {
        IgnitionHudData {
            scene: self.scene,
            playing,
            fast,
            simulation_tick,
            diagnostic: self.diagnostic.clone(),
            details_visible: self.details_visible,
        }
    }
    pub(crate) fn measurement_summary(&self) -> String {
        match &self.diagnostic {
            IgnitionDiagnosticState::Fresh(sample) => format!(
                "sample_tick={} oil={} wood={} burning={} new={} rows={}",
                sample.sample_tick,
                sample.oil_count,
                sample.wood_count,
                sample.burning_count,
                sample.newly_ignited_count,
                sample.rows.len()
            ),
            _ => "sample=unavailable".to_string(),
        }
    }
    fn sample_now(&mut self, simulation: &Simulation) {
        let request = SampleRequest {
            generation: self.generation,
            sequence: self.next_sequence,
            simulation_tick: simulation.tick_count,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.diagnostic = IgnitionDiagnosticState::Sampling {
            generation: request.generation,
            sequence: request.sequence,
            simulation_tick: request.simulation_tick,
        };
        match collect_sample(simulation, self.scene, request) {
            Ok(sample) => {
                self.diagnostic = IgnitionDiagnosticState::Fresh(sample);
            }
            Err(error) => {
                self.diagnostic = IgnitionDiagnosticState::Failed {
                    generation: request.generation,
                    sequence: request.sequence,
                    simulation_tick: request.simulation_tick,
                    message: error.to_string(),
                }
            }
        }
    }
}

fn sample_cells(scene: IgnitionKineticsScene) -> [(&'static str, (u32, u32)); 4] {
    match scene {
        IgnitionKineticsScene::SpikeVsSustained => [
            ("Oil short spike", (52, 110)),
            ("Wood short spike", (92, 110)),
            ("Oil sustained", (152, 110)),
            ("Wood sustained", (204, 110)),
        ],
        IgnitionKineticsScene::FlameVsInert => [
            ("Previous-flame target", (86, 110)),
            ("Inert-heat target", (174, 110)),
            ("Flame source", (85, 110)),
            ("Inert Stone", (173, 110)),
        ],
        IgnitionKineticsScene::SurfaceFront => [
            ("Heated frontier", (72, 104)),
            ("Near interior", (88, 104)),
            ("Deep interior", (108, 104)),
            ("Oil frontier", (144, 116)),
        ],
        IgnitionKineticsScene::AirVacuumSmoke => [
            ("Atmosphere", (52, 110)),
            ("LowPressure", (104, 110)),
            ("Exact Vacuum", (156, 110)),
            ("Sole-Air self-Smoke", (208, 110)),
        ],
    }
}

fn collect_sample(
    simulation: &Simulation,
    scene: IgnitionKineticsScene,
    request: SampleRequest,
) -> Result<IgnitionSample, GpuError> {
    let materials = simulation
        .world
        .read_material_all(&simulation.context.device, &simulation.context.queue)?;
    let flags_all = simulation
        .world
        .read_flags_all(&simulation.context.device, &simulation.context.queue)?;
    let mut rows = Vec::with_capacity(4);
    for (label, cell) in sample_cells(scene) {
        let index = (cell.1 * TE4_WORLD_WIDTH + cell.0) as usize;
        let material = materials[index];
        let flags = flags_all[index];
        let temperature = simulation.world.read_temperature_cell(
            &simulation.context.device,
            &simulation.context.queue,
            i64::from(cell.0),
            i64::from(cell.1),
        )?;
        let (air_access, previous_flames) =
            context_faces(simulation, cell, &materials, &flags_all)?;
        let context = ignition_context(material, temperature, flags, air_access, previous_flames);
        let descriptor = combustion_descriptor(material);
        rows.push(IgnitionSampleRow {
            label,
            cell,
            material,
            temperature,
            exposure: ignition_exposure(flags),
            thermal_rate: context.thermal_rate,
            previous_flames,
            air_access,
            burning: flags & FLAG_COMBUSTING != 0,
            fuel_progress: fuel_progress(flags),
            fuel_duration: descriptor.map_or(0, |d| d.burn_duration_ticks),
            gross_q_this_tick: if flags & FLAG_FLAME_EVENT != 0 {
                descriptor.map_or(0.0, |d| d.chemical_q_per_tick)
            } else {
                0.0
            },
        });
    }
    let oil_count = materials.iter().filter(|&&m| m == MATERIAL_OIL).count();
    let wood_count = materials.iter().filter(|&&m| m == MATERIAL_WOOD).count();
    let burning_count = flags_all
        .iter()
        .filter(|&&f| f & FLAG_COMBUSTING != 0)
        .count();
    let newly_ignited_count = flags_all
        .iter()
        .filter(|&&f| f & FLAG_FLAME_EVENT != 0 && fuel_progress(f) == 1)
        .count();
    Ok(IgnitionSample {
        generation: request.generation,
        sequence: request.sequence,
        sample_tick: request.simulation_tick,
        rows,
        oil_count,
        wood_count,
        burning_count,
        newly_ignited_count,
    })
}

fn context_faces(
    simulation: &Simulation,
    cell: (u32, u32),
    materials: &[u32],
    flags: &[u32],
) -> Result<(bool, u32), GpuError> {
    let neighbors = [
        (cell.0 - 1, cell.1),
        (cell.0 + 1, cell.1),
        (cell.0, cell.1 - 1),
        (cell.0, cell.1 + 1),
    ];
    let air = simulation.world.read_environment_cells(
        &simulation.context.device,
        &simulation.context.queue,
        &neighbors
            .iter()
            .map(|&(x, y)| (i64::from(x), i64::from(y)))
            .collect::<Vec<_>>(),
    )?;
    let mut access = false;
    let mut flames = 0;
    for (n, snapshot) in neighbors.into_iter().zip(air) {
        let i = (n.1 * TE4_WORLD_WIDTH + n.0) as usize;
        access |= materials[i] == MATERIAL_EMPTY && snapshot.current.mass > 0.0;
        flames += u32::from(flags[i] & FLAG_FLAME_EVENT != 0);
    }
    Ok((access, flames))
}

#[allow(clippy::too_many_arguments)]
fn put_cell(
    material: &mut [u32],
    temperature: &mut [f32],
    flags: &mut [u32],
    x: u32,
    y: u32,
    value: u32,
    degrees_c: f32,
    flag_bits: u32,
) {
    let index = (y * TE4_WORLD_WIDTH + x) as usize;
    material[index] = value;
    temperature[index] = degrees_c;
    flags[index] = flag_bits;
}

fn put_cage(
    material: &mut [u32],
    temperature: &mut [f32],
    flags: &mut [u32],
    x: u32,
    y: u32,
    degrees_c: f32,
) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx != 0 || dy != 0 {
                put_cell(
                    material,
                    temperature,
                    flags,
                    (x as i32 + dx) as u32,
                    (y as i32 + dy) as u32,
                    MATERIAL_STONE,
                    degrees_c,
                    0,
                );
            }
        }
    }
    put_cell(
        material,
        temperature,
        flags,
        x,
        y - 1,
        MATERIAL_EMPTY,
        20.0,
        0,
    );
    put_cell(
        material,
        temperature,
        flags,
        x,
        y - 2,
        MATERIAL_STONE,
        degrees_c,
        0,
    );
}

fn stage_scene(simulation: &mut Simulation, scene: IgnitionKineticsScene) -> Result<(), GpuError> {
    let count = (TE4_WORLD_WIDTH * TE4_WORLD_HEIGHT) as usize;
    let mut material = vec![MATERIAL_EMPTY; count];
    let mut temperature = vec![TEMPERATURE_REFERENCE; count];
    let mut flags = vec![0u32; count];
    macro_rules! put {
        ($x:expr, $y:expr, $m:expr, $t:expr, $f:expr) => {
            put_cell(
                &mut material,
                &mut temperature,
                &mut flags,
                $x,
                $y,
                $m,
                $t,
                $f,
            )
        };
    }
    macro_rules! cage {
        ($x:expr, $y:expr, $t:expr) => {
            put_cage(&mut material, &mut temperature, &mut flags, $x, $y, $t)
        };
    }
    for x in 0..TE4_WORLD_WIDTH {
        put!(x, 0, MATERIAL_BOUNDARY_BLOCK, 20.0, 0);
        put!(x, TE4_WORLD_HEIGHT - 1, MATERIAL_BOUNDARY_BLOCK, 20.0, 0);
    }
    for y in 0..TE4_WORLD_HEIGHT {
        put!(0, y, MATERIAL_BOUNDARY_BLOCK, 20.0, 0);
        put!(TE4_WORLD_WIDTH - 1, y, MATERIAL_BOUNDARY_BLOCK, 20.0, 0);
    }
    match scene {
        IgnitionKineticsScene::SpikeVsSustained => {
            cage!(52, 110, 20.0);
            put!(
                52,
                110,
                MATERIAL_OIL,
                100.0,
                powdergame_core::with_ignition_exposure(0, 8)
            );
            cage!(92, 110, 20.0);
            put!(
                92,
                110,
                MATERIAL_WOOD,
                200.0,
                powdergame_core::with_ignition_exposure(0, 8)
            );
            cage!(152, 110, 349.0);
            put!(152, 110, MATERIAL_OIL, 349.0, 0);
            cage!(204, 110, 449.0);
            put!(204, 110, MATERIAL_WOOD, 449.0, 0);
        }
        IgnitionKineticsScene::FlameVsInert => {
            // Matched finite 400 C Stone reservoirs and one positive-Air face
            // per target. The only treatment difference is the previous-tick
            // FLAME_EVENT on the left neighbour of the first target.
            for p in [
                (84, 110),
                (85, 111),
                (86, 111),
                (87, 110),
                (173, 111),
                (174, 111),
                (175, 110),
            ] {
                put!(p.0, p.1, MATERIAL_STONE, 400.0, 0);
            }
            put!(
                85,
                110,
                MATERIAL_WOOD,
                400.0,
                FLAG_COMBUSTING | FLAG_FLAME_EVENT
            );
            put!(86, 110, MATERIAL_WOOD, 400.0, 0);
            put!(173, 110, MATERIAL_STONE, 400.0, 0);
            put!(174, 110, MATERIAL_WOOD, 400.0, 0);
        }
        IgnitionKineticsScene::SurfaceFront => {
            for y in 96..=112 {
                for x in 72..=116 {
                    put!(x, y, MATERIAL_WOOD, 290.0, 0);
                }
            }
            for y in 96..=112 {
                for x in 66..=71 {
                    put!(x, y, MATERIAL_STONE, 1200.0, 0);
                }
            }
            for y in 114..=120 {
                for x in 144..=170 {
                    put!(x, y, MATERIAL_OIL, 249.0, 0);
                }
            }
            for y in 114..=120 {
                put!(143, y, MATERIAL_STONE, 1200.0, 0);
            }
        }
        IgnitionKineticsScene::AirVacuumSmoke => {
            for x in [52, 104, 156, 208] {
                cage!(x, 110, 400.0);
                put!(x, 110, MATERIAL_WOOD, 400.0, 0);
            }
            put!(208, 110, MATERIAL_WOOD, 500.0, FLAG_COMBUSTING);
            // Sole-Air target is lateral; receiver below it is not a GAS movement route.
            for p in [
                (208, 109),
                (207, 109),
                (209, 109),
                (210, 109),
                (207, 110),
                (208, 111),
                (210, 110),
            ] {
                put!(p.0, p.1, MATERIAL_STONE, 20.0, 0);
            }
            put!(209, 110, MATERIAL_EMPTY, 20.0, 0);
            put!(209, 111, MATERIAL_EMPTY, 20.0, 0);
            put!(210, 111, MATERIAL_STONE, 20.0, 0);
            put!(209, 112, MATERIAL_STONE, 20.0, 0);
        }
    }
    simulation.reset()?;
    let q = &simulation.context.queue;
    q.write_buffer(
        &simulation.world.material_current,
        0,
        bytemuck::cast_slice(&material),
    );
    q.write_buffer(
        &simulation.world.material_next,
        0,
        bytemuck::cast_slice(&material),
    );
    q.write_buffer(
        &simulation.world.temperature_current,
        0,
        bytemuck::cast_slice(&temperature),
    );
    q.write_buffer(
        &simulation.world.temperature_next,
        0,
        bytemuck::cast_slice(&temperature),
    );
    q.write_buffer(
        &simulation.world.flags_current,
        0,
        bytemuck::cast_slice(&flags),
    );
    q.write_buffer(
        &simulation.world.flags_next,
        0,
        bytemuck::cast_slice(&flags),
    );
    simulation.world.stage_environment_for_materials(
        q,
        &material,
        EmptyEnvironmentSeed::StandardAtmosphere,
    )?;
    if scene == IgnitionKineticsScene::AirVacuumSmoke {
        write_air(
            simulation,
            (104, 109),
            AirState {
                mass: 0.1,
                energy: 29.315,
            },
        );
        write_air(
            simulation,
            (156, 109),
            AirState {
                mass: 0.0,
                energy: 0.0,
            },
        );
        write_air(
            simulation,
            (209, 110),
            AirState {
                mass: 1.0,
                energy: 293.15,
            },
        );
        write_air(
            simulation,
            (209, 111),
            AirState {
                mass: 0.0,
                energy: 0.0,
            },
        );
    }
    q.submit([]);
    simulation
        .context
        .device
        .poll(wgpu::PollType::Wait)
        .map_err(|e| GpuError::Other(format!("TE-4 staging wait failed: {e}")))?;
    Ok(())
}

fn write_air(simulation: &Simulation, cell: (u32, u32), state: AirState) {
    let offset = u64::from(cell.1 * TE4_WORLD_WIDTH + cell.0) * 4;
    for b in [
        &simulation.world.air_mass_current,
        &simulation.world.air_mass_next,
    ] {
        simulation
            .context
            .queue
            .write_buffer(b, offset, &state.mass.to_ne_bytes());
    }
    for b in [
        &simulation.world.air_energy_current,
        &simulation.world.air_energy_next,
    ] {
        simulation
            .context
            .queue
            .write_buffer(b, offset, &state.energy.to_ne_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sim() -> Simulation {
        pollster::block_on(Simulation::new(
            powdergame_core::WorldConfig::new(TE4_WORLD_WIDTH, TE4_WORLD_HEIGHT, TE4_CHUNK_SIZE)
                .unwrap(),
        ))
        .unwrap()
    }
    #[test]
    fn all_four_scenes_stage_reset_and_sample_real_state() {
        let mut s = sim();
        let mut state = IgnitionKineticsState::new(&mut s).unwrap();
        for n in 1..=4 {
            state
                .select_scene(&mut s, IgnitionKineticsScene::from_number(n).unwrap())
                .unwrap();
            state.tick(&mut s, true).unwrap();
            match &state.diagnostic {
                IgnitionDiagnosticState::Fresh(sample) => assert_eq!(sample.rows.len(), 4),
                other => panic!("unexpected {other:?}"),
            };
        }
    }
    #[test]
    fn scene_one_exposure_decays_and_sustained_controls_progress() {
        let mut s = sim();
        let mut state = IgnitionKineticsState::new(&mut s).unwrap();
        let before = match &state.diagnostic {
            IgnitionDiagnosticState::Fresh(v) => v.rows.clone(),
            _ => unreachable!(),
        };
        for _ in 0..8 {
            state.tick(&mut s, true).unwrap();
        }
        let after = match &state.diagnostic {
            IgnitionDiagnosticState::Fresh(v) => &v.rows,
            _ => unreachable!(),
        };
        assert!(after[0].exposure < before[0].exposure);
        assert!(after[2].exposure > before[2].exposure);
    }
    #[test]
    fn scene_four_exposes_binary_air_policy() {
        let mut s = sim();
        let mut state = IgnitionKineticsState::new(&mut s).unwrap();
        state
            .select_scene(&mut s, IgnitionKineticsScene::AirVacuumSmoke)
            .unwrap();
        let rows = match &state.diagnostic {
            IgnitionDiagnosticState::Fresh(v) => &v.rows,
            _ => unreachable!(),
        };
        assert!(rows[0].air_access && rows[1].air_access);
        assert!(!rows[2].air_access);
    }
    #[test]
    fn scene_four_self_smoke_emits_once_then_extinguishes_on_next_tick() {
        let mut s = sim();
        let mut state = IgnitionKineticsState::new(&mut s).unwrap();
        state
            .select_scene(&mut s, IgnitionKineticsScene::AirVacuumSmoke)
            .unwrap();
        state.tick(&mut s, true).unwrap();
        let after_n = match &state.diagnostic {
            IgnitionDiagnosticState::Fresh(v) => &v.rows[3],
            _ => unreachable!(),
        };
        assert!(after_n.burning);
        assert_eq!(after_n.fuel_progress, 1);
        assert!(
            !after_n.air_access,
            "settled Smoke occupies the sole Air face"
        );
        state.tick(&mut s, true).unwrap();
        let after_n1 = match &state.diagnostic {
            IgnitionDiagnosticState::Fresh(v) => &v.rows[3],
            _ => unreachable!(),
        };
        assert!(!after_n1.burning);
        assert_eq!(
            after_n1.fuel_progress, 1,
            "next no-Air stage consumes no fuel"
        );
        assert_eq!(after_n1.gross_q_this_tick, 0.0);
    }
    #[test]
    fn candidate_reset_restores_exact_authoritative_material_temperature_and_flags() {
        let mut s = sim();
        let mut state = IgnitionKineticsState::new(&mut s).unwrap();
        state
            .select_scene(&mut s, IgnitionKineticsScene::SurfaceFront)
            .unwrap();
        let initial_material = s
            .world
            .read_material_all(&s.context.device, &s.context.queue)
            .unwrap();
        let initial_temperature = s
            .world
            .read_temperature_all(&s.context.device, &s.context.queue)
            .unwrap();
        let initial_flags = s
            .world
            .read_flags_all(&s.context.device, &s.context.queue)
            .unwrap();
        for _ in 0..24 {
            state.tick(&mut s, false).unwrap();
        }
        state.reset(&mut s).unwrap();
        assert_eq!(
            s.world
                .read_material_all(&s.context.device, &s.context.queue)
                .unwrap(),
            initial_material
        );
        assert_eq!(
            s.world
                .read_temperature_all(&s.context.device, &s.context.queue)
                .unwrap(),
            initial_temperature
        );
        assert_eq!(
            s.world
                .read_flags_all(&s.context.device, &s.context.queue)
                .unwrap(),
            initial_flags
        );
        let fresh = match &state.diagnostic {
            IgnitionDiagnosticState::Fresh(v) => v,
            _ => unreachable!(),
        };
        assert_eq!(fresh.generation, state.generation);
        assert_eq!(fresh.sample_tick, 0);
    }
    #[test]
    fn scene_one_finite_sources_reach_locked_outputs_within_horizon() {
        let mut s = sim();
        let mut state = IgnitionKineticsState::new(&mut s).unwrap();
        let mut oil_tick = None;
        let mut wood_tick = None;
        for tick in 1..=96 {
            state.tick(&mut s, true).unwrap();
            let rows = match &state.diagnostic {
                IgnitionDiagnosticState::Fresh(v) => &v.rows,
                _ => unreachable!(),
            };
            assert!(
                !rows[0].burning && !rows[1].burning,
                "short-spike controls must not ignite"
            );
            if rows[2].burning && oil_tick.is_none() {
                oil_tick = Some(tick);
            }
            if rows[3].burning && wood_tick.is_none() {
                wood_tick = Some(tick);
            }
            if oil_tick.is_some() && wood_tick.is_some() {
                break;
            }
        }
        assert!(
            oil_tick.is_some() && wood_tick.is_some(),
            "finite hot-Stone budget must reach both outputs"
        );
        assert!(
            oil_tick < wood_tick,
            "Oil must ignite before Wood: {oil_tick:?} vs {wood_tick:?}"
        );
    }
    #[test]
    fn scene_two_previous_flame_target_ignites_before_matched_inert_control() {
        let mut s = sim();
        let mut state = IgnitionKineticsState::new(&mut s).unwrap();
        state
            .select_scene(&mut s, IgnitionKineticsScene::FlameVsInert)
            .unwrap();
        let mut flame_tick = None;
        let mut inert_tick = None;
        for tick in 1..=80 {
            state.tick(&mut s, true).unwrap();
            let rows = match &state.diagnostic {
                IgnitionDiagnosticState::Fresh(v) => &v.rows,
                _ => unreachable!(),
            };
            if rows[0].burning && flame_tick.is_none() {
                flame_tick = Some(tick);
            }
            if rows[1].burning && inert_tick.is_none() {
                inert_tick = Some(tick);
            }
            if flame_tick.is_some() && inert_tick.is_some() {
                break;
            }
        }
        assert!(flame_tick.is_some() && inert_tick.is_some());
        assert!(
            flame_tick < inert_tick,
            "previous flame must accelerate a matched target"
        );
    }
    #[test]
    fn scene_three_connected_region_does_not_flash_on_first_tick() {
        let mut s = sim();
        let mut state = IgnitionKineticsState::new(&mut s).unwrap();
        state
            .select_scene(&mut s, IgnitionKineticsScene::SurfaceFront)
            .unwrap();
        state.tick(&mut s, true).unwrap();
        let sample = match &state.diagnostic {
            IgnitionDiagnosticState::Fresh(v) => v,
            _ => unreachable!(),
        };
        assert_eq!(
            sample.newly_ignited_count, 0,
            "dose prevents whole-region first-tick ignition"
        );
        assert_eq!(sample.burning_count, 0);
    }
}
