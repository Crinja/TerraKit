//! Poisson-disc object scattering stage implementation.

use terrakit_algorithms::scattering::{
    PoissonSettings,
    ScatterArea,
    ScatterError,
    scatter_poisson,
};
use terrakit_core::{
    ScatterPoints,
    TerrainResource,
};
use terrakit_pipeline::{
    ResourceAccess,
    ResourceKey,
    ResourceKind,
    ResourceProduct,
    ResourceRequirement,
    ResourceSet,
    StageContext,
    StageError,
    StageId,
    TerrainStage,
};

const STAGE_NAME: &str = "poisson_scatter";

/// Terrain stage that generates object placement points using Poisson-disc
/// scattering over a height field.
///
/// The stage reads an existing height field and creates a collection of
/// engine-independent scatter points. The actual object represented by each
/// point is identified by `prototype_id`.
#[derive(Debug, Clone)]
pub struct PoissonScatterStage {
    stage_id: StageId,
    input: ResourceKey,
    output: ResourceKey,

    radius: f32,
    attempts: u32,
    prototype_id: u32,
    min_scale: f32,
    max_scale: f32,
    random_rotation: bool,

    requirements: [ResourceRequirement; 1],
    products: [ResourceProduct; 1],
}

impl PoissonScatterStage {
    /// Creates a Poisson-disc scattering stage.
    pub fn new(
        stage_id: StageId,
        input: ResourceKey,
        output: ResourceKey,
        radius: f32,
        attempts: u32,
        prototype_id: u32,
        min_scale: f32,
        max_scale: f32,
        random_rotation: bool,
    ) -> Result<Self, StageError> {
        validate_configuration(
            radius,
            attempts,
            min_scale,
            max_scale,
        )?;

        if input == output {
            return Err(StageError::new(
                "Poisson scatter input and output keys must be distinct",
            ));
        }

        Ok(Self {
            stage_id,
            input,
            output,
            radius,
            attempts,
            prototype_id,
            min_scale,
            max_scale,
            random_rotation,

            requirements: [ResourceRequirement::required(
                input,
                ResourceKind::HeightField,
                ResourceAccess::Read,
            )],

            products: [ResourceProduct::create(
                output,
                ResourceKind::ScatterPoints,
            )],
        })
    }

    pub const fn input(&self) -> ResourceKey {
        self.input
    }

    pub const fn output(&self) -> ResourceKey {
        self.output
    }

    pub const fn radius(&self) -> f32 {
        self.radius
    }

    pub const fn attempts(&self) -> u32 {
        self.attempts
    }

    pub const fn prototype_id(&self) -> u32 {
        self.prototype_id
    }

    pub const fn min_scale(&self) -> f32 {
        self.min_scale
    }

    pub const fn max_scale(&self) -> f32 {
        self.max_scale
    }

    pub const fn random_rotation(&self) -> bool {
        self.random_rotation
    }
}

impl TerrainStage for PoissonScatterStage {
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
        context
            .region()
            .as_region2()
            .ok_or_else(|| {
                StageError::new(
                    "Poisson scatter stage requires a 2D generation region",
                )
            })?;

        if resources.contains(self.output) {
            return Err(StageError::new(format!(
                "cannot create scatter points at {:?}: resource already exists",
                self.output
            )));
        }

        let Some(height_field) = resources.height_field(self.input) else {
            return Err(StageError::new(format!(
                "Poisson scatter input {:?} is missing or is not a height field",
                self.input
            )));
        };

        let area = height_field_scatter_area(height_field)?;

        /*
         * The current terrakit-algorithms Poisson implementation works in
         * world-space XZ coordinates.
         *
         * We derive a deterministic seed from the pipeline context so the
         * same pipeline seed and region produce the same result.
         */
        let seed = context_seed(context);

        let settings = PoissonSettings {
            seed,
            radius: self.radius,
            attempts: self.attempts,
            prototype_id: self.prototype_id,
            min_scale: self.min_scale,
            max_scale: self.max_scale,
            random_rotation: self.random_rotation,
        };

        let points = scatter_poisson(settings, area)
            .map_err(|error| scatter_stage_error(
                "failed to generate Poisson scatter points",
                error,
            ))?;

        let points = ScatterPoints::from_vec(points);

        resources.insert(
            self.output,
            TerrainResource::ScatterPoints(points),
        );

        Ok(())
    }
}

/// Determines the rectangular XZ world-space area covered by a height field.
fn height_field_scatter_area(
    height_field: &terrakit_core::HeightField,
) -> Result<ScatterArea, StageError> {
    if height_field.is_empty() {
        return Err(StageError::new(
            "cannot scatter objects over an empty height field",
        ));
    }

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_z = f64::INFINITY;
    let mut max_z = f64::NEG_INFINITY;

    for y in 0..height_field.height() {
        for x in 0..height_field.width() {
            let Some(position) = height_field.base_position(x, y) else {
                return Err(StageError::new(format!(
                    "missing height-field position at ({x}, {y})"
                )));
            };

            min_x = min_x.min(position.x);
            max_x = max_x.max(position.x);
            min_z = min_z.min(position.z);
            max_z = max_z.max(position.z);
        }
    }

    if !min_x.is_finite()
        || !max_x.is_finite()
        || !min_z.is_finite()
        || !max_z.is_finite()
    {
        return Err(StageError::new(
            "height field produced non-finite scattering bounds",
        ));
    }

    if max_x <= min_x || max_z <= min_z {
        return Err(StageError::new(
            "height field does not contain a valid scattering area",
        ));
    }

    Ok(ScatterArea::new(
        min_x as f32,
        max_x as f32,
        min_z as f32,
        max_z as f32,
    ))
}

/// Gets the deterministic seed used by this stage.
///
/// This is temporarily kept as a small adapter around the existing
/// StageContext API. Once the scattering algorithm accepts GenerationSeed
/// directly, this function should return that type instead of converting it.
fn context_seed(context: &StageContext) -> u64 {
    context.seed().value()
}

fn validate_configuration(
    radius: f32,
    attempts: u32,
    min_scale: f32,
    max_scale: f32,
) -> Result<(), StageError> {
    if !radius.is_finite() || radius <= 0.0 {
        return Err(StageError::new(
            "Poisson scatter radius must be finite and greater than zero",
        ));
    }

    if attempts == 0 {
        return Err(StageError::new(
            "Poisson scatter attempts must be greater than zero",
        ));
    }

    if !min_scale.is_finite() || min_scale < 0.0 {
        return Err(StageError::new(
            "Poisson scatter minimum scale must be finite and non-negative",
        ));
    }

    if !max_scale.is_finite() {
        return Err(StageError::new(
            "Poisson scatter maximum scale must be finite",
        ));
    }

    if max_scale < min_scale {
        return Err(StageError::new(
            "Poisson scatter maximum scale must be greater than or equal to minimum scale",
        ));
    }

    Ok(())
}

fn scatter_stage_error(
    operation: &str,
    error: ScatterError,
) -> StageError {
    StageError::new(format!("{operation}: {error:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use terrakit_core::{
        Extent2,
        GenerationRegion,
        GenerationSeed,
        LodLevel,
        RegionCoord2,
        RegionLayout2,
        RegionRequest2,
        Vector2F64,
    };

    fn context2(width: u32, height: u32) -> StageContext {
        let layout = RegionLayout2::new(
            Extent2::try_new(width, height).unwrap(),
            Vector2F64::new(1.0, 1.0),
        )
        .unwrap();

        let region = GenerationRegion::Region2(
            layout
                .resolve(RegionRequest2::new(
                    RegionCoord2::ZERO,
                    LodLevel::HIGHEST,
                ))
                .unwrap(),
        );

        StageContext::new(GenerationSeed::new(123), region)
    }

    #[test]
    fn generates_scatter_points() {
        let context = context2(20, 20);

        let mut height_stage =
            crate::FlatHeightStage::standard(
                StageId(1),
                ResourceKey::HEIGHT,
                0.0,
            )
            .unwrap();

        let mut scatter_stage = PoissonScatterStage::new(
            StageId(2),
            ResourceKey::HEIGHT,
            ResourceKey::SCATTER_POINTS,
            2.0,
            30,
            42,
            1.0,
            1.0,
            true,
        )
        .unwrap();

        let mut resources = ResourceSet::new();

        // First create the terrain.
        height_stage
            .execute(&context, &mut resources)
            .unwrap();

        // Then scatter objects across the terrain.
        scatter_stage
            .execute(&context, &mut resources)
            .unwrap();

        let Some(TerrainResource::ScatterPoints(points)) =
            resources.get(ResourceKey::SCATTER_POINTS)
        else {
            panic!("expected scatter points resource");
        };

        println!("Generated {} scatter points", points.len());

        assert!(!points.is_empty());
    }
}