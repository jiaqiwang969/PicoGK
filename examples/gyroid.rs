//! Gyroid example - Create a Gyroid TPMS structure

use nalgebra::Vector3;
use picogk::{BBox3, GyroidImplicit, Implicit, Library, Result, Voxels};

fn main() -> Result<()> {
    let _lib = Library::init(0.3)?;

    println!("PicoGK {}", Library::version());
    println!("Creating Gyroid structure...");

    // Define bounding box
    let bounds = BBox3::new(
        Vector3::new(-30.0, -30.0, -30.0),
        Vector3::new(30.0, 30.0, 30.0),
    );

    // Create Gyroid implicit
    let gyroid = GyroidImplicit::new(
        10.0, // scale (period size)
        1.5,  // thickness
        bounds,
    );

    println!("Gyroid parameters:");
    println!("  Scale: 10.0mm");
    println!("  Thickness: 1.5mm");
    println!("  Bounds: {}", bounds);

    // Test signed distance at a few points
    let test_points = vec![
        Vector3::zeros(),
        Vector3::new(5.0, 0.0, 0.0),
        Vector3::new(0.0, 5.0, 0.0),
    ];

    println!("\nSigned distances:");
    for point in test_points {
        let dist = gyroid.signed_distance(point);
        println!("  {:?}: {:.3}", point, dist);
    }

    let vox = Voxels::from_implicit(&gyroid)?;
    let mesh = vox.as_mesh()?;
    mesh.save_stl("gyroid.stl")?;

    println!("\nDone!");

    Ok(())
}
