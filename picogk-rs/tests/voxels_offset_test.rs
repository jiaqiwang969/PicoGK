use nalgebra::Vector3;
use picogk::{Library, Voxels};
use serial_test::serial;

#[test]
#[serial]
fn test_voxels_triple_offset() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a sphere
    let mut sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");

    // Apply triple offset (smoothing)
    sphere.triple_offset(1.0);

    // Verify the voxels are still valid
    assert!(
        sphere.is_valid(),
        "Voxels should be valid after triple offset"
    );

    println!("✓ Triple offset test passed");
}

#[test]
#[serial]
fn test_voxels_smoothen() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a sphere
    let mut sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");

    // Apply smoothen (which calls triple_offset)
    sphere.smoothen(1.0);

    // Verify the voxels are still valid
    assert!(sphere.is_valid(), "Voxels should be valid after smoothen");

    println!("✓ Smoothen test passed");
}

#[test]
#[serial]
fn test_voxels_over_offset() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a sphere
    let mut sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");

    // Apply over offset
    sphere.over_offset(2.0, 0.0);

    // Verify the voxels are still valid
    assert!(
        sphere.is_valid(),
        "Voxels should be valid after over offset"
    );

    println!("✓ Over offset test passed");
}

#[test]
#[serial]
fn test_voxels_fillet() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a sphere
    let mut sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");

    // Apply fillet
    sphere.fillet(1.5);

    // Verify the voxels are still valid
    assert!(sphere.is_valid(), "Voxels should be valid after fillet");

    println!("✓ Fillet test passed");
}
