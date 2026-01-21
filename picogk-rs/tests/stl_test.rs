use nalgebra::Vector3;
use picogk::{Library, Mesh, TempFolder, Voxels};
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn test_stl_save() {
    // Initialize library
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a simple sphere
    let sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");

    // Convert to mesh
    let mesh = sphere.as_mesh().expect("Failed to convert to mesh");

    println!(
        "Mesh has {} vertices and {} triangles",
        mesh.vertex_count(),
        mesh.triangle_count()
    );

    // Save to STL file
    let tmp = TempFolder::new().expect("Failed to create temp folder");
    let output_path = tmp.path().join("test_sphere.stl");
    mesh.save_stl(&output_path).expect("Failed to save STL");

    // Verify file exists
    assert!(output_path.exists(), "STL file was not created");

    // Check file size is reasonable (should have header + triangle data)
    let metadata = fs::metadata(&output_path).expect("Failed to read file metadata");
    let expected_min_size = 80 + 4; // header + triangle count
    assert!(
        metadata.len() >= expected_min_size,
        "STL file is too small: {} bytes",
        metadata.len()
    );

    // Each triangle is 50 bytes (12 floats * 4 bytes + 2 bytes attribute)
    let expected_size = 80 + 4 + (mesh.triangle_count() * 50);
    assert_eq!(
        metadata.len(),
        expected_size as u64,
        "STL file size mismatch: expected {}, got {}",
        expected_size,
        metadata.len()
    );

    println!("✓ STL file saved successfully: {} bytes", metadata.len());
}

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
    mesh.add_triangle(picogk::Triangle::new(v0, v1, v2));

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
    let metadata = fs::metadata(&output_path).expect("Failed to read file metadata");
    let expected_size = 80 + 4 + 50; // header + count + 1 triangle
    assert_eq!(
        metadata.len(),
        expected_size as u64,
        "STL file size mismatch for single triangle"
    );

    println!(
        "✓ Manual triangle STL saved successfully: {} bytes",
        metadata.len()
    );
}
