//! Wavefront OBJ export helpers for demo meshes.

use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

use terrakit_core::TerrainMesh;

/// Writes a terrain mesh as a Wavefront OBJ file using mesh-local coordinates.
///
/// The mesh world origin is emitted as a comment instead of being baked into
/// every vertex. This preserves TerraKit's large-world contract: `f64` origin,
/// `f32` local vertex positions.
pub fn write_obj(path: impl AsRef<Path>, mesh: &TerrainMesh) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    let origin = mesh.origin();

    writeln!(writer, "# TerraKit OBJ export")?;
    writeln!(
        writer,
        "# TerraKit world origin: {:.17} {:.17} {:.17}",
        origin.x, origin.y, origin.z
    )?;

    for position in mesh.positions() {
        writeln!(
            writer,
            "v {:.9} {:.9} {:.9}",
            position.x, position.y, position.z
        )?;
    }

    if let Some(texcoords) = mesh.texcoords() {
        for texcoord in texcoords {
            writeln!(writer, "vt {:.9} {:.9}", texcoord.x, texcoord.y)?;
        }
    }

    if let Some(normals) = mesh.normals() {
        for normal in normals {
            writeln!(writer, "vn {:.9} {:.9} {:.9}", normal.x, normal.y, normal.z)?;
        }
    }

    for triangle in mesh.indices().chunks_exact(3) {
        write_face(
            &mut writer,
            triangle,
            mesh.texcoords().is_some(),
            mesh.normals().is_some(),
        )?;
    }

    writer.flush()
}

fn write_face<W: Write>(
    writer: &mut W,
    triangle: &[u32],
    has_texcoords: bool,
    has_normals: bool,
) -> io::Result<()> {
    write!(writer, "f")?;

    for index in triangle {
        let obj_index = u64::from(*index) + 1;

        match (has_texcoords, has_normals) {
            (true, true) => write!(writer, " {obj_index}/{obj_index}/{obj_index}")?,
            (true, false) => write!(writer, " {obj_index}/{obj_index}")?,
            (false, true) => write!(writer, " {obj_index}//{obj_index}")?,
            (false, false) => write!(writer, " {obj_index}")?,
        }
    }

    writeln!(writer)
}
