//! Temperature map stage definition.

use terrakit_core::SeedDomain;

use terrakit_pipeline::{
    DefinitionError, ParameterType, ParameterValue, ResolvedParameterSet, ResourceKind,
    StageBuildError, StageDefinition, StageId, StageSchema, StageSchemaVersion,
    ValidatedStageBindings,
};

use crate::height::TemperatureMapStage;

use super::{input, output, parameter, schema};

/// Stage definition for generating a noise-based temperature map as a HeightField resource.
pub struct TemperatureMapDefinition {
    schema: StageSchema,
}

impl TemperatureMapDefinition {
    /// Stable stage type ID for temperature map generation.
    pub const TYPE_ID: &'static str = "terrakit.height.temperature";
    /// Compatible schema revision for this built-in definition.
    pub const SCHEMA_VERSION: StageSchemaVersion = StageSchemaVersion::V1;

    /// Creates the built-in temperature map stage definition.
    pub fn new() -> Result<Self, DefinitionError> {
        Ok(Self {
            schema: schema(
                Self::TYPE_ID,
                Self::SCHEMA_VERSION,
                "Temperature Map",
                "Height",
                "Generates a noise-based temperature map as a HeightField resource. \
                 Accepts an elevation map to reduce temperature at higher altitudes.",
                vec![input(
                    "elevation",
                    "Elevation",
                    "Elevation height field used to cool temperature at higher altitudes.",
                    ResourceKind::HeightField,
                )?],
                vec![output(
                    "height",
                    "Height",
                    "Temperature map as a height field.",
                    ResourceKind::HeightField,
                )?],
                vec![
                    parameter(
                        "scale",
                        "Scale",
                        "Noise sampling scale.",
                        ParameterType::F64 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::F64(0.01)),
                    )?,
                    parameter(
                        "seed_domain",
                        "Seed Domain",
                        "Domain used to derive the deterministic temperature noise seed.",
                        ParameterType::U64 {
                            minimum: None,
                            maximum: None,
                        },
                        Some(ParameterValue::U64(1)),
                    )?,
                    parameter(
                        "elevation_influence",
                        "Elevation Influence",
                        "How much elevation reduces temperature (0.0 = none, 1.0 = full).",
                        ParameterType::F32 {
                            minimum: Some(0.0),
                            maximum: Some(1.0),
                        },
                        Some(ParameterValue::F32(0.3)),
                    )?,
                ],
            )?,
        })
    }
}

impl StageDefinition for TemperatureMapDefinition {
    fn schema(&self) -> &StageSchema {
        &self.schema
    }

    fn build(
        &self,
        stage_id: StageId,
        parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn terrakit_pipeline::TerrainStage>, StageBuildError> {
        let stage = TemperatureMapStage::new(
            stage_id,
            bindings.required_input("elevation")?,
            bindings.output("height")?,
            parameters.f64("scale")?,
            SeedDomain::new(parameters.u64("seed_domain")?),
            parameters.f32("elevation_influence")?,
        )
        .map_err(|source| {
            StageBuildError::from_stage("failed to build Temperature Map stage", source)
        })?;

        Ok(Box::new(stage))
    }
}
