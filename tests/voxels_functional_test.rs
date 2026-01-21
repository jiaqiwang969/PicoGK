use nalgebra::Vector3;
use picogk::{Library, Voxels};
use serial_test::serial;

#[test]
#[serial]
fn test_functional_bool_operations() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create two spheres
    let sphere1 = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere1");
    let sphere2 =
        Voxels::sphere(Vector3::new(5.0, 0.0, 0.0), 10.0).expect("Failed to create sphere2");

    // Test functional boolean union
    let union = sphere1
        .vox_bool_add(&sphere2)
        .expect("Failed to perform union");
    assert!(union.is_valid(), "Union result should be valid");

    // Test functional boolean difference
    let difference = sphere1
        .vox_bool_subtract(&sphere2)
        .expect("Failed to perform difference");
    assert!(difference.is_valid(), "Difference result should be valid");

    // Test functional boolean intersection
    let intersection = sphere1
        .vox_bool_intersect(&sphere2)
        .expect("Failed to perform intersection");
    assert!(
        intersection.is_valid(),
        "Intersection result should be valid"
    );

    // Verify original spheres are unchanged
    assert!(sphere1.is_valid(), "Original sphere1 should still be valid");
    assert!(sphere2.is_valid(), "Original sphere2 should still be valid");

    println!("✓ Functional boolean operations test passed");
}

#[test]
#[serial]
fn test_functional_offset_operations() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a sphere
    let sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");

    // Test functional offset
    let offset = sphere.vox_offset(2.0).expect("Failed to perform offset");
    assert!(offset.is_valid(), "Offset result should be valid");

    // Test functional double offset
    let double_offset = sphere
        .vox_double_offset(2.0, -1.0)
        .expect("Failed to perform double offset");
    assert!(
        double_offset.is_valid(),
        "Double offset result should be valid"
    );

    // Test functional triple offset
    let triple_offset = sphere
        .vox_triple_offset(1.0)
        .expect("Failed to perform triple offset");
    assert!(
        triple_offset.is_valid(),
        "Triple offset result should be valid"
    );

    // Test functional smoothen
    let smoothed = sphere
        .vox_smoothen(1.0)
        .expect("Failed to perform smoothen");
    assert!(smoothed.is_valid(), "Smoothen result should be valid");

    // Test functional over offset
    let over_offset = sphere
        .vox_over_offset(2.0, 0.5)
        .expect("Failed to perform over offset");
    assert!(over_offset.is_valid(), "Over offset result should be valid");

    // Test functional fillet
    let filleted = sphere.vox_fillet(1.5).expect("Failed to perform fillet");
    assert!(filleted.is_valid(), "Fillet result should be valid");

    // Verify original sphere is unchanged
    assert!(sphere.is_valid(), "Original sphere should still be valid");

    println!("✓ Functional offset operations test passed");
}

#[test]
#[serial]
fn test_functional_chaining() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create two spheres
    let sphere1 = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere1");
    let sphere2 =
        Voxels::sphere(Vector3::new(5.0, 0.0, 0.0), 10.0).expect("Failed to create sphere2");

    // Chain functional operations
    let result = sphere1
        .vox_bool_add(&sphere2)
        .expect("Failed to add")
        .vox_offset(1.0)
        .expect("Failed to offset")
        .vox_smoothen(0.5)
        .expect("Failed to smoothen");

    assert!(result.is_valid(), "Chained result should be valid");

    // Verify original spheres are unchanged
    assert!(sphere1.is_valid(), "Original sphere1 should still be valid");
    assert!(sphere2.is_valid(), "Original sphere2 should still be valid");

    println!("✓ Functional chaining test passed");
}
