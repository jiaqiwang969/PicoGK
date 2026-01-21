use picogk::{BBox3, GyroidImplicit, Lattice, Library, Result, TwistedTorusImplicit, Voxels};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

// This is a heavy, end-to-end parity check against the checked-in C# outputs in
// `PicoGK_Test/AdvancedExamples.cs`. Keep it opt-in.
#[test]
#[ignore]
fn parity_against_csharp_advanced_examples() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = manifest_dir.join("target/csharp_advanced_examples_out");
    let baseline_dir = manifest_dir.join("target/csharp_advanced_examples_baseline");

    let files = [
        "gyroid.stl",
        "expanded_sphere.stl",
        "shrunk_sphere.stl",
        "smooth_sphere.stl",
        "shell_structure.stl",
        "shell_smooth.stl",
        "twisted_torus.stl",
        "heat_exchanger.stl",
        "parametric_lattice.stl",
    ];

    ensure_csharp_baseline(&baseline_dir, &files)?;

    std::fs::create_dir_all(&out_dir).map_err(|e| {
        picogk::Error::OperationFailed(format!("Failed to create output dir: {}", e))
    })?;

    // Match `AdvancedExamples.Run()` -> `Library.Go(0.3f, ...)`.
    let _lib = Library::init(0.3)?;

    generate_outputs(&out_dir)?;

    for name in files {
        let expected = baseline_dir.join(name);
        let actual = out_dir.join(name);
        compare_binary_stl_vertices(&expected, &actual)
            .map_err(|e| picogk::Error::OperationFailed(format!("{}: {}", name, e)))?;
    }

    Ok(())
}

fn ensure_csharp_baseline(baseline_dir: &Path, files: &[&str]) -> Result<()> {
    if files.iter().all(|name| baseline_dir.join(name).exists()) {
        return Ok(());
    }

    std::fs::create_dir_all(baseline_dir).map_err(|e| {
        picogk::Error::OperationFailed(format!("Failed to create baseline dir: {}", e))
    })?;

    let status = Command::new("dotnet")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["run", "--project", "../PicoGK_Test", "-c", "Release"])
        .env("PICOGK_TEST_OUTPUT_DIR", baseline_dir.as_os_str())
        .status()
        .map_err(|e| {
            picogk::Error::OperationFailed(format!(
                "Failed to spawn dotnet (required for parity baseline generation): {}",
                e
            ))
        })?;

    if !status.success() {
        return Err(picogk::Error::OperationFailed(format!(
            "dotnet run failed (status: {})",
            status
        )));
    }

    // Ensure all expected files exist.
    for name in files {
        let p = baseline_dir.join(name);
        if !p.exists() {
            return Err(picogk::Error::OperationFailed(format!(
                "C# baseline generation did not produce {}",
                p.display()
            )));
        }
    }

    Ok(())
}

fn generate_outputs(out_dir: &Path) -> Result<()> {
    use nalgebra::Vector3;

    // Example 1: Gyroid
    {
        let bounds = BBox3::new(
            Vector3::new(-30.0, -30.0, -30.0),
            Vector3::new(30.0, 30.0, 30.0),
        );
        let gyroid = GyroidImplicit::new(10.0, 1.5, bounds);
        Voxels::from_implicit(&gyroid)?
            .as_mesh()?
            .save_stl(out_dir.join("gyroid.stl"))?;
    }

    // Example 2: Offset + Smoothen
    {
        let base = Voxels::sphere(Vector3::zeros(), 15.0)?;
        base.vox_offset(5.0)?
            .as_mesh()?
            .save_stl(out_dir.join("expanded_sphere.stl"))?;
        base.vox_offset(-3.0)?
            .as_mesh()?
            .save_stl(out_dir.join("shrunk_sphere.stl"))?;
        base.vox_smoothen(2.0)?
            .as_mesh()?
            .save_stl(out_dir.join("smooth_sphere.stl"))?;
    }

    // Example 3: Shell structures
    {
        let mut lat = Lattice::new()?;
        lat.add_sphere(Vector3::zeros(), 20.0);
        for i in 0..6 {
            let angle = i as f32 * std::f32::consts::PI / 3.0;
            let dir = Vector3::new(angle.cos(), angle.sin(), 0.0) * 25.0;
            lat.add_beam(Vector3::zeros(), dir, 5.0, 2.0);
        }

        let solid = Voxels::from_lattice(&lat)?;
        solid
            .shell_with_offsets(-3.0, 0.0, 0.0)?
            .as_mesh()?
            .save_stl(out_dir.join("shell_structure.stl"))?;
        solid
            .shell_with_offsets(-4.0, 1.0, 1.5)?
            .as_mesh()?
            .save_stl(out_dir.join("shell_smooth.stl"))?;
    }

    // Example 4: Twisted torus
    {
        let bounds = BBox3::new(
            Vector3::new(-30.0, -30.0, -10.0),
            Vector3::new(30.0, 30.0, 10.0),
        );
        let torus = TwistedTorusImplicit::new(20.0, 5.0, 3.0, bounds);
        Voxels::from_implicit(&torus)?
            .as_mesh()?
            .save_stl(out_dir.join("twisted_torus.stl"))?;
    }

    // Example 5: Heat exchanger
    {
        let outer = Voxels::sphere(Vector3::zeros(), 25.0)?;

        let bounds = BBox3::new(
            Vector3::new(-23.0, -23.0, -23.0),
            Vector3::new(23.0, 23.0, 23.0),
        );
        let gyroid = GyroidImplicit::new(8.0, 1.0, bounds);
        let inner = Voxels::from_implicit(&gyroid)?;

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

        let mut result = outer.vox_bool_intersect(&inner)?;
        result.bool_add(&vox_channels);
        result
            .as_mesh()?
            .save_stl(out_dir.join("heat_exchanger.stl"))?;
    }

    // Example 6: Parametric lattice
    {
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
    }

    Ok(())
}

fn compare_binary_stl_vertices(expected: &Path, actual: &Path) -> std::io::Result<()> {
    let (mut exp, exp_triangles) = open_and_read_triangle_count(expected)?;
    let (mut act, act_triangles) = open_and_read_triangle_count(actual)?;

    if exp_triangles != act_triangles {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "triangle_count mismatch: expected {} != actual {}",
                exp_triangles, act_triangles
            ),
        ));
    }

    // Binary STL: 80-byte header, 4-byte triangle count, then N records of 50 bytes:
    // - 12 bytes normal (ignored here)
    // - 36 bytes vertices (3x vec3f) (compared)
    // - 2 bytes attribute (compared)
    let mut exp_buf = [0u8; 50];
    let mut act_buf = [0u8; 50];
    for tri_idx in 0..exp_triangles {
        exp.read_exact(&mut exp_buf)?;
        act.read_exact(&mut act_buf)?;

        if exp_buf[12..] != act_buf[12..] {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "triangle {} vertex/attr bytes differ (normals ignored)",
                    tri_idx
                ),
            ));
        }
    }

    Ok(())
}

fn open_and_read_triangle_count(path: &Path) -> std::io::Result<(File, u32)> {
    let mut f = File::open(path)?;

    // Skip header.
    let mut header = [0u8; 80];
    f.read_exact(&mut header)?;

    let mut count_bytes = [0u8; 4];
    f.read_exact(&mut count_bytes)?;
    let n = u32::from_le_bytes(count_bytes);

    Ok((f, n))
}
