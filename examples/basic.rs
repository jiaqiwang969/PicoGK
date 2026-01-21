//! Basic example - Create a sphere and save to STL

use nalgebra::Vector3;
use picogk::{Library, Result, Voxels};

fn main() -> Result<()> {
    // Initialize PicoGK library with 0.5mm voxel size
    let _lib = Library::init(0.5)?;

    println!("PicoGK {}", Library::version());
    println!("Creating sphere...");

    // Create a sphere at origin with radius 20mm
    let sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;

    // Convert to mesh
    let mesh = sphere.as_mesh()?;
    println!("Generated mesh with {} vertices", mesh.vertex_count());

    // Save to STL
    mesh.save_stl("sphere.stl")?;
    println!("Saved sphere.stl");
    println!("Done!");

    Ok(())
}
