use nalgebra::Vector3;
use picogk::{Lattice, Library, Mesh, Voxels};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PicoGK Rust Bindings - Comprehensive Demo ===\n");

    // Initialize library
    let _lib = Library::init(0.5)?;
    println!("✓ Library initialized (voxel size: 0.5mm)");
    println!("  Name: {}", Library::name());
    println!("  Version: {}", Library::version());

    // ========================================
    // 1. Lattice Operations
    // ========================================
    println!("\n--- 1. Lattice Operations ---");

    let mut lattice = Lattice::new()?;
    lattice.add_sphere(Vector3::zeros(), 5.0);
    lattice.add_sphere(Vector3::new(10.0, 0.0, 0.0), 5.0);
    lattice.add_beam(Vector3::zeros(), Vector3::new(10.0, 0.0, 0.0), 2.0, 2.0);
    println!("✓ Created lattice with spheres and beam");

    // ========================================
    // 2. Voxels Creation and Boolean Operations
    // ========================================
    println!("\n--- 2. Voxels Boolean Operations ---");

    let sphere1 = Voxels::sphere(Vector3::zeros(), 10.0)?;
    let sphere2 = Voxels::sphere(Vector3::new(8.0, 0.0, 0.0), 10.0)?;
    println!("✓ Created two spheres");

    // Mutable operations
    let mut union = sphere1.try_clone()?;
    union.bool_add(&sphere2);
    println!("✓ Boolean union (mutable)");

    // Functional operations
    let _difference = sphere1.vox_bool_subtract(&sphere2)?;
    println!("✓ Boolean difference (functional)");

    let _intersection = sphere1.vox_bool_intersect(&sphere2)?;
    println!("✓ Boolean intersection (functional)");

    // ========================================
    // 3. Offset Operations
    // ========================================
    println!("\n--- 3. Offset Operations ---");

    let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;

    // Basic offset
    let _expanded = sphere.vox_offset(2.0)?;
    println!("✓ Basic offset (+2.0mm)");

    // Double offset
    let _double = sphere.vox_double_offset(2.0, -1.0)?;
    println!("✓ Double offset (2.0mm, -1.0mm)");

    // Triple offset (smoothing)
    let _smoothed = sphere.vox_triple_offset(1.0)?;
    println!("✓ Triple offset / smoothing (1.0mm)");

    // Fillet
    let _filleted = sphere.vox_fillet(1.5)?;
    println!("✓ Fillet (1.5mm rounding)");

    // ========================================
    // 4. Method Chaining
    // ========================================
    println!("\n--- 4. Method Chaining ---");

    let result = Voxels::sphere(Vector3::zeros(), 10.0)?
        .vox_offset(2.0)?
        .vox_smoothen(1.0)?
        .vox_fillet(0.5)?;
    println!("✓ Chained: offset -> smoothen -> fillet");

    // ========================================
    // 5. Mesh Operations
    // ========================================
    println!("\n--- 5. Mesh Operations ---");

    // Create mesh from voxels
    let mesh = result.as_mesh()?;
    println!("✓ Converted voxels to mesh");
    println!("  Vertices: {}", mesh.vertex_count());
    println!("  Triangles: {}", mesh.triangle_count());

    // Get bounding box
    let bbox = mesh.bounding_box();
    let min = bbox.min();
    let max = bbox.max();
    println!("✓ Calculated bounding box");
    println!("  Min: ({:.2}, {:.2}, {:.2})", min.x, min.y, min.z);
    println!("  Max: ({:.2}, {:.2}, {:.2})", max.x, max.y, max.z);

    // ========================================
    // 6. STL File I/O
    // ========================================
    println!("\n--- 6. STL File I/O ---");

    // Save to STL
    mesh.save_stl("demo_output.stl")?;
    println!("✓ Saved mesh to demo_output.stl");

    // Load from STL
    let loaded_mesh = Mesh::load_stl("demo_output.stl")?;
    println!("✓ Loaded mesh from demo_output.stl");
    println!("  Vertices: {}", loaded_mesh.vertex_count());
    println!("  Triangles: {}", loaded_mesh.triangle_count());

    // ========================================
    // 7. Manual Mesh Creation
    // ========================================
    println!("\n--- 7. Manual Mesh Creation ---");

    let mut manual_mesh = Mesh::new()?;
    let v0 = manual_mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    let v1 = manual_mesh.add_vertex(Vector3::new(10.0, 0.0, 0.0));
    let v2 = manual_mesh.add_vertex(Vector3::new(5.0, 10.0, 0.0));
    manual_mesh.add_triangle(picogk::Triangle::new(v0, v1, v2));
    println!("✓ Created manual mesh with 1 triangle");

    manual_mesh.save_stl("triangle.stl")?;
    println!("✓ Saved to triangle.stl");

    // ========================================
    // Summary
    // ========================================
    println!("\n=== Demo Complete ===");
    println!("All operations completed successfully!");
    println!("\nGenerated files:");
    println!("  - demo_output.stl");
    println!("  - triangle.stl");

    Ok(())
}
