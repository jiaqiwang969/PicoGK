use nalgebra::Vector3;
use picogk::{Library, Mesh, Triangle};
use serial_test::serial;

#[test]
#[serial]
fn test_mesh_bounding_box() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    // Create a simple mesh with known bounds
    let mut mesh = Mesh::new().expect("Failed to create mesh");

    // Add vertices at specific positions
    let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    let v1 = mesh.add_vertex(Vector3::new(10.0, 0.0, 0.0));
    let v2 = mesh.add_vertex(Vector3::new(5.0, 10.0, 0.0));
    let v3 = mesh.add_vertex(Vector3::new(5.0, 5.0, 5.0));

    // Add triangles
    mesh.add_triangle(Triangle::new(v0, v1, v2));
    mesh.add_triangle(Triangle::new(v0, v1, v3));

    println!(
        "Mesh has {} vertices and {} triangles",
        mesh.vertex_count(),
        mesh.triangle_count()
    );

    // Get bounding box
    let bbox = mesh.bounding_box();
    let min = bbox.min();
    let max = bbox.max();

    println!("BBox min: ({:.2}, {:.2}, {:.2})", min.x, min.y, min.z);
    println!("BBox max: ({:.2}, {:.2}, {:.2})", max.x, max.y, max.z);

    // Verify bounding box contains all vertices
    assert!(min.x <= 0.0 && max.x >= 10.0, "X bounds incorrect");
    assert!(min.y <= 0.0 && max.y >= 10.0, "Y bounds incorrect");
    assert!(min.z <= 0.0 && max.z >= 5.0, "Z bounds incorrect");

    println!("✓ Mesh bounding box test passed");
}
