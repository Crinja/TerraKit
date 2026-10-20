//! Biome classifier stage implementation.

use terrakit_core::{HeightField, SamplingDomain, TerrainResource, Vector3F64};
use terrakit_pipeline::{
    ResourceAccess, ResourceKey, ResourceKind, ResourceProduct, ResourceRequirement, ResourceSet,
    StageContext, StageError, StageId, TerrainStage,
};

const STAGE_NAME: &str = "biome_classifier";

/// A single cell in the biome classification grid.
///
/// The grid is indexed by temperature (X axis) and humidity (Y axis),
/// both normalised to [0, 1]. Each cell holds a user-defined biome id.
#[derive(Debug, Clone)]
pub struct BiomeCell {
    /// User-defined biome identifier (e.g. 0 = Ocean, 1 = Desert, 2 = Forest …).
    pub biome_id: u32,
}

/// A configurable 2D biome classification grid.
///
/// Temperature is the X axis, humidity is the Y axis.
/// The grid divides both axes into `cols` × `rows` equal-width buckets.
/// Each bucket maps to a biome id supplied by the user.
#[derive(Debug, Clone)]
pub struct BiomeGrid {
    cols: usize,
    rows: usize,
    cells: Vec<BiomeCell>,
}

impl BiomeGrid {
    /// Creates a biome grid.
    ///
    /// `cells` must have exactly `cols * rows` entries, stored row-major
    /// (humidity varies along the outer index, temperature along the inner).
    pub fn new(cols: usize, rows: usize, cells: Vec<BiomeCell>) -> Result<Self, StageError> {
        if cols == 0 || rows == 0 {
            return Err(StageError::new(
                "biome grid dimensions must be greater than zero",
            ));
        }
        if cells.len() != cols * rows {
            return Err(StageError::new(format!(
                "biome grid expects {} cells ({cols}x{rows}) but got {}",
                cols * rows,
                cells.len()
            )));
        }
        Ok(Self { cols, rows, cells })
    }

    /// Returns the biome id for the given normalised temperature and humidity.
    pub fn classify(&self, temperature: f32, humidity: f32) -> u32 {
        let col = ((temperature * self.cols as f32) as usize).min(self.cols - 1);
        let row = ((humidity * self.rows as f32) as usize).min(self.rows - 1);
        self.cells[row * self.cols + col].biome_id
    }

    /// Returns the number of temperature columns.
    pub const fn cols(&self) -> usize {
        self.cols
    }

    /// Returns the number of humidity rows.
    pub const fn rows(&self) -> usize {
        self.rows
    }
}

/// Terrain stage that classifies each terrain point into a biome.
///
/// Reads a temperature map and a humidity map (both as HeightFields with
/// values in [0, 1]) and writes the biome classification as a HeightField
/// whose values are user-defined biome identifiers cast to f32.
/// Downstream stages can read the biome output and cast back to u32.
#[derive(Debug, Clone)]
pub struct BiomeClassifierStage {
    stage_id: StageId,
    temperature: ResourceKey,
    humidity: ResourceKey,
    output: ResourceKey,
    grid: BiomeGrid,
    requirements: [ResourceRequirement; 2],
    products: [ResourceProduct; 1],
}

impl BiomeClassifierStage {
    /// Creates a biome classifier stage.
    pub fn new(
        stage_id: StageId,
        temperature: ResourceKey,
        humidity: ResourceKey,
        output: ResourceKey,
        grid: BiomeGrid,
    ) -> Result<Self, StageError> {
        Ok(Self {
            stage_id,
            temperature,
            humidity,
            output,
            requirements: [
                ResourceRequirement::required(
                    temperature,
                    ResourceKind::HeightField,
                    ResourceAccess::Read,
                ),
                ResourceRequirement::required(
                    humidity,
                    ResourceKind::HeightField,
                    ResourceAccess::Read,
                ),
            ],
            products: [ResourceProduct::create(output, ResourceKind::HeightField)],
            grid,
        })
    }

    /// Returns the temperature resource key this stage reads.
    pub const fn temperature(&self) -> ResourceKey {
        self.temperature
    }

    /// Returns the humidity resource key this stage reads.
    pub const fn humidity(&self) -> ResourceKey {
        self.humidity
    }

    /// Returns the output resource key this stage creates.
    pub const fn output(&self) -> ResourceKey {
        self.output
    }

    /// Returns the configured biome grid.
    pub const fn grid(&self) -> &BiomeGrid {
        &self.grid
    }
}

impl TerrainStage for BiomeClassifierStage {
    fn id(&self) -> StageId {
        self.stage_id
    }

    fn name(&self) -> &str {
        STAGE_NAME
    }

    fn requirements(&self) -> &[ResourceRequirement] {
        &self.requirements
    }

    fn products(&self) -> &[ResourceProduct] {
        &self.products
    }

    fn execute(
        &mut self,
        context: &StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        let region = context.region().as_region2().ok_or_else(|| {
            StageError::new("biome classifier stage requires a 2D generation region")
        })?;

        if resources.contains(self.output) {
            return Err(StageError::new(format!(
                "cannot create biome field at {:?}: resource already exists",
                self.output
            )));
        }

        let Some(temperature_field) = resources.height_field(self.temperature) else {
            return Err(StageError::new(format!(
                "biome classifier temperature input {:?} is missing or is not a height field",
                self.temperature
            )));
        };
        let temperature_field = temperature_field.clone();

        let Some(humidity_field) = resources.height_field(self.humidity) else {
            return Err(StageError::new(format!(
                "biome classifier humidity input {:?} is missing or is not a height field",
                self.humidity
            )));
        };
        let humidity_field = humidity_field.clone();

        let extent = region.point_sample_extent();
        let mut values = Vec::with_capacity((extent.width() * extent.height()) as usize);

        for y in 0..extent.height() {
            for x in 0..extent.width() {
                let x = x as usize;
                let y = y as usize;
                let temperature = temperature_field.get(x, y).unwrap_or(0.0).clamp(0.0, 1.0);
                let humidity = humidity_field.get(x, y).unwrap_or(0.0).clamp(0.0, 1.0);
                // Store biome id as f32 — downstream stages cast back to u32.
                values.push(self.grid.classify(temperature, humidity) as f32);
            }
        }

        let height_field = HeightField::from_values(
            extent,
            values,
            region.transform(),
            SamplingDomain::Points,
            Vector3F64::Y,
        )
        .map_err(|error| StageError::new(format!("failed to create biome field: {error}")))?;

        resources.insert(self.output, TerrainResource::HeightField(height_field));

        Ok(())
    }
}
