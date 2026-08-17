use std::fmt;
use std::ops::Range;
use std::str::FromStr;

use powdergame_core::{
    chunk_count, chunks_x, initial_material_ids, is_valid_cell_material_value, WorldConfig,
    FLAG_COMBUSTING, FLAG_DECAY_AGE_MASK, FLAG_FLAME_EVENT, FLAG_FUEL_PROGRESS_MASK,
    MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY, MATERIAL_ICE, MATERIAL_OIL, MATERIAL_SAND,
    MATERIAL_SMOKE, MATERIAL_STEAM, MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
    PRESSURE_REFERENCE, TEMPERATURE_REFERENCE,
};

const DESIGN_EXTENT: u64 = 256;
const MIN_OFFICIAL_EXTENT: u32 = 256;
const G7_WIDTH: u32 = 256;
const G7_HEIGHT: u32 = 256;
const G7_CHUNK_SIZE: u32 = 64;
const KNOWN_FLAGS: u32 =
    FLAG_COMBUSTING | FLAG_FLAME_EVENT | FLAG_FUEL_PROGRESS_MASK | FLAG_DECAY_AGE_MASK;

/// Stable identities for the five G8-B workloads plus the frozen G7
/// Active/Sleep regression fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScenarioId {
    SandFall,
    WaterFlow,
    FireHeat,
    PressureBurst,
    HeavyMixedWorld,
    ActiveSleepG7,
}

/// The five workloads that make up the official G8-B performance matrix.
pub const OFFICIAL_G8B_SCENARIOS: [ScenarioId; 5] = [
    ScenarioId::SandFall,
    ScenarioId::WaterFlow,
    ScenarioId::FireHeat,
    ScenarioId::PressureBurst,
    ScenarioId::HeavyMixedWorld,
];

/// All scenarios exposed by the gallery, including the frozen G7 regression
/// fixture. `ActiveSleepG7` is not an official G8-B matrix workload.
pub const GALLERY_SCENARIOS: [ScenarioId; 6] = [
    ScenarioId::SandFall,
    ScenarioId::WaterFlow,
    ScenarioId::FireHeat,
    ScenarioId::PressureBurst,
    ScenarioId::HeavyMixedWorld,
    ScenarioId::ActiveSleepG7,
];

impl ScenarioId {
    pub const fn number(self) -> u8 {
        match self {
            Self::SandFall => 1,
            Self::WaterFlow => 2,
            Self::FireHeat => 3,
            Self::PressureBurst => 4,
            Self::HeavyMixedWorld => 5,
            Self::ActiveSleepG7 => 6,
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::SandFall => "sand-fall",
            Self::WaterFlow => "water-flow",
            Self::FireHeat => "fire-heat",
            Self::PressureBurst => "pressure-burst",
            Self::HeavyMixedWorld => "heavy-mixed-world",
            Self::ActiveSleepG7 => "active-sleep-g7",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::SandFall => "Sand Fall",
            Self::WaterFlow => "Water Flow",
            Self::FireHeat => "Fire / Heat",
            Self::PressureBurst => "Pressure Burst",
            Self::HeavyMixedWorld => "Heavy Mixed World",
            Self::ActiveSleepG7 => "G7 Active / Sleep",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::SandFall => {
                "Dense falling Sand streams with shelves, funnels, and a catch basin."
            }
            Self::WaterFlow => {
                "Large Water reservoirs draining through staggered channels into a basin."
            }
            Self::FireHeat => {
                "Burning Wood and Oil drive combustion, heat transfer, smoke, and phase work."
            }
            Self::PressureBurst => {
                "A hot pressurized chamber drives diffusion, expansion, and a Wood relief seam."
            }
            Self::HeavyMixedWorld => {
                "A heterogeneous world combining movement, density, heat, reaction, and pressure."
            }
            Self::ActiveSleepG7 => "Frozen 256x256 G7 Active/Sleep observatory regression fixture.",
        }
    }

    pub const fn is_official_g8b(self) -> bool {
        !matches!(self, Self::ActiveSleepG7)
    }
}

impl fmt::Display for ScenarioId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

impl FromStr for ScenarioId {
    type Err = ScenarioError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "sand-fall" => Ok(Self::SandFall),
            "water-flow" => Ok(Self::WaterFlow),
            "fire-heat" => Ok(Self::FireHeat),
            "pressure-burst" => Ok(Self::PressureBurst),
            "heavy-mixed-world" => Ok(Self::HeavyMixedWorld),
            "active-sleep-g7" => Ok(Self::ActiveSleepG7),
            _ => Err(ScenarioError::UnknownScenario(value.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum ScenarioError {
    UnknownScenario(String),
    InvalidWorld(String),
    TooSmall {
        scenario: ScenarioId,
        width: u32,
        height: u32,
        minimum: u32,
    },
    ActiveSleepConfig {
        width: u32,
        height: u32,
        chunk_size: u32,
    },
    CoordinateOutOfBounds {
        x: i64,
        y: i64,
        width: u32,
        height: u32,
    },
    InvalidMaterial(u32),
    NonFiniteField {
        field: &'static str,
        value: f32,
    },
    InvalidFixture(String),
    Gpu(String),
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownScenario(value) => write!(
                f,
                "unknown scenario '{value}'; expected one of: {}",
                GALLERY_SCENARIOS
                    .iter()
                    .map(|scenario| scenario.slug())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::InvalidWorld(message) => write!(f, "invalid scenario world: {message}"),
            Self::TooSmall {
                scenario,
                width,
                height,
                minimum,
            } => write!(
                f,
                "scenario {} requires width and height >= {minimum}, got {width}x{height}",
                scenario.slug()
            ),
            Self::ActiveSleepConfig {
                width,
                height,
                chunk_size,
            } => write!(
                f,
                "scenario active-sleep-g7 requires exactly 256x256 with chunk size 64, got {width}x{height} chunk {chunk_size}"
            ),
            Self::CoordinateOutOfBounds {
                x,
                y,
                width,
                height,
            } => write!(f, "scenario coordinate ({x}, {y}) is outside {width}x{height}"),
            Self::InvalidMaterial(value) => write!(f, "invalid scenario material value {value}"),
            Self::NonFiniteField { field, value } => {
                write!(f, "scenario {field} value must be finite, got {value}")
            }
            Self::InvalidFixture(message) => write!(f, "invalid scenario fixture: {message}"),
            Self::Gpu(message) => write!(f, "scenario GPU staging failed: {message}"),
        }
    }
}

impl std::error::Error for ScenarioError {}

/// Complete deterministic CPU image for a scenario at tick 0.
///
/// The four authoritative fields are uploaded to both Current and Next GPU
/// buffers. `chunk_edit_wake` is normally zero; the frozen G7 fixture stores
/// the exact wake markers produced by its historical edit-hook staging path.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioFixture {
    pub(crate) scenario: ScenarioId,
    pub(crate) config: WorldConfig,
    pub(crate) materials: Box<[u32]>,
    pub(crate) temperatures: Box<[f32]>,
    pub(crate) pressures: Box<[f32]>,
    pub(crate) flags: Box<[u32]>,
    pub(crate) chunk_edit_wake: Box<[u32]>,
}

impl ScenarioFixture {
    pub fn build(scenario: ScenarioId, config: WorldConfig) -> Result<Self, ScenarioError> {
        validate_scenario_config(scenario, &config)?;
        let preserve_edit_wake = scenario == ScenarioId::ActiveSleepG7;
        let mut builder = FixtureBuilder::new(config, preserve_edit_wake)?;

        match scenario {
            ScenarioId::SandFall => build_sand_fall(&mut builder)?,
            ScenarioId::WaterFlow => build_water_flow(&mut builder)?,
            ScenarioId::FireHeat => build_fire_heat(&mut builder)?,
            ScenarioId::PressureBurst => build_pressure_burst(&mut builder)?,
            ScenarioId::HeavyMixedWorld => build_heavy_mixed_world(&mut builder)?,
            ScenarioId::ActiveSleepG7 => build_active_sleep_g7(&mut builder)?,
        }

        let fixture = builder.finish(scenario);
        fixture.validate()?;
        Ok(fixture)
    }

    pub const fn scenario(&self) -> ScenarioId {
        self.scenario
    }

    pub const fn config(&self) -> WorldConfig {
        self.config
    }

    pub fn materials(&self) -> &[u32] {
        &self.materials
    }

    pub fn temperatures(&self) -> &[f32] {
        &self.temperatures
    }

    pub fn pressures(&self) -> &[f32] {
        &self.pressures
    }

    pub fn flags(&self) -> &[u32] {
        &self.flags
    }

    pub fn chunk_edit_wake(&self) -> &[u32] {
        &self.chunk_edit_wake
    }

    pub fn validate(&self) -> Result<(), ScenarioError> {
        validate_scenario_config(self.scenario, &self.config)?;
        let cell_count = usize::try_from(
            self.config
                .cell_count()
                .map_err(|error| ScenarioError::InvalidWorld(error.to_string()))?,
        )
        .map_err(|_| ScenarioError::InvalidWorld("cell count does not fit usize".into()))?;
        for (name, actual) in [
            ("materials", self.materials.len()),
            ("temperatures", self.temperatures.len()),
            ("pressures", self.pressures.len()),
            ("flags", self.flags.len()),
        ] {
            if actual != cell_count {
                return Err(ScenarioError::InvalidFixture(format!(
                    "{name} length {actual} does not equal cell count {cell_count}"
                )));
            }
        }

        let expected_chunks = chunk_count(
            self.config.width,
            self.config.height,
            self.config.chunk_size,
        ) as usize;
        if self.chunk_edit_wake.len() != expected_chunks {
            return Err(ScenarioError::InvalidFixture(format!(
                "chunk_edit_wake length {} does not equal chunk count {expected_chunks}",
                self.chunk_edit_wake.len()
            )));
        }

        let width = self.config.width as usize;
        let height = self.config.height as usize;
        for index in 0..cell_count {
            let material = self.materials[index];
            if !is_valid_cell_material_value(material) {
                return Err(ScenarioError::InvalidFixture(format!(
                    "invalid material {material} at index {index}"
                )));
            }
            if !self.temperatures[index].is_finite() {
                return Err(ScenarioError::InvalidFixture(format!(
                    "non-finite temperature at index {index}"
                )));
            }
            if !self.pressures[index].is_finite() {
                return Err(ScenarioError::InvalidFixture(format!(
                    "non-finite pressure at index {index}"
                )));
            }
            if self.flags[index] & !KNOWN_FLAGS != 0 {
                return Err(ScenarioError::InvalidFixture(format!(
                    "unknown flags 0x{:08x} at index {index}",
                    self.flags[index]
                )));
            }
            if material == MATERIAL_EMPTY
                && (self.temperatures[index].to_bits() != TEMPERATURE_REFERENCE.to_bits()
                    || self.pressures[index].to_bits() != PRESSURE_REFERENCE.to_bits()
                    || self.flags[index] != 0)
            {
                return Err(ScenarioError::InvalidFixture(format!(
                    "EMPTY hygiene violation at index {index}"
                )));
            }
        }

        for x in 0..width {
            for y in [0, height - 1] {
                if self.materials[y * width + x] != MATERIAL_BOUNDARY_BLOCK {
                    return Err(ScenarioError::InvalidFixture(format!(
                        "boundary ring changed at ({x}, {y})"
                    )));
                }
            }
        }
        for y in 0..height {
            for x in [0, width - 1] {
                if self.materials[y * width + x] != MATERIAL_BOUNDARY_BLOCK {
                    return Err(ScenarioError::InvalidFixture(format!(
                        "boundary ring changed at ({x}, {y})"
                    )));
                }
            }
        }
        if let Some((index, value)) = self
            .chunk_edit_wake
            .iter()
            .copied()
            .enumerate()
            .find(|(_, value)| *value > 1)
        {
            return Err(ScenarioError::InvalidFixture(format!(
                "chunk_edit_wake[{index}] has non-boolean value {value}"
            )));
        }
        Ok(())
    }
}

/// Validates a selected scenario before any dense fixture allocation or GPU
/// initialization occurs.
pub fn validate_scenario_config(
    scenario: ScenarioId,
    config: &WorldConfig,
) -> Result<(), ScenarioError> {
    config
        .validate()
        .map_err(|error| ScenarioError::InvalidWorld(error.to_string()))?;
    if scenario == ScenarioId::ActiveSleepG7 {
        if config.width != G7_WIDTH
            || config.height != G7_HEIGHT
            || config.chunk_size != G7_CHUNK_SIZE
        {
            return Err(ScenarioError::ActiveSleepConfig {
                width: config.width,
                height: config.height,
                chunk_size: config.chunk_size,
            });
        }
    } else if config.width < MIN_OFFICIAL_EXTENT || config.height < MIN_OFFICIAL_EXTENT {
        return Err(ScenarioError::TooSmall {
            scenario,
            width: config.width,
            height: config.height,
            minimum: MIN_OFFICIAL_EXTENT,
        });
    }
    Ok(())
}

struct FixtureBuilder {
    config: WorldConfig,
    materials: Vec<u32>,
    temperatures: Vec<f32>,
    pressures: Vec<f32>,
    flags: Vec<u32>,
    chunk_edit_wake: Vec<u32>,
    preserve_edit_wake: bool,
}

impl FixtureBuilder {
    fn new(config: WorldConfig, preserve_edit_wake: bool) -> Result<Self, ScenarioError> {
        let materials = initial_material_ids(&config)
            .map_err(|error| ScenarioError::InvalidWorld(error.to_string()))?;
        let cell_count = materials.len();
        let chunk_count = chunk_count(config.width, config.height, config.chunk_size) as usize;
        Ok(Self {
            config,
            materials,
            temperatures: vec![TEMPERATURE_REFERENCE; cell_count],
            pressures: vec![PRESSURE_REFERENCE; cell_count],
            flags: vec![0; cell_count],
            chunk_edit_wake: vec![0; chunk_count],
            preserve_edit_wake,
        })
    }

    fn finish(self, scenario: ScenarioId) -> ScenarioFixture {
        ScenarioFixture {
            scenario,
            config: self.config,
            materials: self.materials.into_boxed_slice(),
            temperatures: self.temperatures.into_boxed_slice(),
            pressures: self.pressures.into_boxed_slice(),
            flags: self.flags.into_boxed_slice(),
            chunk_edit_wake: self.chunk_edit_wake.into_boxed_slice(),
        }
    }

    fn index(&self, x: i64, y: i64) -> Result<usize, ScenarioError> {
        if x < 0 || y < 0 || x >= i64::from(self.config.width) || y >= i64::from(self.config.height)
        {
            return Err(ScenarioError::CoordinateOutOfBounds {
                x,
                y,
                width: self.config.width,
                height: self.config.height,
            });
        }
        Ok(y as usize * self.config.width as usize + x as usize)
    }

    fn mark_edit_wake(&mut self, x: i64, y: i64) {
        if !self.preserve_edit_wake {
            return;
        }
        let chunk_x = x as u32 / self.config.chunk_size;
        let chunk_y = y as u32 / self.config.chunk_size;
        let index =
            (chunk_y * chunks_x(self.config.width, self.config.chunk_size) + chunk_x) as usize;
        self.chunk_edit_wake[index] = 1;
    }

    fn set_material(&mut self, x: i64, y: i64, value: u32) -> Result<(), ScenarioError> {
        if !is_valid_cell_material_value(value) {
            return Err(ScenarioError::InvalidMaterial(value));
        }
        let index = self.index(x, y)?;
        self.materials[index] = value;
        self.flags[index] = 0;
        self.pressures[index] = PRESSURE_REFERENCE;
        if value == MATERIAL_EMPTY {
            self.temperatures[index] = TEMPERATURE_REFERENCE;
        }
        self.mark_edit_wake(x, y);
        Ok(())
    }

    fn set_temperature(&mut self, x: i64, y: i64, value: f32) -> Result<(), ScenarioError> {
        if !value.is_finite() {
            return Err(ScenarioError::NonFiniteField {
                field: "temperature",
                value,
            });
        }
        let index = self.index(x, y)?;
        self.temperatures[index] = value;
        self.mark_edit_wake(x, y);
        Ok(())
    }

    fn set_pressure(&mut self, x: i64, y: i64, value: f32) -> Result<(), ScenarioError> {
        if !value.is_finite() {
            return Err(ScenarioError::NonFiniteField {
                field: "pressure",
                value,
            });
        }
        let index = self.index(x, y)?;
        self.pressures[index] = value;
        self.mark_edit_wake(x, y);
        Ok(())
    }

    fn set_flags(&mut self, x: i64, y: i64, value: u32) -> Result<(), ScenarioError> {
        if value & !KNOWN_FLAGS != 0 {
            return Err(ScenarioError::InvalidFixture(format!(
                "unknown authored flags 0x{value:08x}"
            )));
        }
        let index = self.index(x, y)?;
        self.flags[index] = value;
        self.mark_edit_wake(x, y);
        Ok(())
    }

    fn fill_material(
        &mut self,
        x_range: Range<usize>,
        y_range: Range<usize>,
        value: u32,
    ) -> Result<(), ScenarioError> {
        for y in y_range {
            for x in x_range.clone() {
                self.set_material(x as i64, y as i64, value)?;
            }
        }
        Ok(())
    }

    fn fill_temperature(
        &mut self,
        x_range: Range<usize>,
        y_range: Range<usize>,
        value: f32,
    ) -> Result<(), ScenarioError> {
        for y in y_range {
            for x in x_range.clone() {
                self.set_temperature(x as i64, y as i64, value)?;
            }
        }
        Ok(())
    }

    fn fill_pressure(
        &mut self,
        x_range: Range<usize>,
        y_range: Range<usize>,
        value: f32,
    ) -> Result<(), ScenarioError> {
        for y in y_range {
            for x in x_range.clone() {
                self.set_pressure(x as i64, y as i64, value)?;
            }
        }
        Ok(())
    }

    fn fill_flags(
        &mut self,
        x_range: Range<usize>,
        y_range: Range<usize>,
        value: u32,
    ) -> Result<(), ScenarioError> {
        for y in y_range {
            for x in x_range.clone() {
                self.set_flags(x as i64, y as i64, value)?;
            }
        }
        Ok(())
    }

    fn fill_material_ref(
        &mut self,
        x_start: u32,
        x_end: u32,
        y_start: u32,
        y_end: u32,
        value: u32,
    ) -> Result<(), ScenarioError> {
        self.fill_material(
            scaled_interior_range(x_start, x_end, self.config.width),
            scaled_interior_range(y_start, y_end, self.config.height),
            value,
        )
    }

    fn fill_temperature_ref(
        &mut self,
        x_start: u32,
        x_end: u32,
        y_start: u32,
        y_end: u32,
        value: f32,
    ) -> Result<(), ScenarioError> {
        self.fill_temperature(
            scaled_interior_range(x_start, x_end, self.config.width),
            scaled_interior_range(y_start, y_end, self.config.height),
            value,
        )
    }

    fn fill_pressure_ref(
        &mut self,
        x_start: u32,
        x_end: u32,
        y_start: u32,
        y_end: u32,
        value: f32,
    ) -> Result<(), ScenarioError> {
        self.fill_pressure(
            scaled_interior_range(x_start, x_end, self.config.width),
            scaled_interior_range(y_start, y_end, self.config.height),
            value,
        )
    }

    fn fill_flags_ref(
        &mut self,
        x_start: u32,
        x_end: u32,
        y_start: u32,
        y_end: u32,
        value: u32,
    ) -> Result<(), ScenarioError> {
        self.fill_flags(
            scaled_interior_range(x_start, x_end, self.config.width),
            scaled_interior_range(y_start, y_end, self.config.height),
            value,
        )
    }

    fn fill_material_inclusive(
        &mut self,
        x0: i64,
        y0: i64,
        x1: i64,
        y1: i64,
        value: u32,
    ) -> Result<(), ScenarioError> {
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.set_material(x, y, value)?;
            }
        }
        Ok(())
    }

    fn fill_temperature_inclusive(
        &mut self,
        x0: i64,
        y0: i64,
        x1: i64,
        y1: i64,
        value: f32,
    ) -> Result<(), ScenarioError> {
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.set_temperature(x, y, value)?;
            }
        }
        Ok(())
    }
}

fn scaled_interior_range(reference_start: u32, reference_end: u32, extent: u32) -> Range<usize> {
    debug_assert!(reference_start < reference_end);
    let extent_u64 = u64::from(extent);
    let mut start = u64::from(reference_start) * extent_u64 / DESIGN_EXTENT;
    let mut end = (u64::from(reference_end) * extent_u64).div_ceil(DESIGN_EXTENT);
    start = start.clamp(1, extent_u64 - 2);
    end = end.clamp(start + 1, extent_u64 - 1);
    start as usize..end as usize
}

fn build_sand_fall(builder: &mut FixtureBuilder) -> Result<(), ScenarioError> {
    builder.fill_material_ref(12, 244, 228, 236, MATERIAL_STONE)?;
    builder.fill_material_ref(12, 18, 92, 236, MATERIAL_STONE)?;
    builder.fill_material_ref(238, 244, 92, 236, MATERIAL_STONE)?;
    builder.fill_material_ref(40, 104, 150, 156, MATERIAL_STONE)?;
    builder.fill_material_ref(152, 216, 174, 180, MATERIAL_STONE)?;
    builder.fill_material_ref(24, 72, 18, 88, MATERIAL_SAND)?;
    builder.fill_material_ref(92, 152, 30, 118, MATERIAL_SAND)?;
    builder.fill_material_ref(176, 232, 14, 104, MATERIAL_SAND)?;
    builder.fill_material_ref(70, 86, 116, 148, MATERIAL_SAND)?;
    builder.fill_material_ref(130, 146, 126, 170, MATERIAL_SAND)?;
    Ok(())
}

fn build_water_flow(builder: &mut FixtureBuilder) -> Result<(), ScenarioError> {
    builder.fill_material_ref(10, 246, 230, 238, MATERIAL_STONE)?;
    builder.fill_material_ref(10, 18, 90, 238, MATERIAL_STONE)?;
    builder.fill_material_ref(238, 246, 90, 238, MATERIAL_STONE)?;
    builder.fill_material_ref(18, 112, 22, 112, MATERIAL_WATER)?;
    builder.fill_material_ref(144, 238, 34, 130, MATERIAL_WATER)?;
    // A lighter Oil pocket embedded above the right Water reservoir drives
    // ordinary density displacement without any scenario-specific rule.
    builder.fill_material_ref(164, 220, 72, 112, MATERIAL_OIL)?;
    builder.fill_material_ref(72, 164, 154, 160, MATERIAL_STONE)?;
    builder.fill_material_ref(18, 74, 188, 194, MATERIAL_STONE)?;
    builder.fill_material_ref(182, 238, 194, 200, MATERIAL_STONE)?;
    builder.fill_material_ref(112, 124, 110, 202, MATERIAL_STONE)?;
    builder.fill_material_ref(124, 136, 188, 230, MATERIAL_STONE)?;
    Ok(())
}

fn build_fire_heat(builder: &mut FixtureBuilder) -> Result<(), ScenarioError> {
    builder.fill_material_ref(12, 244, 222, 232, MATERIAL_STONE)?;
    builder.fill_material_ref(24, 222, 154, 214, MATERIAL_WOOD)?;
    builder.fill_material_ref(32, 78, 205, 222, MATERIAL_OIL)?;
    builder.fill_material_ref(180, 226, 204, 222, MATERIAL_OIL)?;
    builder.fill_material_ref(14, 26, 144, 222, MATERIAL_STONE)?;
    builder.fill_temperature_ref(14, 26, 144, 222, 260.0)?;
    builder.fill_temperature_ref(24, 42, 168, 202, 500.0)?;
    builder.fill_flags_ref(24, 42, 168, 202, FLAG_COMBUSTING)?;
    builder.fill_temperature_ref(32, 48, 205, 222, 180.0)?;
    builder.fill_flags_ref(32, 48, 205, 222, FLAG_COMBUSTING)?;
    builder.fill_material_ref(88, 168, 90, 118, MATERIAL_ICE)?;
    builder.fill_material_ref(96, 160, 120, 144, MATERIAL_WATER)?;
    builder.fill_temperature_ref(88, 168, 90, 118, -20.0)?;
    builder.fill_temperature_ref(96, 160, 120, 144, -20.0)?;
    Ok(())
}

fn build_pressure_burst(builder: &mut FixtureBuilder) -> Result<(), ScenarioError> {
    builder.fill_material_ref(32, 224, 38, 224, MATERIAL_STONE)?;
    builder.fill_material_ref(40, 216, 46, 216, MATERIAL_WATER)?;
    builder.fill_material_ref(52, 204, 58, 132, MATERIAL_STEAM)?;
    builder.fill_temperature_ref(40, 216, 46, 216, 110.0)?;
    builder.fill_pressure_ref(40, 216, 46, 216, 180.0)?;
    builder.fill_pressure_ref(112, 144, 80, 190, 20.0)?;
    builder.fill_material_ref(104, 152, 38, 46, MATERIAL_WOOD)?;
    builder.fill_temperature_ref(104, 152, 38, 46, 95.0)?;
    builder.fill_material_ref(116, 140, 216, 224, MATERIAL_WOOD)?;
    builder.fill_material_ref(24, 32, 116, 148, MATERIAL_STONE)?;
    builder.fill_material_ref(224, 232, 116, 148, MATERIAL_STONE)?;
    Ok(())
}

fn build_heavy_mixed_world(builder: &mut FixtureBuilder) -> Result<(), ScenarioError> {
    builder.fill_material_ref(8, 248, 232, 240, MATERIAL_STONE)?;

    builder.fill_material_ref(16, 72, 18, 104, MATERIAL_SAND)?;
    builder.fill_material_ref(80, 136, 42, 124, MATERIAL_WATER)?;
    builder.fill_material_ref(140, 196, 42, 124, MATERIAL_OIL)?;
    builder.fill_material_ref(76, 200, 188, 196, MATERIAL_STONE)?;

    builder.fill_material_ref(20, 112, 148, 190, MATERIAL_WOOD)?;
    builder.fill_temperature_ref(20, 36, 160, 188, 500.0)?;
    builder.fill_flags_ref(20, 36, 160, 188, FLAG_COMBUSTING)?;
    builder.fill_material_ref(44, 88, 126, 146, MATERIAL_SMOKE)?;

    builder.fill_material_ref(126, 226, 140, 224, MATERIAL_STONE)?;
    builder.fill_material_ref(134, 218, 148, 216, MATERIAL_WATER)?;
    builder.fill_temperature_ref(134, 218, 148, 216, 120.0)?;
    builder.fill_pressure_ref(134, 218, 148, 216, 140.0)?;
    builder.fill_material_ref(162, 190, 140, 148, MATERIAL_WOOD)?;

    builder.fill_material_ref(202, 238, 34, 96, MATERIAL_ICE)?;
    builder.fill_temperature_ref(202, 238, 34, 96, -25.0)?;
    builder.fill_material_ref(202, 238, 98, 126, MATERIAL_WATER)?;
    builder.fill_temperature_ref(202, 238, 98, 126, 10.0)?;
    Ok(())
}

/// Reproduces the frozen G7 Activity Observatory geometry and the exact
/// chunk edit-wake state created by its historical `write_*` staging hooks.
fn build_active_sleep_g7(builder: &mut FixtureBuilder) -> Result<(), ScenarioError> {
    for y in 1..=254 {
        builder.set_material(127, y, MATERIAL_BOUNDARY_BLOCK)?;
        builder.set_material(128, y, MATERIAL_BOUNDARY_BLOCK)?;
    }
    for x in 1..=254 {
        builder.set_material(x, 127, MATERIAL_BOUNDARY_BLOCK)?;
        builder.set_material(x, 128, MATERIAL_BOUNDARY_BLOCK)?;
    }

    builder.fill_material_inclusive(30, 40, 91, 105, MATERIAL_STONE)?;
    builder.fill_material_inclusive(32, 42, 89, 103, MATERIAL_WATER)?;
    builder.fill_material_inclusive(94, 44, 95, 121, MATERIAL_STONE)?;
    builder.fill_material_inclusive(108, 44, 109, 121, MATERIAL_STONE)?;
    builder.fill_material_inclusive(94, 119, 109, 121, MATERIAL_STONE)?;
    builder.fill_material_inclusive(100, 70, 103, 80, MATERIAL_WATER)?;

    builder.fill_material_inclusive(140, 40, 231, 92, MATERIAL_STONE)?;
    builder.fill_material_inclusive(143, 43, 228, 88, MATERIAL_STEAM)?;
    builder.fill_temperature_inclusive(140, 40, 231, 92, 80.0)?;

    builder.fill_material_inclusive(96, 245, 110, 247, MATERIAL_STONE)?;
    builder.fill_material_inclusive(100, 150, 106, 165, MATERIAL_SAND)?;

    builder.fill_material_inclusive(140, 174, 149, 179, MATERIAL_STONE)?;
    builder.fill_temperature_inclusive(140, 174, 149, 179, 200.0)?;
    builder.fill_material_inclusive(150, 175, 200, 178, MATERIAL_WOOD)?;
    builder.fill_material_inclusive(210, 231, 245, 236, MATERIAL_STONE)?;
    builder.fill_temperature_inclusive(210, 231, 245, 236, 200.0)?;
    builder.fill_material_inclusive(214, 229, 240, 230, MATERIAL_STONE)?;
    builder.fill_material_inclusive(214, 210, 240, 211, MATERIAL_STONE)?;
    builder.fill_material_inclusive(226, 210, 229, 211, MATERIAL_EMPTY)?;
    builder.fill_material_inclusive(214, 212, 215, 228, MATERIAL_STONE)?;
    builder.fill_material_inclusive(239, 212, 240, 228, MATERIAL_STONE)?;
    builder.fill_material_inclusive(217, 216, 238, 228, MATERIAL_WATER)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_exact(left: &ScenarioFixture, right: &ScenarioFixture) {
        assert_eq!(left.scenario, right.scenario);
        assert_eq!(left.config, right.config);
        assert_eq!(left.materials, right.materials);
        assert_eq!(left.flags, right.flags);
        assert_eq!(left.chunk_edit_wake, right.chunk_edit_wake);
        assert_eq!(
            left.temperatures
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            right
                .temperatures
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            left.pressures
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            right
                .pressures
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
    }

    fn cell(fixture: &ScenarioFixture, x: usize, y: usize) -> usize {
        y * fixture.config.width as usize + x
    }

    #[test]
    fn metadata_and_parsing_are_stable() {
        let expected = [
            (1, "sand-fall", "Sand Fall"),
            (2, "water-flow", "Water Flow"),
            (3, "fire-heat", "Fire / Heat"),
            (4, "pressure-burst", "Pressure Burst"),
            (5, "heavy-mixed-world", "Heavy Mixed World"),
            (6, "active-sleep-g7", "G7 Active / Sleep"),
        ];
        assert_eq!(OFFICIAL_G8B_SCENARIOS.len(), 5);
        assert_eq!(GALLERY_SCENARIOS.len(), 6);
        for (scenario, (number, slug, name)) in GALLERY_SCENARIOS.iter().zip(expected) {
            assert_eq!(scenario.number(), number);
            assert_eq!(scenario.slug(), slug);
            assert_eq!(scenario.name(), name);
            assert!(!scenario.description().is_empty());
            assert_eq!(slug.parse::<ScenarioId>().unwrap(), *scenario);
            assert_eq!(
                slug.to_ascii_uppercase().parse::<ScenarioId>().unwrap(),
                *scenario
            );
        }
        assert!(OFFICIAL_G8B_SCENARIOS
            .iter()
            .all(|scenario| scenario.is_official_g8b()));
        assert!(!ScenarioId::ActiveSleepG7.is_official_g8b());
        assert!("calibration".parse::<ScenarioId>().is_err());
    }

    #[test]
    fn all_six_fixtures_are_deterministic_and_internally_valid() {
        let config = WorldConfig::new(256, 256, 64).unwrap();
        for scenario in GALLERY_SCENARIOS {
            let first = ScenarioFixture::build(scenario, config).unwrap();
            let second = ScenarioFixture::build(scenario, config).unwrap();
            first.validate().unwrap();
            assert_exact(&first, &second);
            assert_eq!(first.materials.len(), 256 * 256);
            assert_eq!(first.chunk_edit_wake.len(), 16);
            if scenario == ScenarioId::ActiveSleepG7 {
                assert!(first.chunk_edit_wake.contains(&1));
            } else {
                assert!(first.chunk_edit_wake.iter().all(|value| *value == 0));
            }
        }
    }

    #[test]
    fn official_fixtures_support_rectangular_worlds_without_touching_boundary() {
        let config = WorldConfig::new(321, 257, 64).unwrap();
        for scenario in OFFICIAL_G8B_SCENARIOS {
            let fixture = ScenarioFixture::build(scenario, config).unwrap();
            fixture.validate().unwrap();
            for x in 0..config.width as usize {
                assert_eq!(
                    fixture.materials[cell(&fixture, x, 0)],
                    MATERIAL_BOUNDARY_BLOCK
                );
                assert_eq!(
                    fixture.materials[cell(&fixture, x, config.height as usize - 1)],
                    MATERIAL_BOUNDARY_BLOCK
                );
            }
        }
    }

    #[test]
    fn active_sleep_g7_preserves_frozen_geometry_and_edit_wakes() {
        let fixture = ScenarioFixture::build(
            ScenarioId::ActiveSleepG7,
            WorldConfig::new(256, 256, 64).unwrap(),
        )
        .unwrap();
        assert_eq!(
            fixture.materials[cell(&fixture, 127, 10)],
            MATERIAL_BOUNDARY_BLOCK
        );
        assert_eq!(fixture.materials[cell(&fixture, 32, 42)], MATERIAL_WATER);
        assert_eq!(fixture.materials[cell(&fixture, 143, 43)], MATERIAL_STEAM);
        assert_eq!(
            fixture.temperatures[cell(&fixture, 143, 43)].to_bits(),
            80.0f32.to_bits()
        );
        assert_eq!(fixture.materials[cell(&fixture, 100, 150)], MATERIAL_SAND);
        assert_eq!(fixture.materials[cell(&fixture, 150, 175)], MATERIAL_WOOD);
        assert_eq!(fixture.materials[cell(&fixture, 226, 210)], MATERIAL_EMPTY);
        assert_eq!(fixture.materials[cell(&fixture, 217, 216)], MATERIAL_WATER);

        let mut expected_wakes = vec![1u32; 16];
        expected_wakes[12] = 0; // untouched bottom-left corner chunk (cx=0, cy=3)
        assert_eq!(&*fixture.chunk_edit_wake, expected_wakes);
    }

    #[test]
    fn incompatible_dimensions_fail_before_allocation_or_staging() {
        let too_small = WorldConfig::new(255, 256, 64).unwrap();
        for scenario in OFFICIAL_G8B_SCENARIOS {
            assert!(matches!(
                ScenarioFixture::build(scenario, too_small),
                Err(ScenarioError::TooSmall { .. })
            ));
        }
        for config in [
            WorldConfig::new(257, 256, 64).unwrap(),
            WorldConfig::new(256, 256, 32).unwrap(),
        ] {
            assert!(matches!(
                ScenarioFixture::build(ScenarioId::ActiveSleepG7, config),
                Err(ScenarioError::ActiveSleepConfig { .. })
            ));
        }
    }

    #[test]
    fn water_flow_256_pins_finite_authored_geometry_and_observation_mask() {
        let config = WorldConfig::new(256, 256, 64).unwrap();
        let fixture = ScenarioFixture::build(ScenarioId::WaterFlow, config).unwrap();

        // Reconstruct the authored half-open rectangles independently of
        // `build_water_flow`. This freezes the untuned Scenario 2 candidate
        // geometry before the first Harness run.
        let mut expected = initial_material_ids(&config).unwrap();
        {
            let mut paint = |x_range: Range<usize>, y_range: Range<usize>, material: u32| {
                for y in y_range {
                    for x in x_range.clone() {
                        expected[y * config.width as usize + x] = material;
                    }
                }
            };
            paint(10..246, 230..238, MATERIAL_STONE);
            paint(10..18, 90..238, MATERIAL_STONE);
            paint(238..246, 90..238, MATERIAL_STONE);
            paint(18..112, 22..112, MATERIAL_WATER);
            paint(144..238, 34..130, MATERIAL_WATER);
            paint(164..220, 72..112, MATERIAL_OIL);
            paint(72..164, 154..160, MATERIAL_STONE);
            paint(18..74, 188..194, MATERIAL_STONE);
            paint(182..238, 194..200, MATERIAL_STONE);
            paint(112..124, 110..202, MATERIAL_STONE);
            paint(124..136, 188..230, MATERIAL_STONE);
        }
        assert_eq!(fixture.materials(), expected.as_slice());

        let count = |material| {
            fixture
                .materials()
                .iter()
                .filter(|&&value| value == material)
                .count()
        };
        assert_eq!(count(MATERIAL_WATER), 15_244);
        assert_eq!(count(MATERIAL_OIL), 2_240);
        assert_eq!(count(MATERIAL_STONE), 6_888);
        assert_eq!(count(MATERIAL_BOUNDARY_BLOCK), 1_020);
        assert_eq!(count(MATERIAL_EMPTY), 40_144);

        assert!(fixture
            .temperatures()
            .iter()
            .all(|value| value.to_bits() == TEMPERATURE_REFERENCE.to_bits()));
        assert!(fixture
            .pressures()
            .iter()
            .all(|value| value.to_bits() == PRESSURE_REFERENCE.to_bits()));
        assert!(fixture.flags().iter().all(|value| *value == 0));
        assert!(fixture.chunk_edit_wake().iter().all(|value| *value == 0));

        // Harness observation contract: the destination is the tick-0 EMPTY
        // mask inside the lower basin bounds. It is observation-only and does
        // not add a source, target material, or scripted scenario result.
        let mut destination_mask_cells = 0usize;
        let mut destination_water_cells = 0usize;
        let mut destination_oil_cells = 0usize;
        for y in 200..230 {
            for x in 18..238 {
                let material = fixture.materials()[cell(&fixture, x, y)];
                if material == MATERIAL_EMPTY {
                    destination_mask_cells += 1;
                }
                if material == MATERIAL_WATER {
                    destination_water_cells += 1;
                }
                if material == MATERIAL_OIL {
                    destination_oil_cells += 1;
                }
            }
        }
        assert_eq!(destination_mask_cells, 6_216);
        assert_eq!(destination_water_cells, 0);
        assert_eq!(destination_oil_cells, 0);

        // The complete bottom chunk row is Water-free at tick 0. Observing
        // Water there later is therefore a fixture-derived cross-chunk signal.
        let mut bottom_chunk_row_water = 0usize;
        for y in 192..256 {
            for x in 0..256 {
                if fixture.materials()[cell(&fixture, x, y)] == MATERIAL_WATER {
                    bottom_chunk_row_water += 1;
                }
            }
        }
        assert_eq!(bottom_chunk_row_water, 0);
    }

    #[test]
    fn scenario_payloads_exercise_their_named_subsystems() {
        let config = WorldConfig::new(256, 256, 64).unwrap();
        let sand = ScenarioFixture::build(ScenarioId::SandFall, config).unwrap();
        assert!(sand.materials.contains(&MATERIAL_SAND));

        let water = ScenarioFixture::build(ScenarioId::WaterFlow, config).unwrap();
        assert!(water.materials.contains(&MATERIAL_WATER));
        assert!(water.materials.contains(&MATERIAL_OIL));

        let fire = ScenarioFixture::build(ScenarioId::FireHeat, config).unwrap();
        assert!(fire.flags.contains(&FLAG_COMBUSTING));
        assert!(fire.temperatures.iter().any(|value| *value >= 500.0));

        let pressure = ScenarioFixture::build(ScenarioId::PressureBurst, config).unwrap();
        assert!(pressure.pressures.iter().any(|value| *value >= 180.0));
        assert!(pressure.materials.contains(&MATERIAL_WOOD));

        let heavy = ScenarioFixture::build(ScenarioId::HeavyMixedWorld, config).unwrap();
        for material in [
            MATERIAL_SAND,
            MATERIAL_WATER,
            MATERIAL_OIL,
            MATERIAL_WOOD,
            MATERIAL_SMOKE,
            MATERIAL_ICE,
        ] {
            assert!(heavy.materials.contains(&material));
        }
    }
}
