//! Integration tests for PicoGK Rust bindings
//!
//! These tests verify that the implemented functionality works correctly
//! with the actual C++ library.

use nalgebra::Vector3;
use picogk::{Lattice, Library, Mesh, Voxels};
use serial_test::serial;

/// Test library initialization
#[test]
#[serial]
fn test_library_init() {
    let lib = Library::init(0.5);
    assert!(lib.is_ok(), "Library initialization failed");
}

/// Test library name and version
#[test]
#[serial]
fn test_library_info() {
    let _lib = Library::init(0.5).unwrap();

    let name = Library::name();
    assert!(!name.is_empty(), "Library name is empty");
    println!("Library name: {}", name);

    let version = Library::version();
    assert!(!version.is_empty(), "Library version is empty");
    println!("Library version: {}", version);
}

/// Test invalid voxel size
#[test]
#[serial]
fn test_invalid_voxel_size() {
    let result = Library::init(-1.0);
    assert!(result.is_err(), "Should reject negative voxel size");

    let result = Library::init(0.0);
    assert!(result.is_err(), "Should reject zero voxel size");
}

/// Test empty voxels creation
#[test]
#[serial]
fn test_empty_voxels() {
    let _lib = Library::init(0.5).unwrap();
    let vox = Voxels::new();
    assert!(vox.is_ok(), "Failed to create empty voxels");
    assert!(vox.unwrap().is_valid(), "Empty voxels should be valid");
}

/// Test sphere creation
#[test]
#[serial]
fn test_sphere_creation() {
    let _lib = Library::init(0.5).unwrap();

    let sphere = Voxels::sphere(Vector3::zeros(), 20.0);
    assert!(sphere.is_ok(), "Failed to create sphere");
    assert!(sphere.unwrap().is_valid(), "Sphere should be valid");
}

/// Test invalid sphere parameters
#[test]
#[serial]
fn test_invalid_sphere() {
    let _lib = Library::init(0.5).unwrap();

    let result = Voxels::sphere(Vector3::zeros(), -1.0);
    assert!(result.is_err(), "Should reject negative radius");

    let result = Voxels::sphere(Vector3::zeros(), 0.0);
    assert!(result.is_err(), "Should reject zero radius");
}

/// Test lattice creation
#[test]
#[serial]
fn test_lattice_creation() {
    let _lib = Library::init(0.5).unwrap();

    let lattice = Lattice::new();
    assert!(lattice.is_ok(), "Failed to create lattice");
    assert!(lattice.unwrap().is_valid(), "Lattice should be valid");
}

/// Test lattice with sphere
#[test]
#[serial]
fn test_lattice_sphere() {
    let _lib = Library::init(0.5).unwrap();

    let mut lattice = Lattice::new().unwrap();
    lattice.add_sphere(Vector3::zeros(), 10.0);
    assert!(lattice.is_valid(), "Lattice with sphere should be valid");
}

/// Test lattice with beam
#[test]
#[serial]
fn test_lattice_beam() {
    let _lib = Library::init(0.5).unwrap();

    let mut lattice = Lattice::new().unwrap();
    lattice.add_beam(
        Vector3::new(-10.0, 0.0, 0.0),
        Vector3::new(10.0, 0.0, 0.0),
        2.0,
        2.0,
    );
    assert!(lattice.is_valid(), "Lattice with beam should be valid");
}

/// Test cubic lattice
#[test]
#[serial]
fn test_cubic_lattice() {
    let _lib = Library::init(0.5).unwrap();

    let lattice = Lattice::cubic(3, 10.0, 1.5, 0.8);
    assert!(lattice.is_ok(), "Failed to create cubic lattice");
    assert!(lattice.unwrap().is_valid(), "Cubic lattice should be valid");
}

/// Test voxels from lattice
#[test]
#[serial]
fn test_voxels_from_lattice() {
    let _lib = Library::init(0.5).unwrap();

    let mut lattice = Lattice::new().unwrap();
    lattice.add_sphere(Vector3::zeros(), 10.0);

    let vox = Voxels::from_lattice(&lattice);
    assert!(vox.is_ok(), "Failed to create voxels from lattice");
    assert!(
        vox.unwrap().is_valid(),
        "Voxels from lattice should be valid"
    );
}

/// Test mesh creation
#[test]
#[serial]
fn test_mesh_creation() {
    let _lib = Library::init(0.5).unwrap();

    let mesh = Mesh::new();
    assert!(mesh.is_ok(), "Failed to create mesh");
    assert!(mesh.unwrap().is_valid(), "Mesh should be valid");
}

/// Test mesh from voxels
#[test]
#[serial]
fn test_mesh_from_voxels() {
    let _lib = Library::init(0.5).unwrap();

    let sphere = Voxels::sphere(Vector3::zeros(), 20.0).unwrap();
    let mesh = Mesh::from_voxels(&sphere);
    assert!(mesh.is_ok(), "Failed to create mesh from voxels");

    let mesh = mesh.unwrap();
    assert!(mesh.is_valid(), "Mesh from voxels should be valid");
    assert!(mesh.vertex_count() > 0, "Mesh should have vertices");
    assert!(mesh.triangle_count() > 0, "Mesh should have triangles");

    println!(
        "Mesh has {} vertices and {} triangles",
        mesh.vertex_count(),
        mesh.triangle_count()
    );
}

/// Test voxels to mesh conversion
#[test]
#[serial]
fn test_voxels_as_mesh() {
    let _lib = Library::init(0.5).unwrap();

    let sphere = Voxels::sphere(Vector3::zeros(), 20.0).unwrap();
    let mesh = sphere.as_mesh();
    assert!(mesh.is_ok(), "Failed to convert voxels to mesh");

    let mesh = mesh.unwrap();
    assert!(mesh.is_valid(), "Converted mesh should be valid");
    assert!(
        mesh.vertex_count() > 0,
        "Converted mesh should have vertices"
    );
}

/// Test mesh vertex operations
#[test]
#[serial]
fn test_mesh_vertices() {
    let _lib = Library::init(0.5).unwrap();

    let mut mesh = Mesh::new().unwrap();

    let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    let v1 = mesh.add_vertex(Vector3::new(1.0, 0.0, 0.0));
    let v2 = mesh.add_vertex(Vector3::new(0.0, 1.0, 0.0));

    assert_eq!(v0, 0, "First vertex should have index 0");
    assert_eq!(v1, 1, "Second vertex should have index 1");
    assert_eq!(v2, 2, "Third vertex should have index 2");
    assert_eq!(mesh.vertex_count(), 3, "Should have 3 vertices");

    let vertex = mesh.get_vertex(0);
    assert!(vertex.is_some(), "Should be able to get vertex 0");
    assert_eq!(vertex.unwrap(), Vector3::new(0.0, 0.0, 0.0));
}

/// Test mesh triangle operations
#[test]
#[serial]
fn test_mesh_triangles() {
    let _lib = Library::init(0.5).unwrap();

    let mut mesh = Mesh::new().unwrap();

    let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    let v1 = mesh.add_vertex(Vector3::new(1.0, 0.0, 0.0));
    let v2 = mesh.add_vertex(Vector3::new(0.0, 1.0, 0.0));

    let tri = picogk::Triangle::new(v0, v1, v2);
    let t0 = mesh.add_triangle(tri);

    assert_eq!(t0, 0, "First triangle should have index 0");
    assert_eq!(mesh.triangle_count(), 1, "Should have 1 triangle");

    let triangle = mesh.get_triangle(0);
    assert!(triangle.is_some(), "Should be able to get triangle 0");
    assert_eq!(triangle.unwrap().v0, v0);
    assert_eq!(triangle.unwrap().v1, v1);
    assert_eq!(triangle.unwrap().v2, v2);
}

/// Test boolean union
#[test]
#[serial]
fn test_bool_add() {
    let _lib = Library::init(0.5).unwrap();

    let mut vox1 = Voxels::sphere(Vector3::new(-5.0, 0.0, 0.0), 10.0).unwrap();
    let vox2 = Voxels::sphere(Vector3::new(5.0, 0.0, 0.0), 10.0).unwrap();

    vox1.bool_add(&vox2);
    assert!(vox1.is_valid(), "Union result should be valid");
}

/// Test boolean subtraction
#[test]
#[serial]
fn test_bool_subtract() {
    let _lib = Library::init(0.5).unwrap();

    let mut vox1 = Voxels::sphere(Vector3::zeros(), 20.0).unwrap();
    let vox2 = Voxels::sphere(Vector3::new(10.0, 0.0, 0.0), 15.0).unwrap();

    vox1.bool_subtract(&vox2);
    assert!(vox1.is_valid(), "Subtraction result should be valid");
}

/// Test boolean intersection
#[test]
#[serial]
fn test_bool_intersect() {
    let _lib = Library::init(0.5).unwrap();

    let mut vox1 = Voxels::sphere(Vector3::new(-5.0, 0.0, 0.0), 15.0).unwrap();
    let vox2 = Voxels::sphere(Vector3::new(5.0, 0.0, 0.0), 15.0).unwrap();

    vox1.bool_intersect(&vox2);
    assert!(vox1.is_valid(), "Intersection result should be valid");
}

/// Test offset operation
#[test]
#[serial]
fn test_offset() {
    let _lib = Library::init(0.5).unwrap();

    let mut sphere = Voxels::sphere(Vector3::zeros(), 20.0).unwrap();
    sphere.offset(5.0);
    assert!(sphere.is_valid(), "Offset result should be valid");
}

/// Test double offset
#[test]
#[serial]
fn test_double_offset() {
    let _lib = Library::init(0.5).unwrap();

    let mut sphere = Voxels::sphere(Vector3::zeros(), 20.0).unwrap();
    sphere.double_offset(5.0, -3.0);
    assert!(sphere.is_valid(), "Double offset result should be valid");
}

/// Test smoothen operation
#[test]
#[serial]
fn test_smoothen() {
    let _lib = Library::init(0.5).unwrap();

    let mut sphere = Voxels::sphere(Vector3::zeros(), 20.0).unwrap();
    sphere.smoothen(2.0);
    assert!(sphere.is_valid(), "Smoothen result should be valid");
}

/// Test shell creation
#[test]
#[serial]
fn test_shell() {
    let _lib = Library::init(0.5).unwrap();

    let sphere = Voxels::sphere(Vector3::zeros(), 20.0).unwrap();
    let shell = sphere.shell(3.0);
    assert!(shell.is_ok(), "Failed to create shell");
    assert!(shell.unwrap().is_valid(), "Shell should be valid");
}

/// Test invalid shell thickness
#[test]
#[serial]
fn test_invalid_shell() {
    let _lib = Library::init(0.5).unwrap();

    let sphere = Voxels::sphere(Vector3::zeros(), 20.0).unwrap();
    let result = sphere.shell(-1.0);
    assert!(result.is_ok(), "Negative offset should be allowed");

    let result = sphere.shell(0.0);
    assert!(result.is_ok(), "Zero offset should be allowed");
}

/// Test voxels duplication
#[test]
#[serial]
fn test_voxels_duplicate() {
    let _lib = Library::init(0.5).unwrap();

    let sphere = Voxels::sphere(Vector3::zeros(), 20.0).unwrap();
    let duplicate = sphere.duplicate();
    assert!(duplicate.is_ok(), "Failed to duplicate voxels");
    assert!(duplicate.unwrap().is_valid(), "Duplicate should be valid");
}

/// Test voxels fallible clone
#[test]
#[serial]
fn test_voxels_clone() {
    let _lib = Library::init(0.5).unwrap();

    let sphere = Voxels::sphere(Vector3::zeros(), 20.0).unwrap();
    let cloned = sphere.try_clone().unwrap();
    assert!(cloned.is_valid(), "Cloned voxels should be valid");
}

/// Test complex lattice structure
#[test]
#[serial]
fn test_complex_lattice() {
    let _lib = Library::init(0.5).unwrap();

    let mut lattice = Lattice::new().unwrap();

    // Add central sphere
    lattice.add_sphere(Vector3::zeros(), 5.0);

    // Add radial beams
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::PI / 3.0;
        let dir = Vector3::new(angle.cos(), angle.sin(), 0.0) * 20.0;
        lattice.add_beam(Vector3::zeros(), dir, 3.0, 1.0);
    }

    let vox = Voxels::from_lattice(&lattice);
    assert!(vox.is_ok(), "Failed to create voxels from complex lattice");

    let mesh = vox.unwrap().as_mesh();
    assert!(mesh.is_ok(), "Failed to convert complex lattice to mesh");
}

/// Test complex boolean operations
#[test]
#[serial]
fn test_complex_boolean() {
    let _lib = Library::init(0.5).unwrap();

    // Create three overlapping spheres
    let mut vox1 = Voxels::sphere(Vector3::new(-10.0, 0.0, 0.0), 15.0).unwrap();
    let vox2 = Voxels::sphere(Vector3::new(10.0, 0.0, 0.0), 15.0).unwrap();
    let vox3 = Voxels::sphere(Vector3::new(0.0, 10.0, 0.0), 15.0).unwrap();

    // Union all three
    vox1.bool_add(&vox2);
    vox1.bool_add(&vox3);

    assert!(vox1.is_valid(), "Complex union should be valid");

    // Convert to mesh
    let mesh = vox1.as_mesh();
    assert!(mesh.is_ok(), "Should be able to mesh complex union");
}

/// Test offset and boolean combination
#[test]
#[serial]
fn test_offset_and_boolean() {
    let _lib = Library::init(0.5).unwrap();

    let sphere1 = Voxels::sphere(Vector3::zeros(), 20.0).unwrap();
    let mut sphere2 = sphere1.duplicate().unwrap();

    // Create a shell by offsetting and subtracting
    sphere2.offset(-5.0);
    let mut shell = sphere1.duplicate().unwrap();
    shell.bool_subtract(&sphere2);

    assert!(shell.is_valid(), "Shell from offset should be valid");
}
