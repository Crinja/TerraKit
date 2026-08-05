//! Canonical terrain resource types.
//!
//! These resources are the values exchanged between pipeline stages and stored
//! in `terrakit_pipeline::ResourceSet`.

mod density;
mod height_field;
mod mesh;
mod voxel;

pub use density::{DensityField, DensitySample};
pub use height_field::{HeightField, HeightSample};
pub use mesh::{MeshIndex, TerrainMesh, TerrainMeshParts};
pub use voxel::{VoxelId, VoxelVolume};

/// Canonical terrain resources shared between TerraKit pipeline stages.
#[derive(Debug, Clone, PartialEq)]
pub enum TerrainResource {
    /// A two-dimensional height surface.
    HeightField(HeightField),

    /// A three-dimensional scalar density field.
    DensityField(DensityField),

    /// A dense voxel material or occupancy volume.
    VoxelVolume(VoxelVolume),

    /// A validated indexed triangle mesh.
    Mesh(TerrainMesh),
}
