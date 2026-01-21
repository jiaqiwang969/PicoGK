//! Simple test to diagnose issues

use nalgebra::Vector3;
use picogk::{Lattice, Library, Voxels};
use serial_test::serial;

#[test]
#[serial]
fn test_basic_flow() {
    // Initialize library once
    let _lib = Library::init(0.5).expect("Failed to init library");

    println!("Library initialized");

    // Test lattice creation
    let mut lattice = Lattice::new().expect("Failed to create lattice");
    println!("Lattice created");

    lattice.add_sphere(Vector3::zeros(), 10.0);
    println!("Sphere added to lattice");

    // Test voxels from lattice
    let vox = Voxels::from_lattice(&lattice).expect("Failed to create voxels");
    println!("Voxels created from lattice");

    assert!(vox.is_valid());
    println!("Test passed!");
}
