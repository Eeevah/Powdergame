//! TE-3 user-testable Water / Steam phase-cycle candidate.

use powdergame_core::{
    EmptyEnvironmentSeed, MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_STEAM,
    MATERIAL_STONE, MATERIAL_WATER, TEMPERATURE_REFERENCE,
};
use powdergame_gpu::{GpuError, Simulation};

pub(crate) const TE3_TITLE: &str = "Powdergame TE-3 Water / Steam Phase Cycle";
pub(crate) const TE3_WORLD_WIDTH: u32 = 256;
pub(crate) const TE3_WORLD_HEIGHT: u32 = 192;
pub(crate) const TE3_CHUNK_SIZE: u32 = 64;
pub(crate) const TE3_TPS: u32 = 60;
const SAMPLE_INTERVAL: u64 = 8;

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

#[derive(Clone, Debug)]
pub(crate) struct PhaseCycleSample {
    pub sample_tick: u64,
    pub selected_material: u32,
    pub selected_temperature: f32,
    pub selected_phase_energy: f32,
    pub family_count: usize,
    pub water_count: usize,
    pub steam_count: usize,
    pub ice_count: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct PhaseCycleHudData {
    pub scene: PhaseCycleScene,
    pub playing: bool,
    pub fast: u32,
    pub simulation_tick: u64,
    pub sample: Option<PhaseCycleSample>,
}

pub(crate) struct PhaseCycleState {
    scene: PhaseCycleScene,
    sample: Option<PhaseCycleSample>,
}

impl PhaseCycleState {
    pub(crate) fn new(simulation: &mut Simulation) -> Result<Self, GpuError> {
        let mut state = Self {
            scene: PhaseCycleScene::OpenBeaker,
            sample: None,
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
        stage_scene(simulation, self.scene)?;
        self.sample = None;
        self.sample_now(simulation)
    }

    pub(crate) fn tick(
        &mut self,
        simulation: &mut Simulation,
        force_sample: bool,
    ) -> Result<(), GpuError> {
        simulation.tick()?;
        if force_sample || simulation.tick_count.is_multiple_of(SAMPLE_INTERVAL) {
            self.sample_now(simulation)?;
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
            sample: self.sample.clone(),
        }
    }

    pub(crate) fn measurement_summary(&self) -> String {
        self.sample.as_ref().map_or_else(
            || "sample=unavailable".to_string(),
            |sample| format!(
                "sample_tick={} family={} water={} steam={} ice={} selected_material={} selected_t={:.3} selected_e={:.3}",
                sample.sample_tick, sample.family_count, sample.water_count, sample.steam_count,
                sample.ice_count, sample.selected_material, sample.selected_temperature,
                sample.selected_phase_energy
            ),
        )
    }

    fn sample_now(&mut self, simulation: &Simulation) -> Result<(), GpuError> {
        let materials = simulation
            .world
            .read_material_all(&simulation.context.device, &simulation.context.queue)?;
        let (x, y) = sample_cell(self.scene);
        let selected_material = materials[(y * TE3_WORLD_WIDTH + x) as usize];
        let selected_temperature = simulation.world.read_temperature_cell(
            &simulation.context.device,
            &simulation.context.queue,
            i64::from(x),
            i64::from(y),
        )?;
        let selected_phase_energy = simulation.world.read_phase_energy_cell(
            &simulation.context.device,
            &simulation.context.queue,
            i64::from(x),
            i64::from(y),
        )?;
        let water_count = materials.iter().filter(|&&m| m == MATERIAL_WATER).count();
        let steam_count = materials.iter().filter(|&&m| m == MATERIAL_STEAM).count();
        let ice_count = materials.iter().filter(|&&m| m == MATERIAL_ICE).count();
        self.sample = Some(PhaseCycleSample {
            sample_tick: simulation.tick_count,
            selected_material,
            selected_temperature,
            selected_phase_energy,
            family_count: water_count + steam_count + ice_count,
            water_count,
            steam_count,
            ice_count,
        });
        Ok(())
    }
}

fn sample_cell(scene: PhaseCycleScene) -> (u32, u32) {
    match scene {
        PhaseCycleScene::OpenBeaker => (128, 122),
        PhaseCycleScene::SurfaceVersusBuried => (88, 118),
        PhaseCycleScene::ColdLidVersusFreeAir => (88, 82),
        PhaseCycleScene::ReversalAndNoSink => (82, 104),
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
            for x in 45..211 {
                put(x, 145, MATERIAL_STONE, 20.0, 0.0);
            }
            for y in 119..145 {
                for x in 55..111 {
                    put(x, y, MATERIAL_WATER, 100.0, 479.0);
                }
            }
            for y in 118..145 {
                for x in 145..201 {
                    put(x, y, MATERIAL_WATER, 100.0, 479.0);
                }
            }
            for x in 145..201 {
                put(x, 117, MATERIAL_STONE, 20.0, 0.0);
            }
        }
        PhaseCycleScene::ColdLidVersusFreeAir => {
            for x in 40..216 {
                put(x, 135, MATERIAL_STONE, 20.0, 0.0);
            }
            for x in 50..112 {
                put(x, 82, MATERIAL_STEAM, 68.0, 480.0);
                put(x, 70, MATERIAL_STONE, 20.0, 0.0);
            }
            for x in 145..207 {
                put(x, 82, MATERIAL_STEAM, 68.0, 480.0);
            }
            for y in 65..105 {
                put(128, y, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
            }
        }
        PhaseCycleScene::ReversalAndNoSink => {
            for x in 35..221 {
                put(x, 140, MATERIAL_STONE, 20.0, 0.0);
            }
            put(82, 104, MATERIAL_WATER, 100.0, 240.0);
            put(128, 104, MATERIAL_STEAM, 75.0, 240.0);
            put(180, 104, MATERIAL_STEAM, 60.0, 480.0);
            for y in 90..119 {
                put(166, y, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
                put(194, y, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
            }
            for x in 166..=194 {
                put(x, 90, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
                put(x, 118, MATERIAL_BOUNDARY_BLOCK, 20.0, 0.0);
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
    queue.submit([]);
    simulation
        .context
        .device
        .poll(wgpu::PollType::Wait)
        .map_err(|e| GpuError::Other(format!("TE-3 staging wait failed: {e}")))?;
    Ok(())
}
