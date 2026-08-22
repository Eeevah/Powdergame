//! TE-5R1 user-testable Steam-load relaxing-pressure candidate.

use powdergame_core::{
    derived_air_pressure, is_dynamic_pressure_node, pressure_step_with_phase,
    steam_pressure_target, EmptyEnvironmentSeed, PressureNeighbor, MATERIAL_BOUNDARY_BLOCK,
    MATERIAL_EMPTY, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
    TEMPERATURE_REFERENCE,
};
use powdergame_gpu::{GpuError, Simulation};

use crate::renderer::WorldTransform;

pub(crate) const TE5_TITLE: &str = "Powdergame TE-5 Pressure / Vacuum";
pub(crate) const TE5_WORLD_WIDTH: u32 = 256;
pub(crate) const TE5_WORLD_HEIGHT: u32 = 192;
pub(crate) const TE5_CHUNK_SIZE: u32 = 64;
pub(crate) const TE5_TPS: u32 = 60;
const SAMPLE_INTERVAL: u64 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PressureVacuumScene {
    SparseDense,
    CondensationRelief,
    StructuralDifferential,
    BoilerVent,
}

impl PressureVacuumScene {
    pub(crate) fn from_number(number: u8) -> Option<Self> {
        Some(match number {
            1 => Self::SparseDense,
            2 => Self::CondensationRelief,
            3 => Self::StructuralDifferential,
            4 => Self::BoilerVent,
            _ => return None,
        })
    }

    pub(crate) fn number(self) -> u8 {
        match self {
            Self::SparseDense => 1,
            Self::CondensationRelief => 2,
            Self::StructuralDifferential => 3,
            Self::BoilerVent => 4,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::SparseDense => "Sparse Steam: large versus small chamber",
            Self::CondensationRelief => "Condensation relief with matched control",
            Self::StructuralDifferential => "Uniform versus one-sided differential",
            Self::BoilerVent => "Water heat to real rupture and vent",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::SparseDense => {
                "The same local Steam target spreads differently with available pressure nodes."
            }
            Self::CondensationRelief => {
                "A real cold Stone sink is compared with a K=0 Boundary control."
            }
            Self::StructuralDifferential => {
                "Uniform opposing pressure survives; one-sided total pressure opens Wood."
            }
            Self::BoilerVent => {
                "Production heat, phase, pressure, rupture, Air, and Gas movement form one chain."
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PressureVacuumRow {
    pub label: &'static str,
    pub cell: (u32, u32),
    pub material: u32,
    pub phase_energy: f32,
    pub steam_target: f32,
    pub dynamic_pressure: f32,
    pub air_background: f32,
    pub total_pressure: f32,
    pub predicted_delta: f32,
    pub structure_differential: f32,
    pub air_mass: f32,
    pub air_energy: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PressureVacuumSample {
    pub generation: u64,
    pub sequence: u64,
    pub sample_tick: u64,
    pub rows: Vec<PressureVacuumRow>,
    pub steam_count: usize,
    pub water_count: usize,
    pub wood_count: usize,
    pub family_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PressureVacuumDiagnosticState {
    Sampling {
        generation: u64,
        sequence: u64,
        simulation_tick: u64,
    },
    Fresh(PressureVacuumSample),
    Failed {
        generation: u64,
        sequence: u64,
        simulation_tick: u64,
        message: String,
    },
}

impl PressureVacuumDiagnosticState {
    pub(crate) fn fresh_sample(&self) -> Option<&PressureVacuumSample> {
        match self {
            Self::Fresh(sample) => Some(sample),
            Self::Sampling { .. } | Self::Failed { .. } => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PressureVacuumHudData {
    pub scene: PressureVacuumScene,
    pub playing: bool,
    pub fast: u32,
    pub simulation_tick: u64,
    pub diagnostic: PressureVacuumDiagnosticState,
    pub details_visible: bool,
    pub world_transform: Option<WorldTransform>,
}

pub(crate) struct PressureVacuumState {
    scene: PressureVacuumScene,
    diagnostic: PressureVacuumDiagnosticState,
    generation: u64,
    next_sequence: u64,
    details_visible: bool,
}

impl PressureVacuumState {
    pub(crate) fn new(simulation: &mut Simulation) -> Result<Self, GpuError> {
        let mut state = Self {
            scene: PressureVacuumScene::SparseDense,
            diagnostic: PressureVacuumDiagnosticState::Sampling {
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
        scene: PressureVacuumScene,
    ) -> Result<(), GpuError> {
        self.scene = scene;
        self.reset(simulation)
    }

    pub(crate) fn reset(&mut self, simulation: &mut Simulation) -> Result<(), GpuError> {
        self.generation = self.generation.wrapping_add(1).max(1);
        self.diagnostic = PressureVacuumDiagnosticState::Sampling {
            generation: self.generation,
            sequence: 0,
            simulation_tick: 0,
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
        world_transform: Option<WorldTransform>,
    ) -> PressureVacuumHudData {
        PressureVacuumHudData {
            scene: self.scene,
            playing,
            fast,
            simulation_tick,
            diagnostic: self.diagnostic.clone(),
            details_visible: self.details_visible,
            world_transform,
        }
    }

    pub(crate) fn measurement_summary(&self) -> String {
        match &self.diagnostic {
            PressureVacuumDiagnosticState::Fresh(sample) => format!(
                "sample_tick={} family={} water={} steam={} wood={} rows={}",
                sample.sample_tick,
                sample.family_count,
                sample.water_count,
                sample.steam_count,
                sample.wood_count,
                sample.rows.len()
            ),
            _ => "sample=unavailable".to_string(),
        }
    }

    fn sample_now(&mut self, simulation: &Simulation) {
        let generation = self.generation;
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.diagnostic = PressureVacuumDiagnosticState::Sampling {
            generation,
            sequence,
            simulation_tick: simulation.tick_count,
        };
        self.diagnostic = match collect_sample(simulation, self.scene, generation, sequence) {
            Ok(sample) => PressureVacuumDiagnosticState::Fresh(sample),
            Err(error) => PressureVacuumDiagnosticState::Failed {
                generation,
                sequence,
                simulation_tick: simulation.tick_count,
                message: error.to_string(),
            },
        };
    }
}

fn sample_cells(scene: PressureVacuumScene) -> &'static [(&'static str, (u32, u32))] {
    match scene {
        PressureVacuumScene::SparseDense => &[
            ("Large chamber Steam", (64, 122)),
            ("Large chamber Air", (64, 112)),
            ("Small chamber core", (188, 122)),
            ("Small chamber wall", (188, 105)),
        ],
        PressureVacuumScene::CondensationRelief => &[
            ("Cold-lid Steam", (76, 91)),
            ("Cold Stone sink", (76, 90)),
            ("K=0 control Steam", (180, 91)),
            ("K=0 Boundary", (180, 90)),
        ],
        PressureVacuumScene::StructuralDifferential => &[
            ("Uniform Wood", (72, 96)),
            ("Uniform left face", (71, 96)),
            ("One-sided Wood", (184, 96)),
            ("One-sided source", (183, 96)),
        ],
        PressureVacuumScene::BoilerVent => &[
            ("Boiler Water/Steam", (127, 98)),
            ("Boiler headspace", (128, 96)),
            ("Wood relief plug", (128, 95)),
            ("Outside vent", (128, 94)),
        ],
    }
}

fn collect_sample(
    simulation: &Simulation,
    scene: PressureVacuumScene,
    generation: u64,
    sequence: u64,
) -> Result<PressureVacuumSample, GpuError> {
    let materials = simulation
        .world
        .read_material_all(&simulation.context.device, &simulation.context.queue)?;
    let phase = simulation
        .world
        .read_phase_energy_all(&simulation.context.device, &simulation.context.queue)?;
    let pressure = simulation
        .world
        .read_pressure_all(&simulation.context.device, &simulation.context.queue)?;
    let mut rows = Vec::with_capacity(sample_cells(scene).len());
    for &(label, cell) in sample_cells(scene) {
        let cell_index = index(cell);
        let coordinates = [
            cell,
            (cell.0 - 1, cell.1),
            (cell.0 + 1, cell.1),
            (cell.0, cell.1 - 1),
            (cell.0, cell.1 + 1),
        ];
        let environment = simulation.world.read_environment_cells(
            &simulation.context.device,
            &simulation.context.queue,
            &coordinates
                .iter()
                .map(|&(x, y)| (i64::from(x), i64::from(y)))
                .collect::<Vec<_>>(),
        )?;
        let own_air = environment[0].current;
        let air_background = if materials[cell_index] == MATERIAL_EMPTY {
            derived_air_pressure(own_air)
        } else {
            0.0
        };
        let neighbor_indices = [
            index_of(cell.0 - 1, cell.1),
            index_of(cell.0 + 1, cell.1),
            index_of(cell.0, cell.1 - 1),
            index_of(cell.0, cell.1 + 1),
        ];
        let neighbors = neighbor_indices.map(|neighbor| {
            is_dynamic_pressure_node(materials[neighbor]).then_some(PressureNeighbor {
                material: materials[neighbor],
                pressure: pressure[neighbor],
            })
        });
        let predicted = pressure_step_with_phase(
            materials[cell_index],
            phase[cell_index],
            pressure[cell_index],
            neighbors,
        );
        let totals = neighbor_indices.map(|neighbor| {
            let offset = coordinates
                .iter()
                .position(|&coord| index(coord) == neighbor)
                .expect("neighbor sample");
            pressure[neighbor]
                + if materials[neighbor] == MATERIAL_EMPTY {
                    derived_air_pressure(environment[offset].current)
                } else {
                    0.0
                }
        });
        rows.push(PressureVacuumRow {
            label,
            cell,
            material: materials[cell_index],
            phase_energy: phase[cell_index],
            steam_target: steam_pressure_target(materials[cell_index], phase[cell_index])
                .unwrap_or(0.0),
            dynamic_pressure: pressure[cell_index],
            air_background,
            total_pressure: pressure[cell_index] + air_background,
            predicted_delta: predicted - pressure[cell_index],
            structure_differential: (totals[0] - totals[1])
                .abs()
                .max((totals[2] - totals[3]).abs()),
            air_mass: own_air.mass,
            air_energy: own_air.energy,
        });
    }
    let steam_count = materials.iter().filter(|&&m| m == MATERIAL_STEAM).count();
    let water_count = materials.iter().filter(|&&m| m == MATERIAL_WATER).count();
    Ok(PressureVacuumSample {
        generation,
        sequence,
        sample_tick: simulation.tick_count,
        rows,
        steam_count,
        water_count,
        wood_count: materials.iter().filter(|&&m| m == MATERIAL_WOOD).count(),
        family_count: steam_count + water_count,
    })
}

fn index(cell: (u32, u32)) -> usize {
    index_of(cell.0, cell.1)
}

fn index_of(x: u32, y: u32) -> usize {
    (y * TE5_WORLD_WIDTH + x) as usize
}

#[allow(clippy::too_many_arguments)]
fn put(
    material: &mut [u32],
    temperature: &mut [f32],
    phase: &mut [f32],
    pressure: &mut [f32],
    x: u32,
    y: u32,
    m: u32,
    t: f32,
    e: f32,
    p: f32,
) {
    let i = index_of(x, y);
    material[i] = m;
    temperature[i] = t;
    phase[i] = e;
    pressure[i] = p;
}

#[allow(clippy::too_many_arguments)]
fn ring(
    material: &mut [u32],
    temperature: &mut [f32],
    phase: &mut [f32],
    pressure: &mut [f32],
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
) {
    for x in x0..=x1 {
        put(
            material,
            temperature,
            phase,
            pressure,
            x,
            y0,
            MATERIAL_BOUNDARY_BLOCK,
            20.0,
            0.0,
            0.0,
        );
        put(
            material,
            temperature,
            phase,
            pressure,
            x,
            y1,
            MATERIAL_BOUNDARY_BLOCK,
            20.0,
            0.0,
            0.0,
        );
    }
    for y in y0..=y1 {
        put(
            material,
            temperature,
            phase,
            pressure,
            x0,
            y,
            MATERIAL_BOUNDARY_BLOCK,
            20.0,
            0.0,
            0.0,
        );
        put(
            material,
            temperature,
            phase,
            pressure,
            x1,
            y,
            MATERIAL_BOUNDARY_BLOCK,
            20.0,
            0.0,
            0.0,
        );
    }
}

pub(crate) fn stage_scene(
    simulation: &mut Simulation,
    scene: PressureVacuumScene,
) -> Result<(), GpuError> {
    let count = (TE5_WORLD_WIDTH * TE5_WORLD_HEIGHT) as usize;
    let mut material = vec![MATERIAL_EMPTY; count];
    let mut temperature = vec![TEMPERATURE_REFERENCE; count];
    let mut phase = vec![0.0f32; count];
    let mut pressure = vec![0.0f32; count];
    ring(
        &mut material,
        &mut temperature,
        &mut phase,
        &mut pressure,
        0,
        0,
        TE5_WORLD_WIDTH - 1,
        TE5_WORLD_HEIGHT - 1,
    );
    match scene {
        PressureVacuumScene::SparseDense => {
            ring(
                &mut material,
                &mut temperature,
                &mut phase,
                &mut pressure,
                18,
                34,
                116,
                154,
            );
            ring(
                &mut material,
                &mut temperature,
                &mut phase,
                &mut pressure,
                148,
                84,
                228,
                154,
            );
            put(
                &mut material,
                &mut temperature,
                &mut phase,
                &mut pressure,
                64,
                122,
                MATERIAL_STEAM,
                100.0,
                480.0,
                0.0,
            );
            for y in 116..=128 {
                for x in 182..=194 {
                    put(
                        &mut material,
                        &mut temperature,
                        &mut phase,
                        &mut pressure,
                        x,
                        y,
                        MATERIAL_STEAM,
                        100.0,
                        480.0,
                        0.0,
                    );
                }
            }
        }
        PressureVacuumScene::CondensationRelief => {
            ring(
                &mut material,
                &mut temperature,
                &mut phase,
                &mut pressure,
                42,
                68,
                110,
                142,
            );
            ring(
                &mut material,
                &mut temperature,
                &mut phase,
                &mut pressure,
                146,
                68,
                214,
                142,
            );
            for x in 70..=82 {
                put(
                    &mut material,
                    &mut temperature,
                    &mut phase,
                    &mut pressure,
                    x,
                    90,
                    MATERIAL_STONE,
                    60.0,
                    0.0,
                    0.0,
                );
                put(
                    &mut material,
                    &mut temperature,
                    &mut phase,
                    &mut pressure,
                    x,
                    91,
                    MATERIAL_STEAM,
                    100.0,
                    480.0,
                    60.0,
                );
            }
            for x in 174..=186 {
                put(
                    &mut material,
                    &mut temperature,
                    &mut phase,
                    &mut pressure,
                    x,
                    90,
                    MATERIAL_BOUNDARY_BLOCK,
                    20.0,
                    0.0,
                    0.0,
                );
                put(
                    &mut material,
                    &mut temperature,
                    &mut phase,
                    &mut pressure,
                    x,
                    91,
                    MATERIAL_STEAM,
                    100.0,
                    480.0,
                    60.0,
                );
            }
        }
        PressureVacuumScene::StructuralDifferential => {
            for center in [72u32, 184] {
                for y in 94..=98 {
                    for x in center - 3..=center + 3 {
                        put(
                            &mut material,
                            &mut temperature,
                            &mut phase,
                            &mut pressure,
                            x,
                            y,
                            MATERIAL_BOUNDARY_BLOCK,
                            20.0,
                            0.0,
                            0.0,
                        );
                    }
                }
                put(
                    &mut material,
                    &mut temperature,
                    &mut phase,
                    &mut pressure,
                    center,
                    96,
                    MATERIAL_WOOD,
                    20.0,
                    0.0,
                    0.0,
                );
                put(
                    &mut material,
                    &mut temperature,
                    &mut phase,
                    &mut pressure,
                    center - 1,
                    96,
                    MATERIAL_WATER,
                    20.0,
                    0.0,
                    100.0,
                );
                put(
                    &mut material,
                    &mut temperature,
                    &mut phase,
                    &mut pressure,
                    center + 1,
                    96,
                    MATERIAL_WATER,
                    20.0,
                    0.0,
                    if center == 72 { 100.0 } else { 0.0 },
                );
            }
        }
        PressureVacuumScene::BoilerVent => {
            ring(
                &mut material,
                &mut temperature,
                &mut phase,
                &mut pressure,
                125,
                95,
                131,
                101,
            );
            put(
                &mut material,
                &mut temperature,
                &mut phase,
                &mut pressure,
                128,
                95,
                MATERIAL_WOOD,
                20.0,
                0.0,
                0.0,
            );
            for y in 96..=100 {
                for x in 126..=130 {
                    if (x, y) == (128, 98) {
                        put(
                            &mut material,
                            &mut temperature,
                            &mut phase,
                            &mut pressure,
                            x,
                            y,
                            MATERIAL_STONE,
                            800.0,
                            0.0,
                            0.0,
                        );
                    } else if (x + y) % 2 == 0 {
                        put(
                            &mut material,
                            &mut temperature,
                            &mut phase,
                            &mut pressure,
                            x,
                            y,
                            MATERIAL_STEAM,
                            100.0,
                            480.0,
                            0.0,
                        );
                    } else {
                        let adjacent_heater = x.abs_diff(128) + y.abs_diff(98) == 1;
                        put(
                            &mut material,
                            &mut temperature,
                            &mut phase,
                            &mut pressure,
                            x,
                            y,
                            MATERIAL_WATER,
                            100.0,
                            if adjacent_heater { 475.0 } else { 480.0 },
                            0.0,
                        );
                    }
                }
            }
        }
    }
    simulation.reset()?;
    let q = &simulation.context.queue;
    for buffer in [
        &simulation.world.material_current,
        &simulation.world.material_next,
    ] {
        q.write_buffer(buffer, 0, bytemuck::cast_slice(&material));
    }
    for buffer in [
        &simulation.world.temperature_current,
        &simulation.world.temperature_next,
    ] {
        q.write_buffer(buffer, 0, bytemuck::cast_slice(&temperature));
    }
    for buffer in [
        &simulation.world.phase_energy_current,
        &simulation.world.phase_energy_next,
    ] {
        q.write_buffer(buffer, 0, bytemuck::cast_slice(&phase));
    }
    for buffer in [
        &simulation.world.pressure_current,
        &simulation.world.pressure_next,
    ] {
        q.write_buffer(buffer, 0, bytemuck::cast_slice(&pressure));
    }
    simulation.world.stage_environment_for_materials(
        q,
        &material,
        EmptyEnvironmentSeed::StandardAtmosphere,
    )?;
    q.submit([]);
    simulation
        .context
        .device
        .poll(wgpu::PollType::Wait)
        .map_err(|e| GpuError::Other(format!("TE-5 staging wait failed: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use powdergame_core::WorldConfig;

    fn simulation() -> Simulation {
        pollster::block_on(Simulation::new(
            WorldConfig::new(TE5_WORLD_WIDTH, TE5_WORLD_HEIGHT, TE5_CHUNK_SIZE).unwrap(),
        ))
        .expect("candidate simulation")
    }

    #[test]
    fn scene_reset_produces_fresh_authoritative_rows() {
        let mut sim = simulation();
        let mut state = PressureVacuumState::new(&mut sim).unwrap();
        let first = state.diagnostic.fresh_sample().unwrap();
        assert_eq!(first.sample_tick, 0);
        assert_eq!(first.rows.len(), 4);
        let generation = first.generation;
        state
            .select_scene(&mut sim, PressureVacuumScene::StructuralDifferential)
            .unwrap();
        let reset = state.diagnostic.fresh_sample().unwrap();
        assert!(reset.generation > generation);
        assert_eq!(reset.sample_tick, 0);
        assert_eq!(reset.rows[0].label, "Uniform Wood");
    }

    #[test]
    fn scene_three_uses_real_opposing_face_rupture() {
        let mut sim = simulation();
        stage_scene(&mut sim, PressureVacuumScene::StructuralDifferential).unwrap();
        sim.tick().unwrap();
        assert_eq!(
            sim.world
                .read_material_cell(&sim.context.device, &sim.context.queue, 72, 96)
                .unwrap(),
            MATERIAL_WOOD
        );
        assert_eq!(
            sim.world
                .read_material_cell(&sim.context.device, &sim.context.queue, 184, 96)
                .unwrap(),
            MATERIAL_EMPTY
        );
    }

    #[test]
    fn scene_one_sparse_control_stays_below_wood_threshold_and_dense_load_is_higher() {
        let mut sim = simulation();
        stage_scene(&mut sim, PressureVacuumScene::SparseDense).unwrap();
        for _ in 0..320 {
            sim.tick().unwrap();
        }
        let pressure = sim
            .world
            .read_pressure_all(&sim.context.device, &sim.context.queue)
            .unwrap();
        let sparse_peak = (35..154)
            .flat_map(|y| (19..116).map(move |x| index_of(x, y)))
            .map(|i| pressure[i])
            .fold(0.0f32, f32::max);
        let dense_peak = (85..154)
            .flat_map(|y| (149..228).map(move |x| index_of(x, y)))
            .map(|i| pressure[i])
            .fold(0.0f32, f32::max);
        assert!(sparse_peak < 80.0, "sparse_peak={sparse_peak}");
        assert!(
            dense_peak > sparse_peak + 20.0,
            "sparse={sparse_peak} dense={dense_peak}"
        );
    }

    #[test]
    fn scene_four_preserves_phase_family_quantity_while_heating() {
        let mut sim = simulation();
        stage_scene(&mut sim, PressureVacuumScene::BoilerVent).unwrap();
        let before = sim
            .world
            .read_material_all(&sim.context.device, &sim.context.queue)
            .unwrap();
        let before_family = before
            .iter()
            .filter(|&&m| m == MATERIAL_WATER || m == MATERIAL_STEAM)
            .count();
        for _ in 0..32 {
            sim.tick().unwrap();
        }
        let after = sim
            .world
            .read_material_all(&sim.context.device, &sim.context.queue)
            .unwrap();
        let after_family = after
            .iter()
            .filter(|&&m| m == MATERIAL_WATER || m == MATERIAL_STEAM)
            .count();
        assert_eq!(before_family, after_family);
        assert!(
            after.contains(&MATERIAL_STEAM),
            "real phase pass must produce Steam"
        );
    }

    #[test]
    fn scene_four_runs_the_real_phase_pressure_rupture_and_vent_chain() {
        let mut sim = simulation();
        stage_scene(&mut sim, PressureVacuumScene::BoilerVent).unwrap();
        let initial = sim
            .world
            .read_material_all(&sim.context.device, &sim.context.queue)
            .unwrap();
        let family = initial
            .iter()
            .filter(|&&m| m == MATERIAL_WATER || m == MATERIAL_STEAM)
            .count();
        let mut opened_tick = None;
        for tick in 1..=1_200 {
            sim.tick().unwrap();
            if sim
                .world
                .read_material_cell(&sim.context.device, &sim.context.queue, 128, 95)
                .unwrap()
                == MATERIAL_EMPTY
            {
                opened_tick = Some(tick);
                break;
            }
        }
        let opened_tick =
            opened_tick.expect("Wood plug must open from production total-pressure differential");
        let opening_air_before = sim
            .world
            .read_environment_cells(&sim.context.device, &sim.context.queue, &[(128, 95)])
            .unwrap()[0]
            .current
            .mass;
        for _ in 0..64 {
            sim.tick().unwrap();
        }
        let after = sim
            .world
            .read_material_all(&sim.context.device, &sim.context.queue)
            .unwrap();
        let after_family = after
            .iter()
            .filter(|&&m| m == MATERIAL_WATER || m == MATERIAL_STEAM)
            .count();
        let opening_air_after = sim
            .world
            .read_environment_cells(&sim.context.device, &sim.context.queue, &[(128, 95)])
            .unwrap()[0]
            .current
            .mass;
        assert_eq!(family, after_family);
        assert!(
            opening_air_after > opening_air_before || after[index_of(128, 95)] == MATERIAL_STEAM
        );
        assert!(
            after
                .iter()
                .enumerate()
                .any(|(i, &m)| m == MATERIAL_STEAM && i / TE5_WORLD_WIDTH as usize <= 95),
            "ordinary Gas movement must use the real opening; opened at tick {opened_tick}"
        );
        eprintln!(
            "TE5R1-F21 opened_tick={opened_tick} opening_air_before={opening_air_before:.6} opening_air_after={opening_air_after:.6} family={family}"
        );
    }

    #[test]
    fn scene_four_matched_control_attributes_relief_to_the_real_opening() {
        let mut opened = simulation();
        let mut sealed = simulation();
        stage_scene(&mut opened, PressureVacuumScene::BoilerVent).unwrap();
        stage_scene(&mut sealed, PressureVacuumScene::BoilerVent).unwrap();
        sealed
            .world
            .write_material(&sealed.context.queue, 128, 95, MATERIAL_BOUNDARY_BLOCK)
            .unwrap();
        let mut opening_tick = None;
        for tick in 1..=400 {
            opened.tick().unwrap();
            sealed.tick().unwrap();
            if material_at(&opened, 128, 95) == MATERIAL_EMPTY {
                opening_tick = Some(tick);
                break;
            }
        }
        let opening_tick = opening_tick.expect("treatment must create an opening");
        assert_eq!(material_at(&sealed, 128, 95), MATERIAL_BOUNDARY_BLOCK);
        let treatment_at_open = pressure_at(&opened, 128, 96);
        let control_at_open = pressure_at(&sealed, 128, 96);
        for _ in 0..96 {
            opened.tick().unwrap();
            sealed.tick().unwrap();
        }
        let treatment_later = pressure_at(&opened, 128, 96);
        let control_later = pressure_at(&sealed, 128, 96);
        let treatment_drop = treatment_at_open - treatment_later;
        let control_drop = control_at_open - control_later;
        assert!(
            treatment_drop > control_drop + 5.0,
            "opening_tick={opening_tick} treatment_drop={treatment_drop} control_drop={control_drop} later=({treatment_later},{control_later})"
        );
    }

    #[test]
    fn standard_air_and_exact_vacuum_helpers_match_candidate_labels() {
        assert!(
            (derived_air_pressure(powdergame_core::AirState {
                mass: 1.0,
                energy: powdergame_core::STANDARD_AIR_ENERGY
            }) - 1.0)
                .abs()
                < 1.0e-6
        );
        assert_eq!(
            derived_air_pressure(powdergame_core::AirState {
                mass: 0.0,
                energy: 0.0
            }),
            0.0
        );
    }

    fn material_at(sim: &Simulation, x: i64, y: i64) -> u32 {
        sim.world
            .read_material_cell(&sim.context.device, &sim.context.queue, x, y)
            .unwrap()
    }

    fn pressure_at(sim: &Simulation, x: i64, y: i64) -> f32 {
        sim.world
            .read_pressure_cell(&sim.context.device, &sim.context.queue, x, y)
            .unwrap()
    }
}
