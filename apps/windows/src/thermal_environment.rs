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

pub(crate) const TE2_TITLE: &str = "Powdergame TE-2 Passive Thermal Environment";
pub(crate) const TE2_WORLD_WIDTH: u32 = 256;
pub(crate) const TE2_WORLD_HEIGHT: u32 = 192;
pub(crate) const TE2_CHUNK_SIZE: u32 = 64;
pub(crate) const TE2_TPS: u32 = 60;
pub(crate) const TE2_SAMPLE_INTERVAL_TICKS: u64 = 8;

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

#[derive(Clone, Debug)]
pub(crate) struct ThermalEnvironmentSampleRow {
    pub label: &'static str,
    pub material_temperature_c: Option<f32>,
    pub environment_class: &'static str,
    pub air_mass: f32,
    pub air_temperature_c: Option<f32>,
    pub derived_pressure: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct ThermalEnvironmentSample {
    pub simulation_tick: u64,
    pub sequence: u64,
    pub rows: Vec<ThermalEnvironmentSampleRow>,
}

#[derive(Clone, Debug)]
pub(crate) struct ThermalEnvironmentHudData {
    pub scene: ThermalEnvironmentScene,
    pub playing: bool,
    pub fast: u32,
    pub simulation_tick: u64,
    pub sample: Option<ThermalEnvironmentSample>,
    pub cumulative_external_air_mass: f64,
    pub cumulative_external_advected_energy: f64,
    pub cumulative_external_passive_heat: f64,
}

#[derive(Clone, Copy, Debug, Default)]
struct ReservoirExchange {
    mass: f64,
    advected_energy: f64,
    passive_heat: f64,
}

pub(crate) struct ThermalEnvironmentState {
    scene: ThermalEnvironmentScene,
    sample: Option<ThermalEnvironmentSample>,
    next_sequence: u64,
    exchange: ReservoirExchange,
}

impl ThermalEnvironmentState {
    pub(crate) fn new(simulation: &mut Simulation) -> Result<Self, GpuError> {
        let mut state = Self {
            scene: ThermalEnvironmentScene::DirectAtmosphereVacuum,
            sample: None,
            next_sequence: 1,
            exchange: ReservoirExchange::default(),
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
        stage_scene(simulation, self.scene)?;
        self.sample = None;
        self.next_sequence = 1;
        self.exchange = ReservoirExchange::default();
        self.sample_if_due(simulation, true)
    }

    pub(crate) fn tick(&mut self, simulation: &mut Simulation) -> Result<(), GpuError> {
        let exchange = if self.scene == ThermalEnvironmentScene::ReservoirCooling {
            reservoir_exchange_for_next_tick(simulation)?
        } else {
            ReservoirExchange::default()
        };
        simulation.tick()?;
        self.exchange.mass += exchange.mass;
        self.exchange.advected_energy += exchange.advected_energy;
        self.exchange.passive_heat += exchange.passive_heat;
        self.sample_if_due(simulation, false)
    }

    pub(crate) fn sample_if_due(
        &mut self,
        simulation: &Simulation,
        force: bool,
    ) -> Result<(), GpuError> {
        let tick = simulation.tick_count;
        if !force
            && (!tick.is_multiple_of(TE2_SAMPLE_INTERVAL_TICKS)
                || self
                    .sample
                    .as_ref()
                    .is_some_and(|sample| sample.simulation_tick == tick))
        {
            return Ok(());
        }
        let cells = sample_cells(self.scene);
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
        self.sample = Some(ThermalEnvironmentSample {
            simulation_tick: tick,
            sequence: self.next_sequence,
            rows,
        });
        self.next_sequence += 1;
        Ok(())
    }

    pub(crate) fn hud_data(
        &self,
        playing: bool,
        fast: u32,
        tick: u64,
    ) -> ThermalEnvironmentHudData {
        ThermalEnvironmentHudData {
            scene: self.scene,
            playing,
            fast,
            simulation_tick: tick,
            sample: self.sample.clone(),
            cumulative_external_air_mass: self.exchange.mass,
            cumulative_external_advected_energy: self.exchange.advected_energy,
            cumulative_external_passive_heat: self.exchange.passive_heat,
        }
    }
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
            ("Direct source", (55, 48)),
            ("Direct target", (56, 48)),
            ("Atmosphere gap", (56, 96)),
            ("Atmosphere target", (57, 96)),
            ("Vacuum gap", (56, 144)),
            ("Vacuum target", (57, 144)),
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

fn stage_comparison(sim: &Simulation) -> Result<(), GpuError> {
    for y in [48, 96, 144] {
        wall_lane(sim, 55, 57, y)?;
        write_material(sim, 55, y, MATERIAL_STONE)?;
        write_temperature(sim, 55, y, 300.0)?;
        write_material(sim, if y == 48 { 56 } else { 57 }, y, MATERIAL_STONE)?;
        write_temperature(sim, if y == 48 { 56 } else { 57 }, y, 20.0)?;
    }
    sim.world
        .write_environment_cell_for_test(&sim.context.queue, 56, 144, vacuum_air_state())
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
            assert!(sample_cells(scene).len() <= 6);
        }
        assert_eq!(crate::inspector::INSPECTOR_READBACK_BYTES, 24);
        assert!(
            crate::inspector::INSPECTOR_SAMPLE_INTERVAL >= std::time::Duration::from_millis(100)
        );
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
            state.tick(&mut simulation).unwrap();
            assert_eq!(simulation.tick_count, 1);
            state.reset(&mut simulation).unwrap();
            assert_eq!(simulation.tick_count, 0);
            assert_eq!(scene_signature(&simulation, scene), initial_signature);
            assert_eq!(state.sample.as_ref().unwrap().simulation_tick, 0);
        }
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
        for _ in 0..64 {
            state.tick(&mut simulation).unwrap();
        }
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
    }

    #[test]
    fn reservoir_accounting_is_explicit_and_sealed_scene_has_none() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = ThermalEnvironmentState::new(&mut simulation).unwrap();
        state
            .select_scene(&mut simulation, ThermalEnvironmentScene::SealedCooling)
            .unwrap();
        for _ in 0..16 {
            state.tick(&mut simulation).unwrap();
        }
        assert_eq!(state.exchange.mass, 0.0);
        assert_eq!(state.exchange.advected_energy, 0.0);
        assert_eq!(state.exchange.passive_heat, 0.0);

        state
            .select_scene(&mut simulation, ThermalEnvironmentScene::ReservoirCooling)
            .unwrap();
        for _ in 0..512 {
            state.tick(&mut simulation).unwrap();
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
