use nalgebra::Vector3;
use picogk::{Library, Mesh, TempFolder, Triangle};
use serial_test::serial;

#[test]
#[serial]
fn test_stl_save_and_load() {
    // Initialize library
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a simple triangle mesh
    let mut mesh_original = Mesh::new().expect("Failed to create mesh");

    // Add vertices for a simple triangle
    let v0 = mesh_original.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    let v1 = mesh_original.add_vertex(Vector3::new(10.0, 0.0, 0.0));
    let v2 = mesh_original.add_vertex(Vector3::new(5.0, 10.0, 0.0));

    // Add triangle
    mesh_original.add_triangle(Triangle::new(v0, v1, v2));

    println!(
        "Original mesh: {} vertices, {} triangles",
        mesh_original.vertex_count(),
        mesh_original.triangle_count()
    );

    // Save to STL file
    let tmp = TempFolder::new().expect("Failed to create temp folder");
    let output_path = tmp.path().join("test_roundtrip.stl");
    mesh_original
        .save_stl(&output_path)
        .expect("Failed to save STL");
    println!("✓ STL file saved");

    // Load from STL file
    let mesh_loaded = Mesh::load_stl(&output_path).expect("Failed to load STL");
    println!("✓ STL file loaded");

    println!(
        "Loaded mesh: {} vertices, {} triangles",
        mesh_loaded.vertex_count(),
        mesh_loaded.triangle_count()
    );

    // Verify triangle count matches
    assert_eq!(
        mesh_loaded.triangle_count(),
        mesh_original.triangle_count(),
        "Triangle count mismatch"
    );

    // Verify vertices (should be 3 vertices per triangle since we don't deduplicate)
    assert_eq!(mesh_loaded.vertex_count(), 3, "Vertex count should be 3");

    // Verify the vertices are approximately correct
    let loaded_v0 = mesh_loaded.get_vertex(0).expect("Failed to get vertex 0");
    let loaded_v1 = mesh_loaded.get_vertex(1).expect("Failed to get vertex 1");
    let loaded_v2 = mesh_loaded.get_vertex(2).expect("Failed to get vertex 2");

    println!("Loaded vertices:");
    println!("  v0 = {:?}", loaded_v0);
    println!("  v1 = {:?}", loaded_v1);
    println!("  v2 = {:?}", loaded_v2);

    // Check vertices are approximately correct (within 0.01mm tolerance)
    let epsilon = 0.01;
    assert!(
        (loaded_v0 - Vector3::new(0.0, 0.0, 0.0)).norm() < epsilon,
        "v0 mismatch: {:?}",
        loaded_v0
    );
    assert!(
        (loaded_v1 - Vector3::new(10.0, 0.0, 0.0)).norm() < epsilon,
        "v1 mismatch: {:?}",
        loaded_v1
    );
    assert!(
        (loaded_v2 - Vector3::new(5.0, 10.0, 0.0)).norm() < epsilon,
        "v2 mismatch: {:?}",
        loaded_v2
    );

    println!("✓ All vertices match within tolerance");
}
