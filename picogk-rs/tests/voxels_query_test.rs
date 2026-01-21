use nalgebra::Vector3;
use picogk::{Library, Voxels};
use serial_test::serial;

#[test]
#[serial]
fn test_voxels_calculate_properties() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a sphere with radius 10mm
    let sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");

    // Calculate properties
    let (volume, bbox) = sphere.calculate_properties();
    let volume2 = sphere.volume_mm3();
    let bbox2 = sphere.bounding_box();

    assert!(
        (volume - volume2).abs() <= f32::EPSILON,
        "volume_mm3 mismatch: {} vs {}",
        volume,
        volume2
    );
    assert_eq!(bbox, bbox2, "bounding_box mismatch");

    println!("Sphere properties:");
    println!("  Volume: {:.2} mm³", volume);
    let min = bbox.min();
    let max = bbox.max();
    println!("  BBox min: ({:.2}, {:.2}, {:.2})", min.x, min.y, min.z);
    println!("  BBox max: ({:.2}, {:.2}, {:.2})", max.x, max.y, max.z);

    // Expected volume for sphere: 4/3 * π * r³ ≈ 4188.79 mm³
    let expected_volume = 4.0 / 3.0 * std::f32::consts::PI * 10.0_f32.powi(3);
    println!("  Expected volume: {:.2} mm³", expected_volume);

    // Volume should be within 10% of expected (voxelization introduces some error)
    assert!(
        volume > expected_volume * 0.9 && volume < expected_volume * 1.1,
        "Volume {} is not within 10% of expected {}",
        volume,
        expected_volume
    );

    // Bounding box should contain the sphere
    assert!(min.x < -9.0 && min.x > -11.0, "BBox min.x out of range");
    assert!(max.x > 9.0 && max.x < 11.0, "BBox max.x out of range");

    println!("✓ Calculate properties test passed");
}

#[test]
#[serial]
fn test_voxels_surface_normal() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a sphere centered at origin
    let sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");

    // Get surface normal at a point on the surface (approximately)
    let surface_point = Vector3::new(10.0, 0.0, 0.0);
    let normal = sphere.surface_normal(surface_point);

    println!(
        "Surface normal at ({:.2}, {:.2}, {:.2}): ({:.2}, {:.2}, {:.2})",
        surface_point.x, surface_point.y, surface_point.z, normal.x, normal.y, normal.z
    );

    // For a sphere centered at origin, the normal at (10, 0, 0) should point in +X direction
    assert!(
        normal.x > 0.9,
        "Normal X component should be close to 1.0, got {}",
        normal.x
    );
    assert!(
        normal.y.abs() < 0.1,
        "Normal Y component should be close to 0, got {}",
        normal.y
    );
    assert!(
        normal.z.abs() < 0.1,
        "Normal Z component should be close to 0, got {}",
        normal.z
    );

    println!("✓ Surface normal test passed");
}

#[test]
#[serial]
fn test_voxels_closest_point() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a sphere centered at origin
    let sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");

    // Find closest point from a point outside the sphere
    let search_point = Vector3::new(15.0, 0.0, 0.0);
    let closest = sphere.closest_point_on_surface(search_point);

    assert!(closest.is_some(), "Should find a closest point");

    let closest = closest.unwrap();
    println!(
        "Closest point to ({:.2}, {:.2}, {:.2}): ({:.2}, {:.2}, {:.2})",
        search_point.x, search_point.y, search_point.z, closest.x, closest.y, closest.z
    );

    // Closest point should be approximately at (10, 0, 0)
    assert!(
        (closest.x - 10.0).abs() < 1.0,
        "Closest X should be near 10.0, got {}",
        closest.x
    );
    assert!(
        closest.y.abs() < 1.0,
        "Closest Y should be near 0, got {}",
        closest.y
    );
    assert!(
        closest.z.abs() < 1.0,
        "Closest Z should be near 0, got {}",
        closest.z
    );

    println!("✓ Closest point test passed");
}

#[test]
#[serial]
fn test_voxels_raycast() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a sphere centered at origin
    let sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");

    // Cast a ray from outside towards the sphere
    let ray_origin = Vector3::new(20.0, 0.0, 0.0);
    let ray_direction = Vector3::new(-1.0, 0.0, 0.0);
    let hit = sphere.raycast_to_surface(ray_origin, ray_direction);

    assert!(hit.is_some(), "Ray should hit the sphere");

    let hit = hit.unwrap();
    println!("Ray hit at: ({:.2}, {:.2}, {:.2})", hit.x, hit.y, hit.z);

    // Hit point should be approximately at (10, 0, 0)
    assert!(
        (hit.x - 10.0).abs() < 1.0,
        "Hit X should be near 10.0, got {}",
        hit.x
    );
    assert!(hit.y.abs() < 1.0, "Hit Y should be near 0, got {}", hit.y);
    assert!(hit.z.abs() < 1.0, "Hit Z should be near 0, got {}", hit.z);

    // Test ray that misses
    let ray_origin2 = Vector3::new(20.0, 20.0, 0.0);
    let ray_direction2 = Vector3::new(-1.0, 0.0, 0.0);
    let hit2 = sphere.raycast_to_surface(ray_origin2, ray_direction2);

    assert!(hit2.is_none(), "Ray should miss the sphere");

    println!("✓ Raycast test passed");
}
