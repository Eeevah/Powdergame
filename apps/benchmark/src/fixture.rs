//! Dimension-safe calibration fixture construction and GPU staging.

use powdergame_core::{
    initial_material_ids, WorldConfig, FLAG_COMBUSTING, MATERIAL_OIL, MATERIAL_SAND,
    MATERIAL_STONE, MATERIAL_WATER, MATERIAL_WOOD,
};
use powdergame_gpu::Simulation;

const REFERENCE_EXTENT: u64 = 2048;
const MIN_FIXTURE_EXTENT: u32 = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureError {
    TooSmall {
        width: u32,
        height: u32,
        minimum: u32,
    },
    InvalidWorld(String),
    CoordinateOutOfBounds {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    },
}

impl std::fmt::Display for FixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooSmall {
                width,
                height,
                minimum,
            } => write!(
                f,
                "calibration fixture requires width and height >= {minimum}, got {width}x{height}"
            ),
            Self::InvalidWorld(message) => write!(f, "invalid calibration world: {message}"),
            Self::CoordinateOutOfBounds {
                x,
                y,
                width,
                height,
            } => write!(
                f,
                "calibration fixture coordinate ({x}, {y}) is outside {width}x{height}"
            ),
        }
    }
}

impl std::error::Error for FixtureError {}

/// CPU staging data for the repeatable mixed calibration world.
#[derive(Debug)]
pub struct CalibrationFixture {
    pub materials: Vec<u32>,
    pub temperatures: Vec<f32>,
    pub flags: Vec<u32>,
    width: usize,
    height: usize,
}

impl CalibrationFixture {
    fn index(&self, x: usize, y: usize) -> Result<usize, FixtureError> {
        if x >= self.width || y >= self.height {
            return Err(FixtureError::CoordinateOutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }
        y.checked_mul(self.width)
            .and_then(|row| row.checked_add(x))
            .ok_or_else(|| FixtureError::InvalidWorld("fixture index overflow".into()))
    }

    fn set(
        &mut self,
        x: usize,
        y: usize,
        material: u32,
        temperature: Option<f32>,
        flags: Option<u32>,
    ) -> Result<(), FixtureError> {
        let index = self.index(x, y)?;
        self.materials[index] = material;
        if let Some(value) = temperature {
            self.temperatures[index] = value;
        }
        if let Some(value) = flags {
            self.flags[index] = value;
        }
        Ok(())
    }

    fn fill_rect(
        &mut self,
        x_range: std::ops::Range<usize>,
        y_range: std::ops::Range<usize>,
        material: u32,
        temperature: Option<f32>,
        flags: Option<u32>,
    ) -> Result<(), FixtureError> {
        for y in y_range {
            for x in x_range.clone() {
                self.set(x, y, material, temperature, flags)?;
            }
        }
        Ok(())
    }
}

fn scaled_interior_cell(reference: u32, extent: u32) -> usize {
    let scaled = (u64::from(reference) * u64::from(extent) / REFERENCE_EXTENT) as usize;
    scaled.clamp(1, extent as usize - 2)
}

fn scaled_interior_range(
    reference_start: u32,
    reference_end: u32,
    extent: u32,
) -> std::ops::Range<usize> {
    let extent_u64 = u64::from(extent);
    let mut start = (u64::from(reference_start) * extent_u64 / REFERENCE_EXTENT) as usize;
    let mut end = (u64::from(reference_end) * extent_u64).div_ceil(REFERENCE_EXTENT) as usize;
    start = start.clamp(1, extent as usize - 2);
    end = end.clamp(start + 1, extent as usize - 1);
    start..end
}

/// Builds the calibration fixture without touching the GPU.
///
/// Coordinates are defined in the original 2048x2048 reference space and
/// independently scaled on each axis. The reference configuration therefore
/// remains byte-for-byte positioned as before, while smaller valid worlds do
/// not alias rows or index outside their dense arrays.
pub fn build_calibration_fixture(config: &WorldConfig) -> Result<CalibrationFixture, FixtureError> {
    validate_calibration_fixture_config(config)?;

    let width = config.width as usize;
    let height = config.height as usize;
    let cell_count = width
        .checked_mul(height)
        .ok_or_else(|| FixtureError::InvalidWorld("fixture cell count overflow".into()))?;
    let mut fixture = CalibrationFixture {
        materials: initial_material_ids(config)
            .map_err(|error| FixtureError::InvalidWorld(error.to_string()))?,
        temperatures: vec![0.0; cell_count],
        flags: vec![0; cell_count],
        width,
        height,
    };

    // Falling Sand streams.
    for center_x in (100u32..400).step_by(50) {
        fixture.fill_rect(
            scaled_interior_range(center_x - 10, center_x + 10, config.width),
            scaled_interior_range(100, 500, config.height),
            MATERIAL_SAND,
            None,
            None,
        )?;
    }

    // Water and Oil tanks.
    fixture.fill_rect(
        scaled_interior_range(100, 400, config.width),
        scaled_interior_range(800, 1000, config.height),
        MATERIAL_WATER,
        None,
        None,
    )?;
    fixture.fill_rect(
        scaled_interior_range(500, 800, config.width),
        scaled_interior_range(800, 1000, config.height),
        MATERIAL_OIL,
        None,
        None,
    )?;

    // Boiling Water with Wood relief walls.
    fixture.fill_rect(
        scaled_interior_range(200, 400, config.width),
        scaled_interior_range(1200, 1400, config.height),
        MATERIAL_WATER,
        Some(120.0),
        None,
    )?;
    let boiler_left = scaled_interior_cell(190, config.width);
    let boiler_right = scaled_interior_cell(410, config.width);
    let boiler_top = scaled_interior_cell(1190, config.height);
    let boiler_bottom = scaled_interior_cell(1410, config.height);
    for x in scaled_interior_range(190, 410, config.width) {
        fixture.set(x, boiler_top, MATERIAL_WOOD, None, None)?;
        fixture.set(x, boiler_bottom, MATERIAL_WOOD, None, None)?;
    }
    for y in scaled_interior_range(1190, 1410, config.height) {
        fixture.set(boiler_left, y, MATERIAL_WOOD, None, None)?;
        fixture.set(boiler_right, y, MATERIAL_WOOD, None, None)?;
    }

    // Burning Wood line. Iterate each scaled output cell exactly once so that
    // small fixtures cannot alias a burning reference cell with later writes.
    // At 2048 this preserves the original x=1000..1600, every-tenth-x pattern.
    let fire_x = scaled_interior_range(1000, 1600, config.width);
    let fire_y = scaled_interior_range(300, 320, config.height);
    let fire_start = fire_x.start;
    for x in fire_x {
        for y in fire_y.clone() {
            let burning = (x - fire_start).is_multiple_of(10);
            fixture.set(
                x,
                y,
                MATERIAL_WOOD,
                burning.then_some(500.0),
                burning.then_some(FLAG_COMBUSTING),
            )?;
        }
    }

    // Stable bulk Water in a deep Stone basin.
    fixture.fill_rect(
        scaled_interior_range(1000, 1900, config.width),
        scaled_interior_range(1500, 1900, config.height),
        MATERIAL_WATER,
        Some(20.0),
        None,
    )?;
    let basin_left = scaled_interior_cell(990, config.width);
    let basin_right = scaled_interior_cell(1910, config.width);
    let basin_bottom = scaled_interior_cell(1900, config.height);
    for x in scaled_interior_range(990, 1910, config.width) {
        fixture.set(x, basin_bottom, MATERIAL_STONE, None, None)?;
    }
    for y in scaled_interior_range(1500, 1901, config.height) {
        fixture.set(basin_left, y, MATERIAL_STONE, None, None)?;
        fixture.set(basin_right, y, MATERIAL_STONE, None, None)?;
    }

    Ok(fixture)
}

/// Validates the pure fixture contract without allocating the dense staging arrays.
pub fn validate_calibration_fixture_config(config: &WorldConfig) -> Result<(), FixtureError> {
    config
        .validate()
        .map_err(|error| FixtureError::InvalidWorld(error.to_string()))?;
    if config.width < MIN_FIXTURE_EXTENT || config.height < MIN_FIXTURE_EXTENT {
        return Err(FixtureError::TooSmall {
            width: config.width,
            height: config.height,
            minimum: MIN_FIXTURE_EXTENT,
        });
    }
    Ok(())
}

pub fn stage_calibration_fixture(sim: &mut Simulation) -> Result<(), FixtureError> {
    let fixture = build_calibration_fixture(&sim.world.config)?;

    let mut material_bytes = Vec::with_capacity(fixture.materials.len() * 4);
    for material in &fixture.materials {
        material_bytes.extend_from_slice(&material.to_ne_bytes());
    }
    let mut temperature_bytes = Vec::with_capacity(fixture.temperatures.len() * 4);
    for temperature in &fixture.temperatures {
        temperature_bytes.extend_from_slice(&temperature.to_ne_bytes());
    }
    let mut flag_bytes = Vec::with_capacity(fixture.flags.len() * 4);
    for flags in &fixture.flags {
        flag_bytes.extend_from_slice(&flags.to_ne_bytes());
    }

    let queue = &sim.context.queue;
    queue.write_buffer(&sim.world.material_current, 0, &material_bytes);
    queue.write_buffer(&sim.world.material_next, 0, &material_bytes);
    queue.write_buffer(&sim.world.temperature_current, 0, &temperature_bytes);
    queue.write_buffer(&sim.world.temperature_next, 0, &temperature_bytes);
    queue.write_buffer(&sim.world.flags_current, 0, &flag_bytes);
    queue.write_buffer(&sim.world.flags_next, 0, &flag_bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use powdergame_core::{MATERIAL_BOUNDARY_BLOCK, MATERIAL_EMPTY};

    fn cell(fixture: &CalibrationFixture, x: usize, y: usize) -> usize {
        fixture.index(x, y).unwrap()
    }

    #[test]
    fn reference_fixture_preserves_original_coordinates_and_boundary() {
        let config = WorldConfig::reference();
        let fixture = build_calibration_fixture(&config).unwrap();

        assert_eq!(fixture.materials[cell(&fixture, 95, 100)], MATERIAL_SAND);
        assert_eq!(fixture.materials[cell(&fixture, 100, 800)], MATERIAL_WATER);
        assert_eq!(fixture.materials[cell(&fixture, 500, 800)], MATERIAL_OIL);
        let boiler = cell(&fixture, 200, 1200);
        assert_eq!(fixture.materials[boiler], MATERIAL_WATER);
        assert_eq!(fixture.temperatures[boiler], 120.0);
        let fire = cell(&fixture, 1000, 300);
        assert_eq!(fixture.materials[fire], MATERIAL_WOOD);
        assert_eq!(fixture.flags[fire], FLAG_COMBUSTING);
        assert_eq!(fixture.temperatures[fire], 500.0);
        let unlit_fire = cell(&fixture, 1001, 300);
        assert_eq!(fixture.materials[unlit_fire], MATERIAL_WOOD);
        assert_eq!(fixture.flags[unlit_fire], 0);
        assert_eq!(fixture.temperatures[unlit_fire], 0.0);
        assert_eq!(
            fixture.materials[cell(&fixture, 1000, 1500)],
            MATERIAL_WATER
        );
        assert_eq!(fixture.materials[cell(&fixture, 990, 1900)], MATERIAL_STONE);

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

    #[test]
    fn smaller_fixture_is_rich_and_dimension_safe() {
        let config = WorldConfig::new(256, 256, 64).unwrap();
        let fixture = build_calibration_fixture(&config).unwrap();
        assert_eq!(fixture.materials.len(), 256 * 256);
        for material in [
            MATERIAL_SAND,
            MATERIAL_WATER,
            MATERIAL_OIL,
            MATERIAL_WOOD,
            MATERIAL_STONE,
        ] {
            assert!(fixture.materials.contains(&material));
        }
        assert!(fixture.flags.contains(&FLAG_COMBUSTING));
        assert!(fixture.temperatures.contains(&120.0));
        assert!(fixture.temperatures.contains(&20.0));
        assert!(fixture.materials.contains(&MATERIAL_EMPTY));
    }

    #[test]
    fn undersized_fixture_returns_an_actionable_error() {
        let config = WorldConfig::new(63, 256, 64).unwrap();
        let error = build_calibration_fixture(&config).unwrap_err().to_string();
        assert!(error.contains("63x256"));
        assert!(error.contains(">= 64"));
    }

    #[test]
    fn minimum_fixture_fire_pattern_has_no_aliased_stale_ignition() {
        let config = WorldConfig::new(64, 64, 64).unwrap();
        let fixture = build_calibration_fixture(&config).unwrap();
        let fire_x = scaled_interior_range(1000, 1600, config.width);
        let fire_y = scaled_interior_range(300, 320, config.height);
        let fire_start = fire_x.start;
        let mut burning_cells = 0usize;
        let mut unlit_cells = 0usize;

        for x in fire_x {
            for y in fire_y.clone() {
                let index = cell(&fixture, x, y);
                assert_eq!(fixture.materials[index], MATERIAL_WOOD);
                if (x - fire_start).is_multiple_of(10) {
                    assert_eq!(fixture.flags[index], FLAG_COMBUSTING);
                    assert_eq!(fixture.temperatures[index], 500.0);
                    burning_cells += 1;
                } else {
                    assert_eq!(fixture.flags[index], 0);
                    assert_eq!(fixture.temperatures[index], 0.0);
                    unlit_cells += 1;
                }
            }
        }

        assert!(burning_cells > 0);
        assert!(unlit_cells > 0);
    }
}
