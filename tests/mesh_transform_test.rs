use nalgebra::{Matrix4, Vector3};
use picogk::{Library, Mesh, Triangle};
use serial_test::serial;

#[test]
#[serial]
fn test_mesh_transform_scale_offset() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a simple triangle mesh
    let mut mesh = Mesh::new().expect("Failed to create mesh");
    let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    let v1 = mesh.add_vertex(Vector3::new(1.0, 0.0, 0.0));
    let v2 = mesh.add_vertex(Vector3::new(0.0, 1.0, 0.0));
    mesh.add_triangle(Triangle::new(v0, v1, v2));

    // Transform: scale 2x and offset by (10, 0, 0)
    let transformed = mesh
        .create_transformed(Vector3::new(2.0, 2.0, 2.0), Vector3::new(10.0, 0.0, 0.0))
        .expect("Failed to transform mesh");

    assert_eq!(transformed.triangle_count(), 1);
    assert_eq!(transformed.vertex_count(), 3);

    // Get the transformed vertices
    let (a, b, c) = transformed
        .get_triangle_vertices(0)
        .expect("Failed to get triangle");

    // Check scaled and offset vertices
    assert!((a.x - 10.0).abs() < 0.001); // 0*2 + 10 = 10
    assert!((a.y - 0.0).abs() < 0.001); // 0*2 + 0 = 0

    assert!((b.x - 12.0).abs() < 0.001); // 1*2 + 10 = 12
    assert!((b.y - 0.0).abs() < 0.001); // 0*2 + 0 = 0

    assert!((c.x - 10.0).abs() < 0.001); // 0*2 + 10 = 10
    assert!((c.y - 2.0).abs() < 0.001); // 1*2 + 0 = 2

    println!("✓ Mesh transform (scale + offset) test passed");
}

#[test]
#[serial]
fn test_mesh_transform_matrix() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a simple triangle mesh
    let mut mesh = Mesh::new().expect("Failed to create mesh");
    let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    let v1 = mesh.add_vertex(Vector3::new(1.0, 0.0, 0.0));
    let v2 = mesh.add_vertex(Vector3::new(0.0, 1.0, 0.0));
    mesh.add_triangle(Triangle::new(v0, v1, v2));

    // Create translation matrix
    let matrix = Matrix4::new_translation(&Vector3::new(5.0, 10.0, 0.0));
    let transformed = mesh
        .create_transformed_matrix(&matrix)
        .expect("Failed to transform mesh");

    assert_eq!(transformed.triangle_count(), 1);
    assert_eq!(transformed.vertex_count(), 3);

    let (a, b, c) = transformed
        .get_triangle_vertices(0)
        .expect("Failed to get triangle");

    // Check translated vertices
    assert!((a.x - 5.0).abs() < 0.001);
    assert!((a.y - 10.0).abs() < 0.001);

    assert!((b.x - 6.0).abs() < 0.001);
    assert!((b.y - 10.0).abs() < 0.001);

    assert!((c.x - 5.0).abs() < 0.001);
    assert!((c.y - 11.0).abs() < 0.001);

    println!("✓ Mesh transform (matrix) test passed");
}

#[test]
#[serial]
fn test_mesh_mirror() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a simple triangle mesh
    let mut mesh = Mesh::new().expect("Failed to create mesh");
    let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 1.0));
    let v1 = mesh.add_vertex(Vector3::new(1.0, 0.0, 2.0));
    let v2 = mesh.add_vertex(Vector3::new(0.0, 1.0, 3.0));
    mesh.add_triangle(Triangle::new(v0, v1, v2));

    // Mirror across XY plane (Z=0)
    let mirrored = mesh
        .create_mirrored(Vector3::zeros(), Vector3::new(0.0, 0.0, 1.0))
        .expect("Failed to mirror mesh");

    assert_eq!(mirrored.triangle_count(), 1);
    assert_eq!(mirrored.vertex_count(), 3);

    let (a, b, c) = mirrored
        .get_triangle_vertices(0)
        .expect("Failed to get triangle");

    // Check mirrored vertices (X and Y unchanged, Z negated)
    assert!((a.x - 0.0).abs() < 0.001);
    assert!((a.y - 0.0).abs() < 0.001);
    assert!((a.z - (-1.0)).abs() < 0.001);

    assert!((b.x - 1.0).abs() < 0.001);
    assert!((b.y - 0.0).abs() < 0.001);
    assert!((b.z - (-2.0)).abs() < 0.001);

    assert!((c.x - 0.0).abs() < 0.001);
    assert!((c.y - 1.0).abs() < 0.001);
    assert!((c.z - (-3.0)).abs() < 0.001);

    println!("✓ Mesh mirror test passed");
}

#[test]
#[serial]
fn test_mesh_append() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create first mesh
    let mut mesh1 = Mesh::new().expect("Failed to create mesh1");
    let v0 = mesh1.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    let v1 = mesh1.add_vertex(Vector3::new(1.0, 0.0, 0.0));
    let v2 = mesh1.add_vertex(Vector3::new(0.0, 1.0, 0.0));
    mesh1.add_triangle(Triangle::new(v0, v1, v2));

    // Create second mesh
    let mut mesh2 = Mesh::new().expect("Failed to create mesh2");
    let v0 = mesh2.add_vertex(Vector3::new(2.0, 0.0, 0.0));
    let v1 = mesh2.add_vertex(Vector3::new(3.0, 0.0, 0.0));
    let v2 = mesh2.add_vertex(Vector3::new(2.0, 1.0, 0.0));
    mesh2.add_triangle(Triangle::new(v0, v1, v2));

    // Append mesh2 to mesh1
    mesh1.append(&mesh2).expect("Failed to append mesh");

    assert_eq!(mesh1.triangle_count(), 2);
    assert_eq!(mesh1.vertex_count(), 6);

    println!("✓ Mesh append test passed");
}
