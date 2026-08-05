//! Command-line demonstration interface for TerraKit.
//!
//! The `mesh-demo` command assembles the first-party height pipeline through
//! discoverable stage definitions, runs several neighboring 2D regions through
//! the synchronous runtime, exports each mesh as OBJ, and verifies boundary
//! continuity between adjacent outputs.

mod obj;

use std::{env, fs, path::PathBuf};

use terrakit_builtins::{
    FlatHeightDefinition, HeightFieldMeshDefinition, NoiseHeightDefinition, builtin_stage_registry,
};
use terrakit_core::{
    Extent2, GenerationSeed, HeightField, LodLevel, RegionCoord2, RegionLayout2, TerrainMesh,
    Vector2F64, Vector3F64,
};
use terrakit_pipeline::{
    EnumValueId, ParameterId, ParameterSet, ParameterValue, PipelineError, PortId, ResourceKey,
    StageConstruction, StageId, StageTypeId, TerrainPipelineAssembler,
};
use terrakit_runtime::{GenerationRequest, GenerationResult, TerrainRuntime};

const POSITION_EPSILON: f64 = 1.0e-5;
const BASE_HEIGHT: ResourceKey = ResourceKey(100);
const NOISY_HEIGHT: ResourceKey = ResourceKey(101);
const TERRAIN_MESH: ResourceKey = ResourceKey(102);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match env::args().nth(1).as_deref() {
        None | Some("mesh-demo") => run_mesh_demo(),
        Some(command) => Err(Box::new(PipelineError::new(format!(
            "unknown terrakit-console command '{command}', expected 'mesh-demo'"
        )))),
    }
}

fn run_mesh_demo() -> Result<(), Box<dyn std::error::Error>> {
    let layout = RegionLayout2::new(Extent2::try_new(48, 48)?, Vector2F64::new(1.0, 1.0))?;
    let seed = GenerationSeed::new(1234);
    let lod = LodLevel::HIGHEST;
    let output_dir = PathBuf::from("build").join("demo");
    fs::create_dir_all(&output_dir)?;
    let mut runtime = build_demo_runtime(layout)?;

    let mut regions = Vec::new();
    for coordinate in [
        RegionCoord2::new(1, 1),
        RegionCoord2::new(1, 0),
        RegionCoord2::new(1, -1),
        RegionCoord2::new(0, 1),
        RegionCoord2::new(0, 0),
        RegionCoord2::new(0, -1),
        RegionCoord2::new(-1, 1),
        RegionCoord2::new(-1, 0),
        RegionCoord2::new(-1, -1),
    ] {
        let output_path = output_dir.join(format!("region_{}_{}.obj", coordinate.x, coordinate.y));
        let region = generate_demo_region(&mut runtime, coordinate, lod, seed, output_path)?;
        regions.push(region);
    }

    for region in &regions {
        print_region_summary(region)?;
    }

    println!();
    confirm_adjacent_mesh_edges_match(&regions)?;

    Ok(())
}

#[derive(Debug)]
struct DemoRegion {
    coordinate: RegionCoord2,
    lod: LodLevel,
    result: GenerationResult,
    output_path: PathBuf,
}

impl DemoRegion {
    fn height_field(&self) -> Result<&HeightField, PipelineError> {
        self.result
            .resources()
            .height_field(NOISY_HEIGHT)
            .ok_or_else(|| PipelineError::new("mesh demo produced no height field"))
    }

    fn mesh(&self) -> Result<&TerrainMesh, PipelineError> {
        self.result
            .resources()
            .mesh(TERRAIN_MESH)
            .ok_or_else(|| PipelineError::new("mesh demo produced no terrain mesh"))
    }
}

fn build_demo_runtime(layout: RegionLayout2) -> Result<TerrainRuntime, Box<dyn std::error::Error>> {
    let registry = builtin_stage_registry()?;
    let mut assembler = TerrainPipelineAssembler::new(&registry);

    let mut flat_parameters = ParameterSet::new();
    flat_parameters.set(parameter_id("elevation")?, ParameterValue::F32(0.0));
    assembler.add_stage(
        StageConstruction::new(
            StageId(1),
            stage_type_id(FlatHeightDefinition::TYPE_ID)?,
            FlatHeightDefinition::SCHEMA_VERSION,
        )
        .with_parameters(flat_parameters)
        .with_bindings(
            terrakit_pipeline::StageBindings::new().with_output(port_id("height")?, BASE_HEIGHT)?,
        ),
    )?;

    let mut noise_parameters = ParameterSet::new();
    noise_parameters.set(
        parameter_id("algorithm")?,
        ParameterValue::Enum(enum_value_id("value")?),
    );
    noise_parameters.set(parameter_id("octaves")?, ParameterValue::U64(4));
    noise_parameters.set(parameter_id("frequency")?, ParameterValue::F64(0.12));
    noise_parameters.set(parameter_id("lacunarity")?, ParameterValue::F64(2.0));
    noise_parameters.set(parameter_id("persistence")?, ParameterValue::F32(0.5));
    noise_parameters.set(parameter_id("amplitude")?, ParameterValue::F32(0.35));
    noise_parameters.set(parameter_id("normalize")?, ParameterValue::Bool(true));
    noise_parameters.set(parameter_id("seed_domain")?, ParameterValue::U64(0));
    noise_parameters.set(
        parameter_id("mode")?,
        ParameterValue::Enum(enum_value_id("add")?),
    );
    assembler.add_stage(
        StageConstruction::new(
            StageId(2),
            stage_type_id(NoiseHeightDefinition::TYPE_ID)?,
            NoiseHeightDefinition::SCHEMA_VERSION,
        )
        .with_parameters(noise_parameters)
        .with_bindings(
            terrakit_pipeline::StageBindings::new()
                .with_input(port_id("source")?, BASE_HEIGHT)?
                .with_output(port_id("height")?, NOISY_HEIGHT)?,
        ),
    )?;

    assembler.add_stage(
        StageConstruction::new(
            StageId(3),
            stage_type_id(HeightFieldMeshDefinition::TYPE_ID)?,
            HeightFieldMeshDefinition::SCHEMA_VERSION,
        )
        .with_bindings(
            terrakit_pipeline::StageBindings::new()
                .with_input(port_id("height")?, NOISY_HEIGHT)?
                .with_output(port_id("mesh")?, TERRAIN_MESH)?,
        ),
    )?;

    Ok(TerrainRuntime::new(layout, assembler.finish()))
}

fn generate_demo_region(
    runtime: &mut TerrainRuntime,
    coordinate: RegionCoord2,
    lod: LodLevel,
    seed: GenerationSeed,
    output_path: PathBuf,
) -> Result<DemoRegion, Box<dyn std::error::Error>> {
    let result = runtime.generate(GenerationRequest::region2(seed, coordinate, lod))?;
    let mesh = result
        .resources()
        .mesh(TERRAIN_MESH)
        .ok_or_else(|| PipelineError::new("mesh demo produced no terrain mesh"))?;

    obj::write_obj(&output_path, mesh)?;

    Ok(DemoRegion {
        coordinate,
        lod,
        result,
        output_path,
    })
}

fn print_region_summary(region: &DemoRegion) -> Result<(), Box<dyn std::error::Error>> {
    let mesh = region.mesh()?;
    let origin = mesh.origin();

    println!(
        "Region ({}, {}) LOD {} -> mesh origin ({:.1}, {:.1}, {:.1}), vertices {}, triangles {}, OBJ {}",
        region.coordinate.x,
        region.coordinate.y,
        region.lod.value(),
        origin.x,
        origin.y,
        origin.z,
        mesh.vertex_count(),
        mesh.triangle_count(),
        region.output_path.display()
    );

    Ok(())
}

fn confirm_adjacent_mesh_edges_match(
    regions: &[DemoRegion],
) -> Result<(), Box<dyn std::error::Error>> {
    for region in regions {
        if let Some(next_x) = region.coordinate.x.checked_add(1) {
            let right_coordinate = RegionCoord2::new(next_x, region.coordinate.y);
            if let Some(right) = find_region(regions, right_coordinate) {
                confirm_mesh_right_edge_match(region, right)?;
            }
        }

        if let Some(next_y) = region.coordinate.y.checked_add(1) {
            let positive_z_coordinate = RegionCoord2::new(region.coordinate.x, next_y);
            if let Some(positive_z) = find_region(regions, positive_z_coordinate) {
                confirm_mesh_positive_z_edge_match(region, positive_z)?;
            }
        }
    }

    Ok(())
}

fn find_region(regions: &[DemoRegion], coordinate: RegionCoord2) -> Option<&DemoRegion> {
    regions
        .iter()
        .find(|region| region.coordinate == coordinate)
}

fn confirm_mesh_right_edge_match(
    left: &DemoRegion,
    right: &DemoRegion,
) -> Result<(), Box<dyn std::error::Error>> {
    if mesh_right_edge_matches_left_edge(left, right)? {
        println!(
            "Boundary continuity: region ({}, {}) right edge matches region ({}, {}) left edge within {:.0e}.",
            left.coordinate.x,
            left.coordinate.y,
            right.coordinate.x,
            right.coordinate.y,
            POSITION_EPSILON
        );

        return Ok(());
    }

    Err(Box::new(PipelineError::new(format!(
        "region ({}, {}) right mesh edge did not match region ({}, {}) left mesh edge",
        left.coordinate.x, left.coordinate.y, right.coordinate.x, right.coordinate.y
    ))))
}

fn confirm_mesh_positive_z_edge_match(
    negative_z: &DemoRegion,
    positive_z: &DemoRegion,
) -> Result<(), Box<dyn std::error::Error>> {
    if mesh_positive_z_edge_matches_negative_z_edge(negative_z, positive_z)? {
        println!(
            "Boundary continuity: region ({}, {}) positive-Z edge matches region ({}, {}) negative-Z edge within {:.0e}.",
            negative_z.coordinate.x,
            negative_z.coordinate.y,
            positive_z.coordinate.x,
            positive_z.coordinate.y,
            POSITION_EPSILON
        );

        return Ok(());
    }

    Err(Box::new(PipelineError::new(format!(
        "region ({}, {}) positive-Z mesh edge did not match region ({}, {}) negative-Z mesh edge",
        negative_z.coordinate.x,
        negative_z.coordinate.y,
        positive_z.coordinate.x,
        positive_z.coordinate.y
    ))))
}

fn mesh_right_edge_matches_left_edge(
    left: &DemoRegion,
    right: &DemoRegion,
) -> Result<bool, PipelineError> {
    let left_height_field = left.height_field()?;
    let right_height_field = right.height_field()?;
    let left_mesh = left.mesh()?;
    let right_mesh = right.mesh()?;

    if left_height_field.height() != right_height_field.height()
        || left_height_field.width() == 0
        || right_height_field.width() == 0
    {
        return Ok(false);
    }

    let left_x = left_height_field.width() - 1;

    for y in 0..left_height_field.height() {
        let left_index = y * left_height_field.width() + left_x;
        let right_index = y * right_height_field.width();
        let left_position = reconstructed_world_position(left_mesh, left_index);
        let right_position = reconstructed_world_position(right_mesh, right_index);

        if max_abs_component_delta(left_position, right_position) > POSITION_EPSILON {
            return Ok(false);
        }
    }

    Ok(true)
}

fn mesh_positive_z_edge_matches_negative_z_edge(
    negative_z: &DemoRegion,
    positive_z: &DemoRegion,
) -> Result<bool, PipelineError> {
    let negative_z_height_field = negative_z.height_field()?;
    let positive_z_height_field = positive_z.height_field()?;
    let negative_z_mesh = negative_z.mesh()?;
    let positive_z_mesh = positive_z.mesh()?;

    if negative_z_height_field.width() != positive_z_height_field.width()
        || negative_z_height_field.height() == 0
        || positive_z_height_field.height() == 0
    {
        return Ok(false);
    }

    let negative_z_y = negative_z_height_field.height() - 1;

    for x in 0..negative_z_height_field.width() {
        let negative_z_index = negative_z_y * negative_z_height_field.width() + x;
        let positive_z_index = x;
        let negative_z_position = reconstructed_world_position(negative_z_mesh, negative_z_index);
        let positive_z_position = reconstructed_world_position(positive_z_mesh, positive_z_index);

        if max_abs_component_delta(negative_z_position, positive_z_position) > POSITION_EPSILON {
            return Ok(false);
        }
    }

    Ok(true)
}

fn reconstructed_world_position(mesh: &TerrainMesh, index: usize) -> Vector3F64 {
    let origin = mesh.origin();
    let local = mesh.positions()[index];

    Vector3F64::new(
        origin.x + f64::from(local.x),
        origin.y + f64::from(local.y),
        origin.z + f64::from(local.z),
    )
}

fn max_abs_component_delta(lhs: Vector3F64, rhs: Vector3F64) -> f64 {
    (lhs.x - rhs.x)
        .abs()
        .max((lhs.y - rhs.y).abs())
        .max((lhs.z - rhs.z).abs())
}

fn stage_type_id(value: &'static str) -> Result<StageTypeId, terrakit_pipeline::DefinitionError> {
    StageTypeId::try_new(value)
}

fn port_id(value: &'static str) -> Result<PortId, terrakit_pipeline::DefinitionError> {
    PortId::try_new(value)
}

fn parameter_id(value: &'static str) -> Result<ParameterId, terrakit_pipeline::DefinitionError> {
    ParameterId::try_new(value)
}

fn enum_value_id(value: &'static str) -> Result<EnumValueId, terrakit_pipeline::DefinitionError> {
    EnumValueId::try_new(value)
}
