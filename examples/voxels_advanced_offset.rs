use nalgebra::Vector3;
use picogk::{Library, Voxels};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize library
    let _lib = Library::init(0.5)?;

    println!("=== Voxels Advanced Offset Operations Demo ===\n");

    // Create a sphere
    let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;
    println!("Created sphere with radius 10mm");

    // Test triple offset (smoothing)
    println!("\n1. Triple Offset (Smoothing):");
    let mut smoothed = sphere.try_clone()?;
    smoothed.triple_offset(1.0);
    println!("   Applied triple offset with distance 1.0mm");
    println!("   Result is valid: {}", smoothed.is_valid());

    // Test smoothen (alias for triple offset)
    println!("\n2. Smoothen:");
    let mut smoothed2 = sphere.try_clone()?;
    smoothed2.smoothen(1.0);
    println!("   Applied smoothen with distance 1.0mm");
    println!("   Result is valid: {}", smoothed2.is_valid());

    // Test over offset
    println!("\n3. Over Offset:");
    let mut over_offset = sphere.try_clone()?;
    over_offset.over_offset(2.0, 0.5);
    println!("   Applied over offset: first=2.0mm, final_dist=0.5mm");
    println!("   Result is valid: {}", over_offset.is_valid());

    // Test fillet
    println!("\n4. Fillet:");
    let mut filleted = sphere.try_clone()?;
    filleted.fillet(1.5);
    println!("   Applied fillet with rounding 1.5mm");
    println!("   Result is valid: {}", filleted.is_valid());

    // Compare with basic offset
    println!("\n5. Basic Offset (for comparison):");
    let mut basic = sphere.try_clone()?;
    basic.offset(2.0);
    println!("   Applied basic offset with distance 2.0mm");
    println!("   Result is valid: {}", basic.is_valid());

    println!("\n✓ All advanced offset operations completed successfully!");

    Ok(())
}
