//! Reproduce `PicoGK_Test/AdvancedExamples.cs` in Rust (headless).
//!
//! This is used for cross-language validation against the checked-in C# STL outputs.

use nalgebra::Vector3;
use picogk::{BBox3, GyroidImplicit, Lattice, Library, Result, TwistedTorusImplicit, Voxels};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let out_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&out_dir).map_err(|e| {
        picogk::Error::OperationFailed(format!("Failed to create output dir: {}", e))
    })?;

    // Match `AdvancedExamples.Run()` -> `Library.Go(0.3f, ...)`.
    let _lib = Library::init(0.3)?;

    example1_gyroid(&out_dir)?;
    example2_offset_and_smooth(&out_dir)?;
    example3_shell_structures(&out_dir)?;
    example4_twisted_torus(&out_dir)?;
    example5_heat_exchanger(&out_dir)?;
    example6_parametric_lattice(&out_dir)?;

    Ok(())
}

fn example1_gyroid(out_dir: &Path) -> Result<()> {
    let bounds = BBox3::new(
        Vector3::new(-30.0, -30.0, -30.0),
        Vector3::new(30.0, 30.0, 30.0),
    );
    let gyroid = GyroidImplicit::new(10.0, 1.5, bounds);
    let vox = Voxels::from_implicit(&gyroid)?;

    let (_volume, bbox) = vox.calculate_properties();
    println!("Gyroid bbox: {}", bbox);

    vox.as_mesh()?.save_stl(out_dir.join("gyroid.stl"))?;
    Ok(())
}

fn example2_offset_and_smooth(out_dir: &Path) -> Result<()> {
    let vox_base = Voxels::sphere(Vector3::zeros(), 15.0)?;

    vox_base
        .vox_offset(5.0)?
        .as_mesh()?
        .save_stl(out_dir.join("expanded_sphere.stl"))?;

    vox_base
        .vox_offset(-3.0)?
        .as_mesh()?
        .save_stl(out_dir.join("shrunk_sphere.stl"))?;

    vox_base
        .vox_smoothen(2.0)?
        .as_mesh()?
        .save_stl(out_dir.join("smooth_sphere.stl"))?;

    Ok(())
}

fn example3_shell_structures(out_dir: &Path) -> Result<()> {
    let mut lat = Lattice::new()?;
    lat.add_sphere(Vector3::zeros(), 20.0);

    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::PI / 3.0;
        let dir = Vector3::new(angle.cos(), angle.sin(), 0.0) * 25.0;
        lat.add_beam(Vector3::zeros(), dir, 5.0, 2.0);
    }

    let vox_solid = Voxels::from_lattice(&lat)?;

    vox_solid
        .shell_with_offsets(-3.0, 0.0, 0.0)?
        .as_mesh()?
        .save_stl(out_dir.join("shell_structure.stl"))?;

    vox_solid
        .shell_with_offsets(-4.0, 1.0, 1.5)?
        .as_mesh()?
        .save_stl(out_dir.join("shell_smooth.stl"))?;

    Ok(())
}

fn example4_twisted_torus(out_dir: &Path) -> Result<()> {
    let bounds = BBox3::new(
        Vector3::new(-30.0, -30.0, -10.0),
        Vector3::new(30.0, 30.0, 10.0),
    );
    let torus = TwistedTorusImplicit::new(20.0, 5.0, 3.0, bounds);
    Voxels::from_implicit(&torus)?
        .as_mesh()?
        .save_stl(out_dir.join("twisted_torus.stl"))?;
    Ok(())
}

fn example5_heat_exchanger(out_dir: &Path) -> Result<()> {
    let vox_outer = Voxels::sphere(Vector3::zeros(), 25.0)?;

    let bounds = BBox3::new(
        Vector3::new(-23.0, -23.0, -23.0),
        Vector3::new(23.0, 23.0, 23.0),
    );
    let gyroid = GyroidImplicit::new(8.0, 1.0, bounds);
    let vox_gyroid = Voxels::from_implicit(&gyroid)?;

    let mut channels = Lattice::new()?;
    channels.add_beam(
        Vector3::new(0.0, 0.0, -30.0),
        Vector3::new(0.0, 0.0, -20.0),
        3.0,
        3.0,
    );
    channels.add_beam(
        Vector3::new(0.0, 0.0, 20.0),
        Vector3::new(0.0, 0.0, 30.0),
        3.0,
        3.0,
    );
    let vox_channels = Voxels::from_lattice(&channels)?;

    let mut result = vox_outer.vox_bool_intersect(&vox_gyroid)?;
    result.bool_add(&vox_channels);

    result
        .as_mesh()?
        .save_stl(out_dir.join("heat_exchanger.stl"))?;
    Ok(())
}

fn example6_parametric_lattice(out_dir: &Path) -> Result<()> {
    let mut lattice = Lattice::new()?;

    let grid_size = 5usize;
    let spacing = 10.0f32;
    let beam_radius = 0.8f32;

    for x in 0..grid_size {
        for y in 0..grid_size {
            for z in 0..grid_size {
                let pos = Vector3::new(
                    (x as f32 - grid_size as f32 / 2.0) * spacing,
                    (y as f32 - grid_size as f32 / 2.0) * spacing,
                    (z as f32 - grid_size as f32 / 2.0) * spacing,
                );
                lattice.add_sphere(pos, beam_radius * 1.5);
            }
        }
    }

    for x in 0..grid_size {
        for y in 0..grid_size {
            for z in 0..grid_size {
                let pos = Vector3::new(
                    (x as f32 - grid_size as f32 / 2.0) * spacing,
                    (y as f32 - grid_size as f32 / 2.0) * spacing,
                    (z as f32 - grid_size as f32 / 2.0) * spacing,
                );

                if x < grid_size - 1 {
                    lattice.add_beam(
                        pos,
                        pos + Vector3::new(spacing, 0.0, 0.0),
                        beam_radius,
                        beam_radius,
                    );
                }
                if y < grid_size - 1 {
                    lattice.add_beam(
                        pos,
                        pos + Vector3::new(0.0, spacing, 0.0),
                        beam_radius,
                        beam_radius,
                    );
                }
                if z < grid_size - 1 {
                    lattice.add_beam(
                        pos,
                        pos + Vector3::new(0.0, 0.0, spacing),
                        beam_radius,
                        beam_radius,
                    );
                }
            }
        }
    }

    Voxels::from_lattice(&lattice)?
        .as_mesh()?
        .save_stl(out_dir.join("parametric_lattice.stl"))?;

    Ok(())
}
