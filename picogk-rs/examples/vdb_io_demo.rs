use nalgebra::Vector3;
use picogk::{Library, VdbFile, Voxels};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PicoGK VDB File I/O Demo ===\n");

    // Initialize library
    let _lib = Library::init(0.5)?;
    println!("✓ Library initialized (voxel size: 0.5mm)");

    // ========================================
    // 1. Simple VDB Save/Load
    // ========================================
    println!("\n--- 1. Simple VDB Save/Load ---");

    let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;
    println!("✓ Created sphere (radius: 10mm)");

    sphere.save_vdb("simple_sphere.vdb")?;
    println!("✓ Saved to simple_sphere.vdb");

    let loaded = Voxels::load_vdb("simple_sphere.vdb")?;
    println!("✓ Loaded from simple_sphere.vdb");
    println!("  Valid: {}", loaded.is_valid());

    // ========================================
    // 2. Multiple Fields in One VDB File
    // ========================================
    println!("\n--- 2. Multiple Fields in One VDB File ---");

    let sphere1 = Voxels::sphere(Vector3::zeros(), 10.0)?;
    let sphere2 = Voxels::sphere(Vector3::new(15.0, 0.0, 0.0), 8.0)?;
    let sphere3 = Voxels::sphere(Vector3::new(0.0, 15.0, 0.0), 6.0)?;

    let mut vdb = VdbFile::new()?;
    vdb.add_voxels(&sphere1, "large_sphere")?;
    vdb.add_voxels(&sphere2, "medium_sphere")?;
    vdb.add_voxels(&sphere3, "small_sphere")?;
    println!("✓ Added 3 voxel fields to VDB file");

    vdb.save("multi_field.vdb")?;
    println!("✓ Saved to multi_field.vdb");

    // Load and inspect
    let loaded_vdb = VdbFile::load("multi_field.vdb")?;
    println!("✓ Loaded multi_field.vdb");
    println!("  Field count: {}", loaded_vdb.field_count());

    for i in 0..loaded_vdb.field_count() {
        println!(
            "  Field {}: {} ({:?})",
            i,
            loaded_vdb.field_name(i),
            loaded_vdb.field_type(i)
        );
    }

    // Load specific field by name
    let medium = loaded_vdb.get_voxels_by_name("medium_sphere")?;
    println!("✓ Loaded 'medium_sphere' by name");
    println!("  Valid: {}", medium.is_valid());

    // ========================================
    // 3. Complex Geometry Workflow
    // ========================================
    println!("\n--- 3. Complex Geometry Workflow ---");

    // Create complex shape
    let s1 = Voxels::sphere(Vector3::zeros(), 10.0)?;
    let s2 = Voxels::sphere(Vector3::new(8.0, 0.0, 0.0), 10.0)?;
    let union = s1.vox_bool_add(&s2)?.vox_smoothen(1.0)?.vox_fillet(0.5)?;
    println!("✓ Created complex shape (union + smoothing + fillet)");

    // Save to VDB
    union.save_vdb("complex_shape.vdb")?;
    println!("✓ Saved to complex_shape.vdb");

    // Load and convert to mesh
    let loaded_complex = Voxels::load_vdb("complex_shape.vdb")?;
    let mesh = loaded_complex.as_mesh()?;
    println!("✓ Loaded and converted to mesh");
    println!("  Vertices: {}", mesh.vertex_count());
    println!("  Triangles: {}", mesh.triangle_count());

    // Save mesh as STL
    mesh.save_stl("complex_shape.stl")?;
    println!("✓ Saved mesh to complex_shape.stl");

    // ========================================
    // Summary
    // ========================================
    println!("\n=== Demo Complete ===");
    println!("Generated files:");
    println!("  - simple_sphere.vdb");
    println!("  - multi_field.vdb (3 fields)");
    println!("  - complex_shape.vdb");
    println!("  - complex_shape.stl");

    Ok(())
}
