//! TE-3 user-testable Water / Steam phase-cycle candidate.

use powdergame_core::{
    AirState, EmptyEnvironmentSeed, AIR_ZERO_OFFSET, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY,
    MATERIAL_ICE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, TEMPERATURE_REFERENCE,
};
use powdergame_gpu::{GpuError, Simulation};

pub(crate) const TE3_TITLE: &str = "Powdergame TE-3 Water / Steam Phase Cycle";
pub(crate) const TE3_WORLD_WIDTH: u32 = 256;
pub(crate) const TE3_WORLD_HEIGHT: u32 = 192;
pub(crate) const TE3_CHUNK_SIZE: u32 = 64;
pub(crate) const TE3_TPS: u32 = 60;
const SAMPLE_INTERVAL: u64 = 8;
const LV: f32 = 480.0;
const SCENE2_REVEAL_TICK: u64 = 24;
const SCENE4_RESTORE_TICK: u64 = 32;

const SCENE2_SURFACE: (u32, u32) = (72, 112);
const SCENE2_BURIED: (u32, u32) = (172, 112);
const SCENE2_OPENING: (u32, u32) = (172, 111);
const SCENE3_LID: (u32, u32) = (60, 102);
const SCENE3_FREE_AIR: (u32, u32) = (124, 102);
const SCENE3_BOUNDARY: (u32, u32) = (196, 102);
const SCENE4_BOILING: (u32, u32) = (60, 108);
const SCENE4_CONDENSATION: (u32, u32) = (128, 108);
const SCENE4_NO_SINK: (u32, u32) = (196, 108);
const SCENE4_RESTORED_FACE: (u32, u32) = (196, 107);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PhaseCycleScene {
    OpenBeaker,
    SurfaceVersusBuried,
    ColdLidVersusFreeAir,
    ReversalAndNoSink,
}

impl PhaseCycleScene {
    pub(crate) fn from_number(number: u8) -> Option<Self> {
        Some(match number {
            1 => Self::OpenBeaker,
            2 => Self::SurfaceVersusBuried,
            3 => Self::ColdLidVersusFreeAir,
            4 => Self::ReversalAndNoSink,
            _ => return None,
        })
    }

    pub(crate) fn number(self) -> u8 {
        match self {
            Self::OpenBeaker => 1,
            Self::SurfaceVersusBuried => 2,
            Self::ColdLidVersusFreeAir => 3,
            Self::ReversalAndNoSink => 4,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::OpenBeaker => "Open beaker cycle",
            Self::SurfaceVersusBuried => "Surface versus buried Water",
            Self::ColdLidVersusFreeAir => "Cold lid versus free Air",
            Self::ReversalAndNoSink => "Reversal and no-sink controls",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::OpenBeaker => "Heat, 1:1 Steam rise, cooling, condensation, and return.",
            Self::SurfaceVersusBuried => "Equal H; only gas-facing Water may complete boiling.",
            Self::ColdLidVersusFreeAir => {
                "A real cold sink competes with sparse radius-2 nucleation."
            }
            Self::ReversalAndNoSink => {
                "Reverse partial work; isolated no-work Steam stays metastable."
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PhaseCycleSampleRow {
    pub label: &'static str,
    pub cell: (u32, u32),
    pub material: u32,
    pub temperature: f32,
    pub phase_energy: f32,
    pub progress: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PhaseCycleSample {
    pub generation: u64,
    pub sequence: u64,
    pub sample_tick: u64,
    pub rows: Vec<PhaseCycleSampleRow>,
    pub family_count: usize,
    pub water_count: usize,
    pub steam_count: usize,
    pub ice_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PhaseCycleDiagnosticState {
    Sampling {
        generation: u64,
        sequence: u64,
        simulation_tick: u64,
    },
    Fresh(PhaseCycleSample),
    Failed {
        generation: u64,
        sequence: u64,
        simulation_tick: u64,
        message: String,
    },
}

impl PhaseCycleDiagnosticState {
    pub(crate) fn fresh_sample(&self) -> Option<&PhaseCycleSample> {
        match self {
            Self::Fresh(sample) => Some(sample),
            Self::Sampling { .. } | Self::Failed { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SampleRequest {
    generation: u64,
    sequence: u64,
    simulation_tick: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct PhaseCycleHudData {
    pub scene: PhaseCycleScene,
    pub playing: bool,
    pub fast: u32,
    pub simulation_tick: u64,
    pub diagnostic: PhaseCycleDiagnosticState,
    pub details_visible: bool,
}

pub(crate) struct PhaseCycleState {
    scene: PhaseCycleScene,
    diagnostic: PhaseCycleDiagnosticState,
    generation: u64,
    next_sequence: u64,
    details_visible: bool,
}

impl PhaseCycleState {
    pub(crate) fn new(simulation: &mut Simulation) -> Result<Self, GpuError> {
        let mut state = Self {
            scene: PhaseCycleScene::OpenBeaker,
            diagnostic: PhaseCycleDiagnosticState::Sampling {
                generation: 0,
                sequence: 0,
                simulation_tick: simulation.tick_count,
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
        scene: PhaseCycleScene,
    ) -> Result<(), GpuError> {
        self.scene = scene;
        self.reset(simulation)
    }

    pub(crate) fn reset(&mut self, simulation: &mut Simulation) -> Result<(), GpuError> {
        self.begin_generation(simulation.tick_count);
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
        apply_scene_sequence_action(simulation, self.scene)?;
        simulation.tick()?;
        if force_sample || simulation.tick_count.is_multiple_of(SAMPLE_INTERVAL) {
            self.sample_now(simulation);
        }
        Ok(())
    }

    pub(crate) fn hud_data(
        &self,
        playing: bool,
        fast: u32,
        simulation_tick: u64,
    ) -> PhaseCycleHudData {
        PhaseCycleHudData {
            scene: self.scene,
            playing,
            fast,
            simulation_tick,
            diagnostic: self.diagnostic.clone(),
            details_visible: self.details_visible,
        }
    }

    pub(crate) fn toggle_details(&mut self) -> bool {
        self.details_visible = !self.details_visible;
        self.details_visible
    }

    pub(crate) fn measurement_summary(&self) -> String {
        self.diagnostic.fresh_sample().map_or_else(
            || "sample=unavailable".to_string(),
            |sample| {
                format!(
                    "sample_tick={} family={} water={} steam={} ice={} rows={}",
                    sample.sample_tick,
                    sample.family_count,
                    sample.water_count,
                    sample.steam_count,
                    sample.ice_count,
                    sample.rows.len()
                )
            },
        )
    }

    fn begin_generation(&mut self, simulation_tick: u64) {
        self.generation = self.generation.wrapping_add(1).max(1);
        self.diagnostic = PhaseCycleDiagnosticState::Sampling {
            generation: self.generation,
            sequence: 0,
            simulation_tick,
        };
    }

    fn begin_sample(&mut self, simulation_tick: u64) -> SampleRequest {
        let request = SampleRequest {
            generation: self.generation,
            sequence: self.next_sequence,
            simulation_tick,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.diagnostic = PhaseCycleDiagnosticState::Sampling {
            generation: request.generation,
            sequence: request.sequence,
            simulation_tick,
        };
        request
    }

    fn commit_sample_result(
        &mut self,
        request: SampleRequest,
        result: Result<PhaseCycleSample, String>,
    ) -> bool {
        if request.generation != self.generation
            || !matches!(
                self.diagnostic,
                PhaseCycleDiagnosticState::Sampling {
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
            Ok(sample) => PhaseCycleDiagnosticState::Fresh(sample),
            Err(message) => PhaseCycleDiagnosticState::Failed {
                generation: request.generation,
                sequence: request.sequence,
                simulation_tick: request.simulation_tick,
                message,
            },
        };
        true
    }

    fn sample_now(&mut self, simulation: &Simulation) {
        let request = self.begin_sample(simulation.tick_count);
        let result = collect_sample(simulation, self.scene, request)
            .map_err(|error| format!("fixed diagnostic readback failed: {error}"));
        let _ = self.commit_sample_result(request, result);
    }
}

fn collect_sample(
    simulation: &Simulation,
    scene: PhaseCycleScene,
    request: SampleRequest,
) -> Result<PhaseCycleSample, GpuError> {
    let materials = simulation
        .world
        .read_material_all(&simulation.context.device, &simulation.context.queue)?;
    let mut rows = Vec::with_capacity(3);
    for (label, (x, y)) in sample_cells(scene) {
        let material = materials[(y * TE3_WORLD_WIDTH + x) as usize];
        let temperature = simulation.world.read_temperature_cell(
            &simulation.context.device,
            &simulation.context.queue,
            i64::from(x),
            i64::from(y),
        )?;
        let phase_energy = simulation.world.read_phase_energy_cell(
            &simulation.context.device,
            &simulation.context.queue,
            i64::from(x),
            i64::from(y),
        )?;
        rows.push(PhaseCycleSampleRow {
            label,
            cell: (x, y),
            material,
            temperature,
            phase_energy,
            progress: progress_meaning(material, phase_energy),
        });
    }
    let water_count = materials.iter().filter(|&&m| m == MATERIAL_WATER).count();
    let steam_count = materials.iter().filter(|&&m| m == MATERIAL_STEAM).count();
    let ice_count = materials.iter().filter(|&&m| m == MATERIAL_ICE).count();
    Ok(PhaseCycleSample {
        generation: request.generation,
        sequence: request.sequence,
        sample_tick: request.simulation_tick,
        rows,
        family_count: water_count + steam_count + ice_count,
        water_count,
        steam_count,
        ice_count,
    })
}

fn sample_cells(scene: PhaseCycleScene) -> [(&'static str, (u32, u32)); 3] {
    match scene {
        PhaseCycleScene::OpenBeaker => [
            ("Beaker surface", (128, 125)),
            ("Rising route", (128, 105)),
            ("Cold lid", (128, 75)),
        ],
        PhaseCycleScene::SurfaceVersusBuried => [
            ("Surface Water", SCENE2_SURFACE),
            ("Buried Water", SCENE2_BURIED),
            ("Exposed-after-open result", SCENE2_BURIED),
        ],
        PhaseCycleScene::ColdLidVersusFreeAir => [
            ("Lid Steam", SCENE3_LID),
            ("Free-Air Steam", SCENE3_FREE_AIR),
            ("Boundary-control Steam", SCENE3_BOUNDARY),
        ],
        PhaseCycleScene::ReversalAndNoSink => [
            ("Boiling reversal", SCENE4_BOILING),
            ("Condensation reversal", SCENE4_CONDENSATION),
            ("No-sink Steam", SCENE4_NO_SINK),
        ],
    }
}

fn progress_meaning(material: u32, phase_energy: f32) -> String {
    match material {
        MATERIAL_WATER if phase_energy > 0.0 => {
            format!("boiling {:.1}%", 100.0 * phase_energy / LV)
        }
        MATERIAL_WATER if phase_energy < 0.0 => {
            format!("freezing {:.1}%", 100.0 * -phase_energy / 80.0)
        }
        MATERIAL_STEAM => format!("condensing {:.1}%", 100.0 * (LV - phase_energy) / LV),
        MATERIAL_ICE => format!("melting {:.1}%", 100.0 * (phase_energy + 80.0) / 80.0),
        MATERIAL_EMPTY => "no foreground phase".to_string(),
        _ => "canonical / no phase progress".to_string(),
    }
}

fn stage_scene(simulation: &mut Simulation, scene: PhaseCycleScene) -> Result<(), GpuError> {
    let count = (TE3_WORLD_WIDTH * TE3_WORLD_HEIGHT) as usize;
    let mut material = vec![MATERIAL_EMPTY; count];
    let mut temperature = vec![TEMPERATURE_REFERENCE; count];
    let mut phase = vec![0.0f32; count];
    let idx = |x: u32, y: u32| (y * TE3_WORLD_WIDTH + x) as usize;
    let mut put = |x: u32, y: u32, m: u32, t: f32, e: f32| {
        let i = idx(x, y);
        material[i] = m;
        temperature[i] = t;
        phase[i] = e;
    };
    for x in 0..TE3_WORLD_WIDTH {
        put(x, 0, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
        put(x, TE3_WORLD_HEIGHT - 1, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
    }
    for y in 0..TE3_WORLD_HEIGHT {
        put(0, y, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
        put(TE3_WORLD_WIDTH - 1, y, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
    }
    match scene {
        PhaseCycleScene::OpenBeaker => {
            for y in 110..160 {
                put(72, y, MATERIAL_STONE, 20.0, 0.0);
                put(184, y, MATERIAL_STONE, 20.0, 0.0);
            }
            for x in 72..=184 {
                put(x, 160, MATERIAL_STONE, 20.0, 0.0);
                put(x, 74, MATERIAL_STONE, -20.0, 0.0);
            }
            for y in 125..160 {
                for x in 73..184 {
                    put(x, y, MATERIAL_WATER, 96.0, 0.0);
                }
            }
            for x in 96..160 {
                put(x, 161, MATERIAL_STONE, 800.0, 0.0);
            }
        }
        PhaseCycleScene::SurfaceVersusBuried => {
            // Equal-H endpoint controls. The surface Cell has an EMPTY face;
            // the buried Cell has four 100 C occupied orthogonal neighbours.
            put(
                SCENE2_SURFACE.0,
                SCENE2_SURFACE.1,
                MATERIAL_WATER,
                100.0,
                LV,
            );
            for (x, y) in [
                (71, 111),
                (73, 111),
                (71, 112),
                (73, 112),
                (71, 113),
                (72, 113),
                (73, 113),
            ] {
                put(x, y, MATERIAL_STONE, 100.0, 0.0);
            }
            put(SCENE2_BURIED.0, SCENE2_BURIED.1, MATERIAL_WATER, 100.0, LV);
            for y in 111..=113 {
                for x in 171..=173 {
                    if (x, y) == SCENE2_BURIED {
                        continue;
                    }
                    put(x, y, MATERIAL_STONE, 100.0, 0.0);
                }
            }
        }
        PhaseCycleScene::ColdLidVersusFreeAir => {
            // Lane A: one motionless canonical Steam Cell directly under a
            // positive-conductance cold Stone lid.
            for y in 101..=103 {
                for x in 59..=61 {
                    if (x, y) == SCENE3_LID {
                        continue;
                    }
                    let material_id = if (x, y) == (60, 101) {
                        MATERIAL_STONE
                    } else {
                        MATERIAL_BOUNDARY_BLOCK
                    };
                    put(x, y, material_id, 20.0, 0.0);
                }
            }
            put(SCENE3_LID.0, SCENE3_LID.1, MATERIAL_STEAM, 94.0, LV);

            // Lane B: nine canonical Steam controls cannot move up,
            // up-diagonal, or laterally. Their lower orthogonal faces remain
            // EMPTY Air: GAS does not move downward, while TE-2 can cool
            // through that real Air face until radius-2 nucleation is legal.
            for x in 124..=132 {
                put(x, 102, MATERIAL_STEAM, 94.0, LV);
            }
            for x in 123..=133 {
                put(x, 101, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
            }
            put(123, 102, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
            put(133, 102, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);

            // Lane C: a true K=0 Boundary cage. The 20 C Boundary is adjacent
            // but supplies no TE-2 removal work.
            put(
                SCENE3_BOUNDARY.0,
                SCENE3_BOUNDARY.1,
                MATERIAL_STEAM,
                94.0,
                LV,
            );
            for y in 101..=103 {
                for x in 195..=197 {
                    if (x, y) != SCENE3_BOUNDARY {
                        put(x, y, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
                    }
                }
            }
            // Long K=0 dividers keep the three thermal controls independent.
            for y in 80..125 {
                put(94, y, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
                put(162, y, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
            }
        }
        PhaseCycleScene::ReversalAndNoSink => {
            put(
                SCENE4_BOILING.0,
                SCENE4_BOILING.1,
                MATERIAL_WATER,
                100.0,
                240.0,
            );
            for y in 107..=109 {
                for x in 59..=61 {
                    if (x, y) != SCENE4_BOILING {
                        let t = if (x, y) == (60, 107) { 20.0 } else { 100.0 };
                        put(x, y, MATERIAL_STONE, t, 0.0);
                    }
                }
            }

            put(
                SCENE4_CONDENSATION.0,
                SCENE4_CONDENSATION.1,
                MATERIAL_STEAM,
                100.0,
                240.0,
            );
            for y in 107..=109 {
                for x in 127..=129 {
                    if (x, y) != SCENE4_CONDENSATION {
                        let t = if (x, y) == (128, 107) { 300.0 } else { 100.0 };
                        put(x, y, MATERIAL_STONE, t, 0.0);
                    }
                }
            }

            put(SCENE4_NO_SINK.0, SCENE4_NO_SINK.1, MATERIAL_STEAM, 60.0, LV);
            for y in 107..=109 {
                for x in 195..=197 {
                    if (x, y) != SCENE4_NO_SINK {
                        put(x, y, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
                    }
                }
            }
        }
    }
    simulation.reset()?;
    let queue = &simulation.context.queue;
    let u32_bytes = bytemuck::cast_slice(&material);
    let t_bytes = bytemuck::cast_slice(&temperature);
    let e_bytes = bytemuck::cast_slice(&phase);
    queue.write_buffer(&simulation.world.material_current, 0, u32_bytes);
    queue.write_buffer(&simulation.world.material_next, 0, u32_bytes);
    queue.write_buffer(&simulation.world.temperature_current, 0, t_bytes);
    queue.write_buffer(&simulation.world.temperature_next, 0, t_bytes);
    queue.write_buffer(&simulation.world.phase_energy_current, 0, e_bytes);
    queue.write_buffer(&simulation.world.phase_energy_next, 0, e_bytes);
    simulation.world.stage_environment_for_materials(
        queue,
        &material,
        EmptyEnvironmentSeed::StandardAtmosphere,
    )?;
    if scene == PhaseCycleScene::SurfaceVersusBuried {
        // Scene 2 compares completion permission, not cooling. A uniform 100 C
        // Atmosphere keeps both endpoint controls at equal H until the
        // predeclared opening is made.
        let hot_air_energy = material
            .iter()
            .map(|&material_id| {
                if material_id == MATERIAL_EMPTY {
                    100.0 + AIR_ZERO_OFFSET
                } else {
                    0.0
                }
            })
            .collect::<Vec<f32>>();
        let bytes = bytemuck::cast_slice(&hot_air_energy);
        queue.write_buffer(&simulation.world.air_energy_current, 0, bytes);
        queue.write_buffer(&simulation.world.air_energy_next, 0, bytes);
    }
    queue.submit([]);
    simulation
        .context
        .device
        .poll(wgpu::PollType::Wait)
        .map_err(|e| GpuError::Other(format!("TE-3 staging wait failed: {e}")))?;
    Ok(())
}

fn apply_scene_sequence_action(
    simulation: &Simulation,
    scene: PhaseCycleScene,
) -> Result<(), GpuError> {
    let next_tick = simulation.tick_count.saturating_add(1);
    match scene {
        PhaseCycleScene::SurfaceVersusBuried if next_tick == SCENE2_REVEAL_TICK => {
            open_at_temperature(simulation, SCENE2_OPENING, 100.0)?;
        }
        PhaseCycleScene::ReversalAndNoSink if next_tick == SCENE4_RESTORE_TICK => {
            simulation.world.write_material(
                &simulation.context.queue,
                i64::from(SCENE4_RESTORED_FACE.0),
                i64::from(SCENE4_RESTORED_FACE.1),
                MATERIAL_STONE,
            )?;
        }
        _ => {}
    }
    Ok(())
}

fn open_at_temperature(
    simulation: &Simulation,
    (x, y): (u32, u32),
    temperature_c: f32,
) -> Result<(), GpuError> {
    simulation.world.write_material(
        &simulation.context.queue,
        i64::from(x),
        i64::from(y),
        MATERIAL_EMPTY,
    )?;
    let index = u64::from(y * TE3_WORLD_WIDTH + x);
    let offset = index * 4;
    let air = AirState {
        mass: 1.0,
        energy: temperature_c + AIR_ZERO_OFFSET,
    };
    for buffer in [
        &simulation.world.air_mass_current,
        &simulation.world.air_mass_next,
    ] {
        simulation
            .context
            .queue
            .write_buffer(buffer, offset, &air.mass.to_ne_bytes());
    }
    for buffer in [
        &simulation.world.air_energy_current,
        &simulation.world.air_energy_next,
    ] {
        simulation
            .context
            .queue
            .write_buffer(buffer, offset, &air.energy.to_ne_bytes());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use powdergame_core::WorldConfig;

    fn config() -> WorldConfig {
        WorldConfig::new(TE3_WORLD_WIDTH, TE3_WORLD_HEIGHT, TE3_CHUNK_SIZE).unwrap()
    }

    fn material(simulation: &Simulation, (x, y): (u32, u32)) -> u32 {
        simulation
            .world
            .read_material_cell(
                &simulation.context.device,
                &simulation.context.queue,
                i64::from(x),
                i64::from(y),
            )
            .unwrap()
    }

    fn temperature(simulation: &Simulation, (x, y): (u32, u32)) -> f32 {
        simulation
            .world
            .read_temperature_cell(
                &simulation.context.device,
                &simulation.context.queue,
                i64::from(x),
                i64::from(y),
            )
            .unwrap()
    }

    fn energy(simulation: &Simulation, (x, y): (u32, u32)) -> f32 {
        simulation
            .world
            .read_phase_energy_cell(
                &simulation.context.device,
                &simulation.context.queue,
                i64::from(x),
                i64::from(y),
            )
            .unwrap()
    }

    fn fresh(state: &PhaseCycleState) -> &PhaseCycleSample {
        state
            .diagnostic
            .fresh_sample()
            .unwrap_or_else(|| panic!("expected Fresh diagnostic: {:?}", state.diagnostic))
    }

    fn advance(state: &mut PhaseCycleState, simulation: &mut Simulation, tick: u64) {
        while simulation.tick_count < tick {
            state.tick(simulation, true).unwrap();
        }
    }

    #[test]
    fn fixed_rows_are_bounded_named_and_product_inspector_is_unchanged() {
        for scene in [
            PhaseCycleScene::OpenBeaker,
            PhaseCycleScene::SurfaceVersusBuried,
            PhaseCycleScene::ColdLidVersusFreeAir,
            PhaseCycleScene::ReversalAndNoSink,
        ] {
            let rows = sample_cells(scene);
            assert_eq!(rows.len(), 3);
            assert!(rows.iter().all(|(label, _)| !label.is_empty()));
        }
        assert_eq!(crate::inspector::INSPECTOR_READBACK_BYTES, 24);
        assert!(
            crate::inspector::INSPECTOR_SAMPLE_INTERVAL >= std::time::Duration::from_millis(100)
        );
    }

    #[test]
    fn scene_two_surface_buried_and_reopen_checkpoints_are_real_ticks() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = PhaseCycleState::new(&mut simulation).unwrap();
        state
            .select_scene(&mut simulation, PhaseCycleScene::SurfaceVersusBuried)
            .unwrap();

        assert_eq!(temperature(&simulation, SCENE2_SURFACE), 100.0);
        assert_eq!(temperature(&simulation, SCENE2_BURIED), 100.0);
        assert_eq!(energy(&simulation, SCENE2_SURFACE), LV);
        assert_eq!(energy(&simulation, SCENE2_BURIED), LV);
        assert_eq!(fresh(&state).family_count, 2);

        advance(&mut state, &mut simulation, 1);
        println!(
            "[te3 scene2] tick 1 surface={} T={:.6} E={:.6}; buried={} T={:.6} E={:.6}; family={}",
            material(&simulation, SCENE2_SURFACE),
            temperature(&simulation, SCENE2_SURFACE),
            energy(&simulation, SCENE2_SURFACE),
            material(&simulation, SCENE2_BURIED),
            temperature(&simulation, SCENE2_BURIED),
            energy(&simulation, SCENE2_BURIED),
            fresh(&state).family_count
        );
        assert_eq!(material(&simulation, SCENE2_SURFACE), MATERIAL_STEAM);
        assert_eq!(material(&simulation, SCENE2_BURIED), MATERIAL_WATER);
        assert_eq!(energy(&simulation, SCENE2_BURIED), LV);
        assert_eq!(fresh(&state).family_count, 2);

        advance(&mut state, &mut simulation, SCENE2_REVEAL_TICK - 1);
        assert_eq!(material(&simulation, SCENE2_BURIED), MATERIAL_WATER);
        assert_eq!(energy(&simulation, SCENE2_BURIED), LV);
        advance(&mut state, &mut simulation, SCENE2_REVEAL_TICK);
        println!(
            "[te3 scene2] tick {} opening={} exposed={} T={:.6} E={:.6}; family={}",
            SCENE2_REVEAL_TICK,
            material(&simulation, SCENE2_OPENING),
            material(&simulation, SCENE2_BURIED),
            temperature(&simulation, SCENE2_BURIED),
            energy(&simulation, SCENE2_BURIED),
            fresh(&state).family_count
        );
        assert_eq!(material(&simulation, SCENE2_OPENING), MATERIAL_EMPTY);
        assert_eq!(material(&simulation, SCENE2_BURIED), MATERIAL_STEAM);
        assert_eq!(fresh(&state).family_count, 2);
    }

    #[test]
    fn scene_three_orders_lid_before_free_air_and_boundary_never_acts_as_sink() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = PhaseCycleState::new(&mut simulation).unwrap();
        state
            .select_scene(&mut simulation, PhaseCycleScene::ColdLidVersusFreeAir)
            .unwrap();
        let initial_family = fresh(&state).family_count;
        for cell in [SCENE3_LID, SCENE3_FREE_AIR, SCENE3_BOUNDARY] {
            assert_eq!(material(&simulation, cell), MATERIAL_STEAM);
            assert_eq!(temperature(&simulation, cell), 94.0);
            assert_eq!(energy(&simulation, cell), LV);
        }

        advance(&mut state, &mut simulation, 1);
        println!(
            "[te3 scene3] tick 1 lid T={:.6} E={:.6}; free T={:.6} E={:.6}; boundary T={:.6} E={:.6}",
            temperature(&simulation, SCENE3_LID),
            energy(&simulation, SCENE3_LID),
            temperature(&simulation, SCENE3_FREE_AIR),
            energy(&simulation, SCENE3_FREE_AIR),
            temperature(&simulation, SCENE3_BOUNDARY),
            energy(&simulation, SCENE3_BOUNDARY)
        );
        assert!(energy(&simulation, SCENE3_LID) < LV);
        assert_eq!(energy(&simulation, SCENE3_FREE_AIR), LV);
        assert!((temperature(&simulation, SCENE3_BOUNDARY) - 94.0).abs() <= 1.0e-3);
        assert_eq!(energy(&simulation, SCENE3_BOUNDARY), LV);

        let mut free_air_start = None;
        let mut first_partial_count = None;
        for tick in 2_u64..=120 {
            state.tick(&mut simulation, true).unwrap();
            if free_air_start.is_none() && energy(&simulation, SCENE3_FREE_AIR) < LV {
                free_air_start = Some(tick);
                first_partial_count = Some(
                    (124..=132)
                        .filter(|&x| energy(&simulation, (x, 102)) < LV)
                        .count(),
                );
                break;
            }
        }
        let free_air_start = free_air_start.expect("free-Air control must eventually nucleate");
        let first_partial_count = first_partial_count.expect("nucleation count must be recorded");
        println!(
            "[te3 scene3] free-Air start tick {} T={:.6} E={:.6}; first partial count={}; family={}",
            free_air_start,
            temperature(&simulation, SCENE3_FREE_AIR),
            energy(&simulation, SCENE3_FREE_AIR),
            first_partial_count,
            fresh(&state).family_count
        );
        assert!(free_air_start > 1);
        assert!(first_partial_count < 9);
        assert_eq!(material(&simulation, SCENE3_BOUNDARY), MATERIAL_STEAM);
        assert!((temperature(&simulation, SCENE3_BOUNDARY) - 94.0).abs() <= 1.0e-3);
        assert_eq!(energy(&simulation, SCENE3_BOUNDARY), LV);
        assert_eq!(fresh(&state).family_count, initial_family);
    }

    #[test]
    fn scene_four_reverses_both_directions_and_holds_then_wakes_no_sink() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = PhaseCycleState::new(&mut simulation).unwrap();
        state
            .select_scene(&mut simulation, PhaseCycleScene::ReversalAndNoSink)
            .unwrap();
        let initial_family = fresh(&state).family_count;

        advance(&mut state, &mut simulation, 1);
        println!(
            "[te3 scene4] tick 1 boil T={:.6} E={:.6}; condense T={:.6} E={:.6}; no-sink T={:.6} E={:.6}",
            temperature(&simulation, SCENE4_BOILING),
            energy(&simulation, SCENE4_BOILING),
            temperature(&simulation, SCENE4_CONDENSATION),
            energy(&simulation, SCENE4_CONDENSATION),
            temperature(&simulation, SCENE4_NO_SINK),
            energy(&simulation, SCENE4_NO_SINK)
        );
        assert_eq!(material(&simulation, SCENE4_BOILING), MATERIAL_WATER);
        assert!(energy(&simulation, SCENE4_BOILING) < 240.0);
        assert!(temperature(&simulation, SCENE4_BOILING) >= 99.999);
        assert_eq!(material(&simulation, SCENE4_CONDENSATION), MATERIAL_STEAM);
        assert!(energy(&simulation, SCENE4_CONDENSATION) > 240.0);
        assert!(temperature(&simulation, SCENE4_CONDENSATION) <= 100.001);
        assert_eq!(material(&simulation, SCENE4_NO_SINK), MATERIAL_STEAM);
        assert_eq!(temperature(&simulation, SCENE4_NO_SINK), 60.0);
        assert_eq!(energy(&simulation, SCENE4_NO_SINK), LV);

        advance(&mut state, &mut simulation, SCENE4_RESTORE_TICK - 1);
        assert_eq!(temperature(&simulation, SCENE4_NO_SINK), 60.0);
        assert_eq!(energy(&simulation, SCENE4_NO_SINK), LV);
        advance(&mut state, &mut simulation, SCENE4_RESTORE_TICK);
        println!(
            "[te3 scene4] tick {} restored={} no-sink T={:.6} E={:.6}; family={}",
            SCENE4_RESTORE_TICK,
            material(&simulation, SCENE4_RESTORED_FACE),
            temperature(&simulation, SCENE4_NO_SINK),
            energy(&simulation, SCENE4_NO_SINK),
            fresh(&state).family_count
        );
        assert_eq!(material(&simulation, SCENE4_RESTORED_FACE), MATERIAL_STONE);
        assert!(energy(&simulation, SCENE4_NO_SINK) < LV);
        assert_eq!(fresh(&state).family_count, initial_family);
    }

    #[test]
    fn reset_is_exact_and_late_rows_cannot_cross_scene_generation() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = PhaseCycleState::new(&mut simulation).unwrap();
        state
            .select_scene(&mut simulation, PhaseCycleScene::SurfaceVersusBuried)
            .unwrap();
        let initial = fresh(&state).clone();
        let failed = state.begin_sample(simulation.tick_count);
        assert!(state.commit_sample_result(failed, Err("map failed".to_string())));
        assert!(state.diagnostic.fresh_sample().is_none());
        assert!(matches!(
            &state.diagnostic,
            PhaseCycleDiagnosticState::Failed { message, .. } if message == "map failed"
        ));
        state.reset(&mut simulation).unwrap();
        assert_eq!(fresh(&state).rows, initial.rows);
        let generation = state.generation;
        let late = state.begin_sample(simulation.tick_count);
        state.tick(&mut simulation, true).unwrap();
        state.reset(&mut simulation).unwrap();
        assert!(state.generation > generation);
        assert_eq!(simulation.tick_count, 0);
        assert_eq!(fresh(&state).rows, initial.rows);
        assert_eq!(fresh(&state).family_count, initial.family_count);
        assert!(!state.commit_sample_result(late, Err("late prior scene".to_string())));

        let reset_generation = state.generation;
        state
            .select_scene(&mut simulation, PhaseCycleScene::ColdLidVersusFreeAir)
            .unwrap();
        assert!(state.generation > reset_generation);
        assert_eq!(fresh(&state).sample_tick, 0);
        assert!(fresh(&state)
            .rows
            .iter()
            .all(|row| !matches!(row.label, "Surface Water" | "Buried Water")));
    }

    #[test]
    fn forced_step_commits_one_new_fresh_sample() {
        let mut simulation = pollster::block_on(Simulation::new(config())).unwrap();
        let mut state = PhaseCycleState::new(&mut simulation).unwrap();
        let initial = fresh(&state).clone();
        state.tick(&mut simulation, true).unwrap();
        let stepped = fresh(&state);
        assert_eq!(stepped.sample_tick, 1);
        assert_eq!(stepped.sequence, initial.sequence + 1);
        assert_eq!(stepped.generation, initial.generation);
    }
}
