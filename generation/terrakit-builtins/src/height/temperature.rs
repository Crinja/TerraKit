//! Temperature map stage implementation.

use terrakit_algorithms::noise::{Noise2D, NoiseAlgorithm};
use terrakit_core::{HeightField, SamplingDomain, SeedDomain, TerrainResource, Vector3F64};
use terrakit_pipeline::{
    ResourceAccess, ResourceKey, ResourceKind, ResourceProduct, ResourceRequirement, ResourceSet,
    StageContext, StageError, StageId, TerrainStage,
};

const STAGE_NAME: &str = "biome_temperature";

/// Terrain stage that generates a temperature map for biome classification.
///
/// Uses noise to create a smooth temperature gradient across the world.
/// Higher values represent warmer regions, lower values represent colder regions.
/// An elevation map is accepted as input: higher elevation reduces temperature
/// proportionally to the configured `elevation_influence`.
#[derive(Debug, Clone)]
pub struct TemperatureMapStage {
    stage_id: StageId,
    elevation: ResourceKey,
    output: ResourceKey,
    scale: f64,
    domain: SeedDomain,
    elevation_influence: f32,
    requirements: [ResourceRequirement; 1],
    products: [ResourceProduct; 1],
}

impl TemperatureMapStage {
    /// Creates a temperature map stage.
    ///
    /// `elevation` is the height field resource key to read elevation from.
    /// `elevation_influence` controls how much elevation cools temperature (0.0–1.0).
    pub fn new(
        stage_id: StageId,
        elevation: ResourceKey,
        output: ResourceKey,
        scale: f64,
        domain: SeedDomain,
        elevation_influence: f32,
    ) -> Result<Self, StageError> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(StageError::new(
                "temperature scale must be finite and greater than zero",
            ));
        }

        if !elevation_influence.is_finite() || !(0.0..=1.0).contains(&elevation_influence) {
            return Err(StageError::new(
                "temperature elevation influence must be finite and between 0.0 and 1.0",
            ));
        }

        Ok(Self {
            stage_id,
            elevation,
            output,
            scale,
            domain,
            elevation_influence,
            requirements: [ResourceRequirement::required(
                elevation,
                ResourceKind::HeightField,
                ResourceAccess::Read,
            )],
            products: [ResourceProduct::create(output, ResourceKind::HeightField)],
        })
    }

    /// Returns the elevation resource key this stage reads.
    pub const fn elevation(&self) -> ResourceKey {
        self.elevation
    }

    /// Returns the resource key this stage creates.
    pub const fn output(&self) -> ResourceKey {
        self.output
    }

    /// Returns the configured scale.
    pub const fn scale(&self) -> f64 {
        self.scale
    }

    /// Returns the domain used to derive this stage's sampler seed.
    pub const fn domain(&self) -> SeedDomain {
        self.domain
    }

    /// Returns how much elevation reduces temperature.
    pub const fn elevation_influence(&self) -> f32 {
        self.elevation_influence
    }
}

impl TerrainStage for TemperatureMapStage {
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
        let region = context
            .region()
            .as_region2()
            .ok_or_else(|| StageError::new("temperature stage requires a 2D generation region"))?;

        if resources.contains(self.output) {
            return Err(StageError::new(format!(
                "cannot create temperature map at {:?}: resource already exists",
                self.output
            )));
        }

        let Some(elevation_field) = resources.height_field(self.elevation) else {
            return Err(StageError::new(format!(
                "temperature stage elevation input {:?} is missing or is not a height field",
                self.elevation
            )));
        };
        let elevation_field = elevation_field.clone();

        let extent = region.point_sample_extent();

        let derived_seed = context.derive_seed(self.domain);
        let noise = NoiseAlgorithm::Simplex.create_2d(derived_seed);

        let mut values = Vec::with_capacity((extent.width() * extent.height()) as usize);

        let transform = region.transform();

        for y in 0..extent.height() {
            for x in 0..extent.width() {
                let world_position = transform.position_at(f64::from(x), f64::from(y));

                let sample_x = world_position.x * self.scale;
                let sample_y = world_position.z * self.scale;

                let noise_temperature =
                    ((Noise2D::sample(&noise, sample_x, sample_y) as f64 + 1.0) * 0.5) as f32;

                // Higher elevation cools the temperature.
                let elevation = elevation_field.get(x as usize, y as usize).unwrap_or(0.0).clamp(0.0, 1.0);
                let temperature =
                    (noise_temperature - elevation * self.elevation_influence).clamp(0.0, 1.0);

                values.push(temperature);
            }
        }

        let height_field = HeightField::from_values(
            extent,
            values,
            region.transform(),
            SamplingDomain::Points,
            Vector3F64::Y,
        )
        .map_err(|error| StageError::new(format!("failed to create temperature map: {error}")))?;

        resources.insert(self.output, TerrainResource::HeightField(height_field));

        Ok(())
    }
}
