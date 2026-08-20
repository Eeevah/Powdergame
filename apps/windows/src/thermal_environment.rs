//! TE-2 user-observable passive Thermal Environment candidate.

use powdergame_core::{
    advected_energy, air_temperature_celsius_like, canonical_directed_face_flow,
    classify_air_state, derived_air_pressure, donor_outflow_scale, raw_directed_air_flow,
    receiver_accept_scale, standard_air_state, vacuum_air_state, AirState, EnvironmentBoundaryMode,
    EnvironmentClass, AIR_THERMAL_CONDUCTIVITY, EMPTY_EMPTY_AIR_PERMEABILITY,
    MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_STONE, STANDARD_AIR_ENERGY,
    THERMAL_BASE_STEP, THERMAL_DEADBAND_C, THERMAL_MAX_MIX_FRACTION,
};
use powdergame_gpu::{GpuError, Simulation};

use crate::renderer::WorldTransform;

pub(crate) const TE2_TITLE: &str = "Powdergame TE-2 Passive Thermal Environment";
pub(crate) const TE2_WORLD_WIDTH: u32 = 256;
pub(crate) const TE2_WORLD_HEIGHT: u32 = 192;
pub(crate) const TE2_CHUNK_SIZE: u32 = 64;
pub(crate) const TE2_TPS: u32 = 60;
pub(crate) const TE2_SAMPLE_INTERVAL_TICKS: u64 = 8;
const TE2_ACCOUNTING_READBACK_BATCH_CELLS: usize = 64;

const COMPARISON_FRAME_X0: i64 = 86;
const COMPARISON_FRAME_X1: i64 = 120;
const COMPARISON_SOURCE_X0: i64 = 87;
const COMPARISON_SOURCE_X1: i64 = 102;
const COMPARISON_GAP_X: i64 = 103;
const COMPARISON_TARGET_X1: i64 = 119;
const COMPARISON_HALF_HEIGHT: i64 = 4;
const COMPARISON_SOURCE_TEMPERATURE_C: f32 = 300.0;
const COMPARISON_TARGET_TEMPERATURE_C: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThermalEnvironmentScene {
    DirectAtmosphereVacuum,
    AtmosphereRefillsVacuum,
    SealedCooling,
    ReservoirCooling,
}

impl ThermalEnvironmentScene {
    pub(crate) fn number(self) -> u8 {
        match self {
            Self::DirectAtmosphereVacuum => 1,
            Self::AtmosphereRefillsVacuum => 2,
            Self::SealedCooling => 3,
            Self::ReservoirCooling => 4,
        }
    }

    pub(crate) fn from_number(number: u8) -> Option<Self> {
        Some(match number {
            1 => Self::DirectAtmosphereVacuum,
            2 => Self::AtmosphereRefillsVacuum,
            3 => Self::SealedCooling,
            4 => Self::ReservoirCooling,
            _ => return None,
        })
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::DirectAtmosphereVacuum => "Direct / Atmosphere / Vacuum",
            Self::AtmosphereRefillsVacuum => "Atmosphere refills Vacuum",
            Self::SealedCooling => "Sealed cooling",
            Self::ReservoirCooling => "Fixed-reservoir cooling",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::DirectAtmosphereVacuum => {
                "Compare direct contact, one Air gap, and one Vacuum gap."
            }
            Self::AtmosphereRefillsVacuum => "A sealed Air corridor relaxes into connected Vacuum.",
            Self::SealedCooling => "Hot Stone exchanges heat with a sealed Atmosphere corridor.",
            Self::ReservoirCooling => "The same corridor has one explicit standard-reservoir edge.",
        }
    }

    pub(crate) fn boundary_mode(self) -> EnvironmentBoundaryMode {
        match self {
            Self::ReservoirCooling => EnvironmentBoundaryMode::FixedStandardAtmosphereReservoir,
            _ => EnvironmentBoundaryMode::Sealed,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ThermalEnvironmentSampleRow {
    pub label: &'static str,
    pub cell: (u32, u32),
    pub material_temperature_c: Option<f32>,
    pub environment_class: &'static str,
    pub air_mass: f32,
    pub air_temperature_c: Option<f32>,
    pub derived_pressure: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ThermalEnvironmentAccounting {
    pub label: &'static str,
    pub air_mass: f64,
    pub air_energy: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ThermalEnvironmentSample {
    pub generation: u64,
    pub simulation_tick: u64,
    pub sequence: u64,
    pub rows: Vec<ThermalEnvironmentSampleRow>,
    pub accounting: Option<ThermalEnvironmentAccounting>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ThermalEnvironmentDiagnosticState {
    Sampling {
        generation: u64,
        sequence: u64,
        simulation_tick: u64,
    },
    Fresh(ThermalEnvironmentSample),
    Failed {
        generation: u64,
        sequence: u64,
        simulation_tick: u64,
        message: String,
    },
}

impl ThermalEnvironmentDiagnosticState {
    pub(crate) fn fresh_sample(&self) -> Option<&ThermalEnvironmentSample> {
        match self {
            Self::Fresh(sample) => Some(sample),
            Self::Sampling { .. } | Self::Failed { .. } => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ThermalEnvironmentHudData {
    pub scene: ThermalEnvironmentScene,
    pub playing: bool,
    pub fast: u32,
    pub simulation_tick: u64,
    pub diagnostic: ThermalEnvironmentDiagnosticState,
    pub details_visible: bool,
    pub last_step_tick: Option<u64>,
    pub world_transform: Option<WorldTransform>,
    pub cumulative_external_air_mass: f64,
    pub cumulative_external_advected_energy: f64,
    pub cumulative_external_passive_heat: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SampleRequest {
    generation: u64,
    sequence: u64,
    simulation_tick: u64,
}

#[derive(Clone, Copy, Debug, Default)]
struct ReservoirExchange {
    mass: f64,
    advected_energy: f64,
    passive_heat: f64,
}

pub(crate) struct ThermalEnvironmentState {
    scene: ThermalEnvironmentScene,
    diagnostic: ThermalEnvironmentDiagnosticState,
    generation: u64,
    next_sequence: u64,
    exchange: ReservoirExchange,
    details_visible: bool,
    last_step_tick: Option<u64>,
}

impl ThermalEnvironmentState {
    pub(crate) fn new(simulation: &mut Simulation) -> Result<Self, GpuError> {
        let mut state = Self {
            scene: ThermalEnvironmentScene::DirectAtmosphereVacuum,
            diagnostic: ThermalEnvironmentDiagnosticState::Sampling {
                generation: 0,
                sequence: 0,
                simulation_tick: simulation.tick_count,
            },
            generation: 0,
            next_sequence: 1,
            exchange: ReservoirExchange::default(),
            details_visible: true,
            last_step_tick: None,
        };
        state.reset(simulation)?;
        Ok(state)
    }

    pub(crate) fn select_scene(
        &mut self,
        simulation: &mut Simulation,
        scene: ThermalEnvironmentScene,
    ) -> Result<(), GpuError> {
        self.scene = scene;
        self.reset(simulation)
    }

    pub(crate) fn reset(&mut self, simulation: &mut Simulation) -> Result<(), GpuError> {
        self.begin_generation(simulation.tick_count);
        if let Err(error) = stage_scene(simulation, self.scene) {
            self.record_failure(
                simulation.tick_count,
                format!("scene/reset staging failed: {error}"),
            );
            return Err(error);
        }
        self.next_sequence = 1;
        self.exchange = ReservoirExchange::default();
        self.last_step_tick = None;
        self.sample_if_due(simulation, true);
        Ok(())
    }

    pub(crate) fn tick_playing(&mut self, simulation: &mut Simulation) -> Result<(), GpuError> {
        self.last_step_tick = None;
        let diagnostic_error = self.tick_production(simulation)?;
        if let Some(message) = diagnostic_error {
            self.record_failure(simulation.tick_count, message);
        } else {
            self.sample_if_due(simulation, false);
        }
        Ok(())
    }

    pub(crate) fn single_step(&mut self, simulation: &mut Simulation) -> Result<(), GpuError> {
        let diagnostic_error = self.tick_production(simulation)?;
        self.last_step_tick = Some(simulation.tick_count);
        if let Some(message) = diagnostic_error {
            self.record_failure(simulation.tick_count, message);
        } else {
            self.sample_if_due(simulation, true);
        }
        Ok(())
    }

    fn tick_production(&mut self, simulation: &mut Simulation) -> Result<Option<String>, GpuError> {
        let exchange = if self.scene == ThermalEnvironmentScene::ReservoirCooling {
            reservoir_exchange_for_next_tick(simulation)
        } else {
            Ok(ReservoirExchange::default())
        };
        simulation.tick()?;
        match exchange {
            Ok(exchange) => {
                self.exchange.mass += exchange.mass;
                self.exchange.advected_energy += exchange.advected_energy;
                self.exchange.passive_heat += exchange.passive_heat;
                Ok(None)
            }
            Err(error) => Ok(Some(format!(
                "external-exchange readback failed before committed tick {}: {error}",
                simulation.tick_count
            ))),
        }
    }

    fn begin_generation(&mut self, simulation_tick: u64) {
        self.generation = self.generation.wrapping_add(1).max(1);
        self.diagnostic = ThermalEnvironmentDiagnosticState::Sampling {
            generation: self.generation,
            sequence: 0,
            simulation_tick,
        };
        self.last_step_tick = None;
    }

    fn begin_sample(&mut self, simulation_tick: u64) -> SampleRequest {
        let request = SampleRequest {
            generation: self.generation,
            sequence: self.next_sequence,
            simulation_tick,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.diagnostic = ThermalEnvironmentDiagnosticState::Sampling {
            generation: request.generation,
            sequence: request.sequence,
            simulation_tick: request.simulation_tick,
        };
        request
    }

    fn record_failure(&mut self, simulation_tick: u64, message: String) {
        let request = self.begin_sample(simulation_tick);
        let _ = self.commit_sample_result(request, Err(message));
    }

    fn commit_sample_result(
        &mut self,
        request: SampleRequest,
        result: Result<ThermalEnvironmentSample, String>,
    ) -> bool {
        if request.generation != self.generation
            || !matches!(
                self.diagnostic,
                ThermalEnvironmentDiagnosticState::Sampling {
                    generation,
                    sequence,
                    simulation_tick,
                } if generation == request.generation
                    && sequence == request.sequence
                    && simulation_tick == request.simulation_tick
            )
        {
            return false;
        }
        self.diagnostic = match result {
            Ok(sample) => ThermalEnvironmentDiagnosticState::Fresh(sample),
            Err(message) => ThermalEnvironmentDiagnosticState::Failed {
                generation: request.generation,
                sequence: request.sequence,
                simulation_tick: request.simulation_tick,
                message,
            },
        };
        true
    }

    fn sample_if_due(&mut self, simulation: &Simulation, force: bool) {
        let tick = simulation.tick_count;
        if !force
            && (!tick.is_multiple_of(TE2_SAMPLE_INTERVAL_TICKS)
                || self.diagnostic.fresh_sample().is_some_and(|sample| {
                    sample.generation == self.generation && sample.simulation_tick == tick
                }))
        {
            return;
        }
        let request = self.begin_sample(tick);
        let result = collect_sample(simulation, self.scene, request).map_err(|error| {
            format!("bounded diagnostic readback failed at committed tick {tick}: {error}")
        });
        let _ = self.commit_sample_result(request, result);
    }

    pub(crate) fn toggle_details(&mut self) -> bool {
        self.details_visible = !self.details_visible;
        self.details_visible
    }

    pub(crate) fn clear_step_applied(&mut self) {
        self.last_step_tick = None;
    }

    pub(crate) fn hud_data(
        &self,
        playing: bool,
        fast: u32,
        tick: u64,
        world_transform: Option<WorldTransform>,
    ) -> ThermalEnvironmentHudData {
        ThermalEnvironmentHudData {
            scene: self.scene,
            playing,
            fast,
            simulation_tick: tick,
            diagnostic: self.diagnostic.clone(),
            details_visible: self.details_visible,
            last_step_tick: self.last_step_tick,
            world_transform,
            cumulative_external_air_mass: self.exchange.mass,
            cumulative_external_advected_energy: self.exchange.advected_energy,
            cumulative_external_passive_heat: self.exchange.passive_heat,
        }
    }
}

fn collect_sample(
    simulation: &Simulation,
    scene: ThermalEnvironmentScene,
    request: SampleRequest,
) -> Result<ThermalEnvironmentSample, GpuError> {
    let cells = sample_cells(scene);
    let environments = simulation.world.read_environment_cells(
        &simulation.context.device,
        &simulation.context.queue,
        &cells.iter().map(|(_, cell)| *cell).collect::<Vec<_>>(),
    )?;
    let mut rows = Vec::with_capacity(cells.len());
    for ((label, (x, y)), environment) in cells.into_iter().zip(environments) {
        let material = simulation.world.read_material_cell(
            &simulation.context.device,
            &simulation.context.queue,
            x,
            y,
        )?;
        let material_temperature_c = if material == MATERIAL_EMPTY {
            None
        } else {
            Some(simulation.world.read_temperature_cell(
                &simulation.context.device,
                &simulation.context.queue,
                x,
                y,
            )?)
        };
        rows.push(ThermalEnvironmentSampleRow {
            label,
            cell: (x as u32, y as u32),
            material_temperature_c,
            environment_class: if material == MATERIAL_EMPTY {
                class_name(classify_air_state(environment.current).ok())
            } else {
                "Occupied / no Air"
            },
            air_mass: environment.current.mass,
            air_temperature_c: air_temperature_celsius_like(environment.current),
            derived_pressure: derived_air_pressure(environment.current),
        });
    }
    let accounting = accounting_cells(scene)
        .map(|(label, cells)| collect_accounting(simulation, label, &cells))
        .transpose()?;
    Ok(ThermalEnvironmentSample {
        generation: request.generation,
        simulation_tick: request.simulation_tick,
        sequence: request.sequence,
        rows,
        accounting,
    })
}

fn collect_accounting(
    simulation: &Simulation,
    label: &'static str,
    cells: &[(i64, i64)],
) -> Result<ThermalEnvironmentAccounting, GpuError> {
    let mut air_mass = 0.0_f64;
    let mut air_energy = 0.0_f64;
    for batch in cells.chunks(TE2_ACCOUNTING_READBACK_BATCH_CELLS) {
        let states = simulation.world.read_environment_cells(
            &simulation.context.device,
            &simulation.context.queue,
            batch,
        )?;
        air_mass += states
            .iter()
            .map(|state| f64::from(state.current.mass))
            .sum::<f64>();
        air_energy += states
            .iter()
            .map(|state| f64::from(state.current.energy))
            .sum::<f64>();
    }
    Ok(ThermalEnvironmentAccounting {
        label,
        air_mass,
        air_energy,
    })
}

fn class_name(class: Option<EnvironmentClass>) -> &'static str {
    match class {
        Some(EnvironmentClass::Vacuum) => "Vacuum",
        Some(EnvironmentClass::LowPressure) => "Low pressure Air",
        Some(EnvironmentClass::Atmosphere) => "Atmosphere",
        None => "Occupied / no Air",
    }
}

fn sample_cells(scene: ThermalEnvironmentScene) -> Vec<(&'static str, (i64, i64))> {
    match scene {
        ThermalEnvironmentScene::DirectAtmosphereVacuum => vec![
            ("Direct source", (COMPARISON_SOURCE_X1, 40)),
            ("Direct target", (COMPARISON_GAP_X, 40)),
            ("Atmosphere source", (COMPARISON_SOURCE_X1, 96)),
            ("Atmosphere gap", (COMPARISON_GAP_X, 96)),
            ("Atmosphere target", (COMPARISON_GAP_X + 1, 96)),
            ("Vacuum source", (COMPARISON_SOURCE_X1, 152)),
            ("Vacuum gap", (COMPARISON_GAP_X, 152)),
            ("Vacuum target", (COMPARISON_GAP_X + 1, 152)),
        ],
        ThermalEnvironmentScene::AtmosphereRefillsVacuum => vec![
            ("Atmosphere left", (72, 96)),
            ("Interface left", (126, 96)),
            ("Interface right", (129, 96)),
            ("Vacuum right", (184, 96)),
        ],
        ThermalEnvironmentScene::SealedCooling => vec![
            ("Hot Stone", (248, 96)),
            ("Near Air", (249, 96)),
            ("Mid Air", (253, 96)),
            ("Sealed edge Air", (254, 96)),
        ],
        ThermalEnvironmentScene::ReservoirCooling => vec![
            ("Hot Stone", (248, 96)),
            ("Near Air", (249, 96)),
            ("Mid Air", (253, 96)),
            ("Reservoir edge Air", (255, 96)),
        ],
    }
}

fn accounting_cells(scene: ThermalEnvironmentScene) -> Option<(&'static str, Vec<(i64, i64)>)> {
    match scene {
        ThermalEnvironmentScene::DirectAtmosphereVacuum => None,
        ThermalEnvironmentScene::AtmosphereRefillsVacuum => Some((
            "Sealed corridor total",
            (48..=207).map(|x| (x, 96)).collect(),
        )),
        ThermalEnvironmentScene::SealedCooling => Some((
            "Sealed chamber total",
            (224..=255).map(|x| (x, 96)).collect(),
        )),
        ThermalEnvironmentScene::ReservoirCooling => Some((
            "Reservoir chamber total",
            (224..=255).map(|x| (x, 96)).collect(),
        )),
    }
}

fn stage_scene(
    simulation: &mut Simulation,
    scene: ThermalEnvironmentScene,
) -> Result<(), GpuError> {
    simulation.reset()?;
    simulation.set_environment_boundary_mode(scene.boundary_mode());
    match scene {
        ThermalEnvironmentScene::DirectAtmosphereVacuum => stage_comparison(simulation),
        ThermalEnvironmentScene::AtmosphereRefillsVacuum => stage_refill(simulation),
        ThermalEnvironmentScene::SealedCooling => stage_cooling(simulation, false),
        ThermalEnvironmentScene::ReservoirCooling => stage_cooling(simulation, true),
    }
}

fn write_material(sim: &Simulation, x: i64, y: i64, material: u32) -> Result<(), GpuError> {
    sim.world.write_material(&sim.context.queue, x, y, material)
}

fn write_temperature(sim: &Simulation, x: i64, y: i64, value: f32) -> Result<(), GpuError> {
    sim.world.write_temperature(&sim.context.queue, x, y, value)
}

fn wall_lane(sim: &Simulation, x0: i64, x1: i64, y: i64) -> Result<(), GpuError> {
    for x in x0..=x1 {
        write_material(sim, x, y - 1, MATERIAL_BOUNDARY_BLOCK)?;
        write_material(sim, x, y + 1, MATERIAL_BOUNDARY_BLOCK)?;
    }
    if x0 > 0 {
        write_material(sim, x0 - 1, y, MATERIAL_BOUNDARY_BLOCK)?;
    }
    if x1 < i64::from(TE2_WORLD_WIDTH) - 1 {
        write_material(sim, x1 + 1, y, MATERIAL_BOUNDARY_BLOCK)?;
    }
    Ok(())
}

fn wall_box(sim: &Simulation, x0: i64, x1: i64, y: i64) -> Result<(), GpuError> {
    let top = y - COMPARISON_HALF_HEIGHT - 1;
    let bottom = y + COMPARISON_HALF_HEIGHT + 1;
    for x in x0..=x1 {
        write_material(sim, x, top, MATERIAL_BOUNDARY_BLOCK)?;
        write_material(sim, x, bottom, MATERIAL_BOUNDARY_BLOCK)?;
    }
    for row in (top + 1)..bottom {
        write_material(sim, x0, row, MATERIAL_BOUNDARY_BLOCK)?;
        write_material(sim, x1, row, MATERIAL_BOUNDARY_BLOCK)?;
    }
    Ok(())
}

fn fill_stone_block(
    sim: &Simulation,
    x0: i64,
    x1: i64,
    center_y: i64,
    temperature_c: f32,
) -> Result<(), GpuError> {
    for y in (center_y - COMPARISON_HALF_HEIGHT)..=(center_y + COMPARISON_HALF_HEIGHT) {
        for x in x0..=x1 {
            write_material(sim, x, y, MATERIAL_STONE)?;
            write_temperature(sim, x, y, temperature_c)?;
        }
    }
    Ok(())
}

fn stage_comparison(sim: &Simulation) -> Result<(), GpuError> {
    for (lane, y) in [40, 96, 152].into_iter().enumerate() {
        wall_box(sim, COMPARISON_FRAME_X0, COMPARISON_FRAME_X1, y)?;
        fill_stone_block(
            sim,
            COMPARISON_SOURCE_X0,
            COMPARISON_SOURCE_X1,
            y,
            COMPARISON_SOURCE_TEMPERATURE_C,
        )?;
        let target_x0 = if lane == 0 {
            COMPARISON_GAP_X
        } else {
            COMPARISON_GAP_X + 1
        };
        fill_stone_block(
            sim,
            target_x0,
            COMPARISON_TARGET_X1,
            y,
            COMPARISON_TARGET_TEMPERATURE_C,
        )?;
        if lane == 2 {
            for gap_y in (y - COMPARISON_HALF_HEIGHT)..=(y + COMPARISON_HALF_HEIGHT) {
                sim.world.write_environment_cell_for_test(
                    &sim.context.queue,
                    COMPARISON_GAP_X,
                    gap_y,
                    vacuum_air_state(),
                )?;
            }
        }
    }
    Ok(())
}

fn stage_refill(sim: &Simulation) -> Result<(), GpuError> {
    wall_lane(sim, 48, 207, 96)?;
    for x in 128..=207 {
        sim.world
            .write_environment_cell_for_test(&sim.context.queue, x, 96, vacuum_air_state())?;
    }
    Ok(())
}

fn stage_cooling(sim: &Simulation, reservoir: bool) -> Result<(), GpuError> {
    wall_lane(sim, 224, 255, 96)?;
    write_material(sim, 248, 96, MATERIAL_STONE)?;
    write_temperature(sim, 248, 96, 300.0)?;
    if reservoir {
        write_material(sim, 256 - 1, 96, MATERIAL_EMPTY)?;
    }
    Ok(())
}

fn reservoir_exchange_for_next_tick(sim: &Simulation) -> Result<ReservoirExchange, GpuError> {
    let states = sim.world.read_environment_cells(
        &sim.context.device,
        &sim.context.queue,
        &[(255, 96), (254, 96), (253, 96)],
    )?;
    let edge = states[0].current;
    let inner = states[1].current;
    let inner2 = states[2].current;
    let standard = standard_air_state();

    let edge_out_external = raw_directed_air_flow(edge, standard, EMPTY_EMPTY_AIR_PERMEABILITY);
    let edge_out_inner = raw_directed_air_flow(edge, inner, EMPTY_EMPTY_AIR_PERMEABILITY);
    let edge_donor_scale = donor_outflow_scale(edge.mass, edge_out_external + edge_out_inner);
    let edge_in_external = raw_directed_air_flow(standard, edge, EMPTY_EMPTY_AIR_PERMEABILITY);
    let edge_in_inner = raw_directed_air_flow(inner, edge, EMPTY_EMPTY_AIR_PERMEABILITY);
    let edge_receiver_scale = receiver_accept_scale(
        edge,
        edge_in_external + edge_in_inner,
        advected_energy(edge_in_external, standard).unwrap_or(0.0)
            + advected_energy(edge_in_inner, inner).unwrap_or(0.0),
    )
    .unwrap_or(0.0);

    let inner_out_edge = raw_directed_air_flow(inner, edge, EMPTY_EMPTY_AIR_PERMEABILITY);
    let inner_out_inner2 = raw_directed_air_flow(inner, inner2, EMPTY_EMPTY_AIR_PERMEABILITY);
    let inner_donor_scale = donor_outflow_scale(inner.mass, inner_out_edge + inner_out_inner2);
    let inner_in_edge = raw_directed_air_flow(edge, inner, EMPTY_EMPTY_AIR_PERMEABILITY);
    let inner_in_inner2 = raw_directed_air_flow(inner2, inner, EMPTY_EMPTY_AIR_PERMEABILITY);
    let inner_receiver_scale = receiver_accept_scale(
        inner,
        inner_in_edge + inner_in_inner2,
        advected_energy(inner_in_edge, edge).unwrap_or(0.0)
            + advected_energy(inner_in_inner2, inner2).unwrap_or(0.0),
    )
    .unwrap_or(0.0);

    let external_out = edge_out_external * edge_donor_scale;
    let external_in = edge_in_external * edge_receiver_scale;
    let edge_to_inner = canonical_directed_face_flow(
        edge,
        inner,
        edge_donor_scale,
        inner_receiver_scale,
        EMPTY_EMPTY_AIR_PERMEABILITY,
    )
    .unwrap_or(AirState {
        mass: 0.0,
        energy: 0.0,
    });
    let inner_to_edge = canonical_directed_face_flow(
        inner,
        edge,
        inner_donor_scale,
        edge_receiver_scale,
        EMPTY_EMPTY_AIR_PERMEABILITY,
    )
    .unwrap_or(AirState {
        mass: 0.0,
        energy: 0.0,
    });
    let external_advected =
        external_in * STANDARD_AIR_ENERGY - advected_energy(external_out, edge).unwrap_or(0.0);
    let edge_after_transport = AirState {
        mass: edge.mass + external_in - external_out + inner_to_edge.mass - edge_to_inner.mass,
        energy: edge.energy + external_advected + inner_to_edge.energy - edge_to_inner.energy,
    };
    let edge_temperature = air_temperature_celsius_like(edge_after_transport).unwrap_or(20.0);
    let conductance_sum = AIR_THERMAL_CONDUCTIVITY * 2.0;
    let lambda = if conductance_sum > 0.0 {
        (THERMAL_MAX_MIX_FRACTION * edge_after_transport.mass
            / (THERMAL_BASE_STEP * conductance_sum))
            .min(1.0)
    } else {
        0.0
    };
    let delta = 20.0 - edge_temperature;
    let passive_heat = if delta.abs() > THERMAL_DEADBAND_C {
        THERMAL_BASE_STEP * lambda * AIR_THERMAL_CONDUCTIVITY * delta
    } else {
        0.0
    };
    Ok(ReservoirExchange {
        mass: (external_in - external_out) as f64,
        advected_energy: external_advected as f64,
        passive_heat: passive_heat as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use powdergame_core::{WorldConfig, MATTER_AIR_INTERFACE_CONDUCTANCE, THERMAL_C_STONE};

    fn config() -> WorldConfig {
        WorldConfig::new(TE2_WORLD_WIDTH, TE2_WORLD_HEIGHT, TE2_CHUNK_SIZE).unwrap()
    }

    fn scene_signature(
        simulation: &Simulation,
        scene: ThermalEnvironmentScene,
    ) -> Vec<(u32, u32, u32, u32)> {
        sample_cells(scene)
            .into_iter()
            .map(|(_, (x, y))| {
                let material = simulation
                    .world
                    .read_material_cell(&simulation.context.device, &simulation.context.queue, x, y)
                    .unwrap();
                let temperature = simulation
                    .world
                    .read_temperature_cell(
                        &simulation.context.device,
                        &simulation.context.queue,
                        x,
                        y,
                    )
                    .unwrap();
                let environment = simulation
                    .world
                    .read_environment_cells(
                        &simulation.context.device,
                        &simulation.context.queue,
                        &[(x, y)],
                    )
                    .unwrap()[0]
                    .current;
                (
                    material,
                    temperature.to_bits(),
                    environment.mass.to_bits(),
                    environment.energy.to_bits(),
                )
            })
            .collect()
    }

    fn fresh_sample(state: &ThermalEnvironmentState) -> &ThermalEnvironmentSample {
        state
            .diagnostic
            .fresh_sample()
            .unwrap_or_else(|| panic!("candidate diagnostic must be Fresh: {:?}", state.diagnostic))
    }

    fn material_temperature(sample: &ThermalEnvironmentSample, label: &str) -> f32 {
        sample
            .rows
            .iter()
            .find(|row| row.label == label)
            .and_then(|row| row.material_temperature_c)
            .unwrap_or_else(|| panic!("missing Matter temperature for {label}"))
    }

    #[test]
    fn scene_identity_and_boundary_modes_are_exact() {
        for number in 1..=4 {
            let scene = ThermalEnvironmentScene::from_number(number).unwrap();
            assert_eq!(scene.number(), number);
            assert!(!scene.name().is_empty());
            assert!(!scene.description().is_empty());
        }
        assert_eq!(ThermalEnvironmentScene::from_number(0), None);
        assert_eq!(ThermalEnvironmentScene::from_number(5), None);
        assert_eq!(
            ThermalEnvironmentScene::ReservoirCooling.boundary_mode(),
            EnvironmentBoundaryMode::FixedStandardAtmosphereReservoir
        );
        assert_eq!(
            ThermalEnvironmentScene::SealedCooling.boundary_mode(),
            EnvironmentBoundaryMode::Sealed
        );
    }

    #[test]
    fn sample_contract_is_bounded_and_never_expands_the_product_inspector() {
        for scene in [
            ThermalEnvironmentScene::DirectAtmosphereVacuum,
            ThermalEnvironmentScene::AtmosphereRefillsVacuum,
            ThermalEnvironmentScene::SealedCooling,
            ThermalEnvironmentScene::ReservoirCooling,
        ] {
            assert!(sample_cells(scene).len() <= 8);
            assert!(
                accounting_cells(scene)
                    .map(|(_, cells)| cells.len() <= 160)
                    .unwrap_or(true),
                "candidate accounting remains a fixed bounded corridor"
            );
        }
        assert_eq!(crate::inspector::INSPECTOR_READBACK_BYTES, 24);
        assert!(
            crate::inspector::INSPECTOR_SAMPLE_INTERVAL >= std::time::Duration::from_millis(100)
        );

        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = ThermalEnvironmentState::new(&mut simulation).unwrap();
        assert!(state.details_visible);
        assert!(!state.toggle_details());
        assert!(state.toggle_details());
    }

    #[test]
    fn candidate_world_and_celsius_constants_are_pinned() {
        assert_eq!(
            WorldConfig::new(TE2_WORLD_WIDTH, TE2_WORLD_HEIGHT, TE2_CHUNK_SIZE)
                .unwrap()
                .width,
            256
        );
        assert_eq!(MATTER_AIR_INTERFACE_CONDUCTANCE, 0.05);
        assert_eq!(THERMAL_C_STONE, 2.0);
    }

    #[test]
    fn all_candidate_scenes_stage_from_pristine_and_reset_exactly() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = ThermalEnvironmentState::new(&mut simulation).unwrap();
        for scene in [
            ThermalEnvironmentScene::DirectAtmosphereVacuum,
            ThermalEnvironmentScene::AtmosphereRefillsVacuum,
            ThermalEnvironmentScene::SealedCooling,
            ThermalEnvironmentScene::ReservoirCooling,
        ] {
            state.select_scene(&mut simulation, scene).unwrap();
            assert_eq!(simulation.tick_count, 0);
            let initial_signature = scene_signature(&simulation, scene);
            let initial_sample = fresh_sample(&state).clone();
            let initial_generation = state.generation;
            state.tick_playing(&mut simulation).unwrap();
            assert_eq!(simulation.tick_count, 1);
            state.reset(&mut simulation).unwrap();
            assert_eq!(simulation.tick_count, 0);
            assert_eq!(scene_signature(&simulation, scene), initial_signature);
            let reset_sample = fresh_sample(&state);
            assert_eq!(reset_sample.simulation_tick, 0);
            assert_eq!(reset_sample.rows, initial_sample.rows);
            assert_eq!(reset_sample.accounting, initial_sample.accounting);
            assert!(state.generation > initial_generation);
            assert_eq!(state.exchange.mass, 0.0);
            assert_eq!(state.exchange.advected_energy, 0.0);
            assert_eq!(state.exchange.passive_heat, 0.0);
        }
    }

    #[test]
    fn paused_single_step_forces_tick_one_sample_and_playing_keeps_eight_tick_cadence() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = ThermalEnvironmentState::new(&mut simulation).unwrap();
        let tick_zero = fresh_sample(&state).clone();
        assert_eq!(tick_zero.simulation_tick, 0);

        state.single_step(&mut simulation).unwrap();
        let tick_one = fresh_sample(&state);
        assert_eq!(simulation.tick_count, 1);
        assert_eq!(tick_one.simulation_tick, 1);
        assert_eq!(tick_one.sequence, tick_zero.sequence + 1);
        assert_eq!(state.last_step_tick, Some(1));
        assert!(
            material_temperature(tick_one, "Direct target")
                > material_temperature(&tick_zero, "Direct target")
        );

        state.reset(&mut simulation).unwrap();
        let reset_sequence = fresh_sample(&state).sequence;
        for expected_tick in 1..TE2_SAMPLE_INTERVAL_TICKS {
            state.tick_playing(&mut simulation).unwrap();
            assert_eq!(simulation.tick_count, expected_tick);
            assert_eq!(fresh_sample(&state).simulation_tick, 0);
            assert_eq!(fresh_sample(&state).sequence, reset_sequence);
        }
        state.tick_playing(&mut simulation).unwrap();
        assert_eq!(simulation.tick_count, TE2_SAMPLE_INTERVAL_TICKS);
        assert_eq!(
            fresh_sample(&state).simulation_tick,
            TE2_SAMPLE_INTERVAL_TICKS
        );
        assert_eq!(fresh_sample(&state).sequence, reset_sequence + 1);
    }

    #[test]
    fn rapid_single_steps_remain_ordered_and_each_commits_a_fresh_sample() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = ThermalEnvironmentState::new(&mut simulation).unwrap();
        let mut last_sequence = fresh_sample(&state).sequence;
        for expected_tick in 1..=3 {
            state.single_step(&mut simulation).unwrap();
            let sample = fresh_sample(&state);
            assert_eq!(simulation.tick_count, expected_tick);
            assert_eq!(sample.simulation_tick, expected_tick);
            assert_eq!(sample.sequence, last_sequence + 1);
            last_sequence = sample.sequence;
        }
    }

    #[test]
    fn failed_and_late_sample_results_never_reuse_old_rows() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = ThermalEnvironmentState::new(&mut simulation).unwrap();
        assert!(fresh_sample(&state).rows.len() >= 4);

        let older = state.begin_sample(simulation.tick_count);
        let newer = state.begin_sample(simulation.tick_count);
        assert!(!state.commit_sample_result(older, Err("late old sample".to_string())));
        assert!(matches!(
            state.diagnostic,
            ThermalEnvironmentDiagnosticState::Sampling { sequence, .. }
                if sequence == newer.sequence
        ));
        assert!(state.commit_sample_result(newer, Err("map failed".to_string())));
        assert!(state.diagnostic.fresh_sample().is_none());
        assert!(matches!(
            &state.diagnostic,
            ThermalEnvironmentDiagnosticState::Failed { message, .. }
                if message == "map failed"
        ));

        let previous_generation = state.generation;
        let late_from_reset = state.begin_sample(simulation.tick_count);
        state.reset(&mut simulation).unwrap();
        assert!(state.generation > previous_generation);
        assert!(!state.commit_sample_result(late_from_reset, Err("late after reset".to_string())));
        assert_eq!(fresh_sample(&state).simulation_tick, 0);

        let reset_generation = state.generation;
        state
            .select_scene(
                &mut simulation,
                ThermalEnvironmentScene::AtmosphereRefillsVacuum,
            )
            .unwrap();
        assert!(state.generation > reset_generation);
        assert_eq!(fresh_sample(&state).simulation_tick, 0);
    }

    #[test]
    fn scene_one_semantic_checkpoints_make_direct_then_air_then_vacuum_order_visible() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = ThermalEnvironmentState::new(&mut simulation).unwrap();

        for checkpoint in [0_u64, 1, 8, 60, 300] {
            while simulation.tick_count < checkpoint {
                if simulation.tick_count == 0 {
                    state.single_step(&mut simulation).unwrap();
                } else {
                    state.tick_playing(&mut simulation).unwrap();
                }
            }
            if fresh_sample(&state).simulation_tick != checkpoint {
                state.sample_if_due(&simulation, true);
            }
            let sample = fresh_sample(&state);
            let direct = material_temperature(sample, "Direct target");
            let atmosphere = material_temperature(sample, "Atmosphere target");
            let vacuum_target = material_temperature(sample, "Vacuum target");
            println!(
                "[te2 checkpoint] tick {checkpoint}: direct {direct:.6} C | atmosphere {atmosphere:.6} C | vacuum {vacuum_target:.6} C"
            );
            assert!(
                direct >= atmosphere,
                "tick {checkpoint}: {direct} < {atmosphere}"
            );
            assert!(
                atmosphere >= vacuum_target,
                "tick {checkpoint}: {atmosphere} < {vacuum_target}"
            );
            if checkpoint >= 1 {
                assert!(direct > COMPARISON_TARGET_TEMPERATURE_C);
            }
            if checkpoint >= 60 {
                assert!(atmosphere > COMPARISON_TARGET_TEMPERATURE_C);
                assert!(direct > atmosphere);
                assert_eq!(vacuum_target, COMPARISON_TARGET_TEMPERATURE_C);
            }
        }
        let sample = fresh_sample(&state);
        let vacuum_gap = sample
            .rows
            .iter()
            .find(|row| row.label == "Vacuum gap")
            .unwrap();
        assert_eq!(vacuum_gap.environment_class, "Vacuum");
        assert_eq!(vacuum_gap.air_mass, 0.0);
    }

    #[test]
    fn atmosphere_corridor_refills_connected_vacuum_with_production_air_transport() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = ThermalEnvironmentState::new(&mut simulation).unwrap();
        state
            .select_scene(
                &mut simulation,
                ThermalEnvironmentScene::AtmosphereRefillsVacuum,
            )
            .unwrap();
        let before = simulation
            .world
            .read_environment_cells(
                &simulation.context.device,
                &simulation.context.queue,
                &[(128, 96), (129, 96)],
            )
            .unwrap();
        assert_eq!(before[0].current.mass, 0.0);
        assert_eq!(before[1].current.mass, 0.0);
        let before_accounting = fresh_sample(&state).accounting.clone().unwrap();
        for _ in 0..64 {
            state.tick_playing(&mut simulation).unwrap();
        }
        state.sample_if_due(&simulation, true);
        let after = simulation
            .world
            .read_environment_cells(
                &simulation.context.device,
                &simulation.context.queue,
                &[(128, 96), (129, 96)],
            )
            .unwrap();
        assert!(after[0].current.mass > 0.0);
        assert!(after[1].current.mass > 0.0);
        assert!(after.iter().all(|cell| cell.current == cell.next));
        let after_sample = fresh_sample(&state);
        let after_accounting = after_sample.accounting.as_ref().unwrap();
        assert!((after_accounting.air_mass - before_accounting.air_mass).abs() < 1.0e-3);
        assert!((after_accounting.air_energy - before_accounting.air_energy).abs() < 1.0e-2);
        assert!(after_sample.rows.iter().all(|row| row.air_mass >= 0.0));
    }

    #[test]
    fn reservoir_accounting_is_explicit_and_sealed_scene_has_none() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = ThermalEnvironmentState::new(&mut simulation).unwrap();
        state
            .select_scene(&mut simulation, ThermalEnvironmentScene::SealedCooling)
            .unwrap();
        for _ in 0..16 {
            state.tick_playing(&mut simulation).unwrap();
        }
        assert_eq!(state.exchange.mass, 0.0);
        assert_eq!(state.exchange.advected_energy, 0.0);
        assert_eq!(state.exchange.passive_heat, 0.0);

        state
            .select_scene(&mut simulation, ThermalEnvironmentScene::ReservoirCooling)
            .unwrap();
        for _ in 0..512 {
            state.tick_playing(&mut simulation).unwrap();
        }
        assert!(state.exchange.mass.is_finite());
        assert!(state.exchange.advected_energy.is_finite());
        assert!(state.exchange.passive_heat.is_finite());
        assert!(
            state.exchange.mass != 0.0
                || state.exchange.advected_energy != 0.0
                || state.exchange.passive_heat != 0.0
        );
    }
}
