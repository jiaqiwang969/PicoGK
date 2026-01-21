use nalgebra::Vector3;
use picogk::{Library, Mesh, Triangle};
use serial_test::serial;

#[test]
#[serial]
fn test_mesh_creation_only() {
    println!("Initializing library...");
    let _lib = Library::init(0.5).expect("Failed to initialize library");
    println!("✓ Library initialized");

    println!("Creating mesh...");
    let mut mesh = Mesh::new().expect("Failed to create mesh");
    println!("✓ Mesh created");

    println!("Adding vertices...");
    let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    println!("  v0 = {}", v0);
    let v1 = mesh.add_vertex(Vector3::new(10.0, 0.0, 0.0));
    println!("  v1 = {}", v1);
    let v2 = mesh.add_vertex(Vector3::new(5.0, 10.0, 0.0));
    println!("  v2 = {}", v2);
    println!("✓ Vertices added");

    println!("Adding triangle...");
    let tri_idx = mesh.add_triangle(Triangle::new(v0, v1, v2));
    println!("  triangle index = {}", tri_idx);
    println!("✓ Triangle added");

    println!(
        "Mesh has {} vertices and {} triangles",
        mesh.vertex_count(),
        mesh.triangle_count()
    );

    println!("Reading back vertices...");
    if let Some(vertex) = mesh.get_vertex(0) {
        println!("  v0 = {:?}", vertex);
    }
    if let Some(vertex) = mesh.get_vertex(1) {
        println!("  v1 = {:?}", vertex);
    }
    if let Some(vertex) = mesh.get_vertex(2) {
        println!("  v2 = {:?}", vertex);
    }
    println!("✓ Vertices read back");

    println!("Reading back triangle...");
    if let Some(tri) = mesh.get_triangle(0) {
        println!("  triangle = ({}, {}, {})", tri.v0, tri.v1, tri.v2);
    }
    println!("✓ Triangle read back");

    println!("✓ All mesh operations successful");
}
