use nalgebra::Vector3;
use picogk::{Library, Mesh, TempFolder, Triangle};
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn test_stl_save_manual_mesh() {
    // Initialize library
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a simple triangle mesh manually
    let mut mesh = Mesh::new().expect("Failed to create mesh");

    // Add vertices for a simple triangle
    let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    let v1 = mesh.add_vertex(Vector3::new(10.0, 0.0, 0.0));
    let v2 = mesh.add_vertex(Vector3::new(5.0, 10.0, 0.0));

    // Add triangle
    mesh.add_triangle(Triangle::new(v0, v1, v2));

    println!(
        "Manual mesh has {} vertices and {} triangles",
        mesh.vertex_count(),
        mesh.triangle_count()
    );

    // Save to STL file
    let tmp = TempFolder::new().expect("Failed to create temp folder");
    let output_path = tmp.path().join("test_triangle.stl");
    mesh.save_stl(&output_path).expect("Failed to save STL");

    // Verify file exists and has correct size
    assert!(output_path.exists(), "STL file was not created");

    let metadata = fs::metadata(&output_path).expect("Failed to read file metadata");
    let expected_size = 80 + 4 + 50; // header + count + 1 triangle
    assert_eq!(
        metadata.len(),
        expected_size as u64,
        "STL file size mismatch for single triangle: expected {}, got {}",
        expected_size,
        metadata.len()
    );

    println!(
        "✓ Manual triangle STL saved successfully: {} bytes",
        metadata.len()
    );
}
