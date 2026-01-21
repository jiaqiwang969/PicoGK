use nalgebra::Vector3;
use picogk::{FieldType, Library, TempFolder, VdbFile, Voxels};
use serial_test::serial;

#[test]
#[serial]
fn test_vdb_save_and_load() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");
    let tmp = TempFolder::new().expect("Failed to create temp folder");

    // Create a sphere
    let sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");

    // Save to VDB
    let path = tmp.path().join("test_sphere.vdb");
    sphere.save_vdb(&path).expect("Failed to save VDB");

    // Load from VDB
    let loaded = Voxels::load_vdb(&path).expect("Failed to load VDB");

    // Verify loaded voxels are valid
    assert!(loaded.is_valid(), "Loaded voxels should be valid");

    println!("✓ VDB save and load test passed");
}

#[test]
#[serial]
fn test_vdb_file_multiple_fields() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");
    let tmp = TempFolder::new().expect("Failed to create temp folder");

    // Create two spheres
    let sphere1 = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere1");
    let sphere2 =
        Voxels::sphere(Vector3::new(5.0, 0.0, 0.0), 8.0).expect("Failed to create sphere2");

    // Create VDB file and add both spheres
    let mut vdb = VdbFile::new().expect("Failed to create VDB file");
    let idx1 = vdb
        .add_voxels(&sphere1, "sphere1")
        .expect("Failed to add sphere1");
    let idx2 = vdb
        .add_voxels(&sphere2, "sphere2")
        .expect("Failed to add sphere2");

    println!("Added sphere1 at index {}", idx1);
    println!("Added sphere2 at index {}", idx2);

    // Save to file
    let path = tmp.path().join("test_multi.vdb");
    vdb.save(&path).expect("Failed to save VDB");

    // Load from file
    let loaded_vdb = VdbFile::load(&path).expect("Failed to load VDB");

    // Verify field count
    assert_eq!(loaded_vdb.field_count(), 2, "Should have 2 fields");

    // Verify field types
    assert_eq!(
        loaded_vdb.field_type(0),
        FieldType::Voxels,
        "Field 0 should be Voxels"
    );
    assert_eq!(
        loaded_vdb.field_type(1),
        FieldType::Voxels,
        "Field 1 should be Voxels"
    );

    // Get field names
    let name0 = loaded_vdb.field_name(0);
    let name1 = loaded_vdb.field_name(1);
    println!("Field 0: {}", name0);
    println!("Field 1: {}", name1);

    // Load voxels by index
    let vox0 = loaded_vdb.get_voxels(0).expect("Failed to get voxels 0");
    let vox1 = loaded_vdb.get_voxels(1).expect("Failed to get voxels 1");

    assert!(vox0.is_valid(), "Voxels 0 should be valid");
    assert!(vox1.is_valid(), "Voxels 1 should be valid");

    // Load voxels by name
    let vox_by_name = loaded_vdb
        .get_voxels_by_name(&name0)
        .expect("Failed to get voxels by name");
    assert!(vox_by_name.is_valid(), "Voxels by name should be valid");

    println!("✓ VDB multiple fields test passed");
}

#[test]
#[serial]
fn test_vdb_roundtrip() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");
    let tmp = TempFolder::new().expect("Failed to create temp folder");

    // Create a complex shape
    let sphere1 = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere1");
    let sphere2 =
        Voxels::sphere(Vector3::new(8.0, 0.0, 0.0), 10.0).expect("Failed to create sphere2");
    let union = sphere1
        .vox_bool_add(&sphere2)
        .expect("Failed to perform union");

    // Save to VDB
    let path = tmp.path().join("test_roundtrip.vdb");
    union.save_vdb(&path).expect("Failed to save VDB");

    // Load from VDB
    let loaded = Voxels::load_vdb(&path).expect("Failed to load VDB");

    // Verify loaded voxels are valid
    assert!(loaded.is_valid(), "Loaded voxels should be valid");

    // Convert to mesh to verify geometry
    let mesh = loaded.as_mesh().expect("Failed to convert to mesh");
    assert!(mesh.vertex_count() > 0, "Mesh should have vertices");
    assert!(mesh.triangle_count() > 0, "Mesh should have triangles");

    println!(
        "Loaded mesh: {} vertices, {} triangles",
        mesh.vertex_count(),
        mesh.triangle_count()
    );

    println!("✓ VDB roundtrip test passed");
}

#[test]
#[serial]
fn test_vdb_save_stamps_picogk_metadata() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");
    let tmp = TempFolder::new().expect("Failed to create temp folder");

    let sphere = Voxels::sphere(Vector3::zeros(), 10.0).expect("Failed to create sphere");
    let mut vdb = VdbFile::new().expect("Failed to create VDB file");
    vdb.add_voxels(&sphere, "sphere")
        .expect("Failed to add voxels");

    let path = tmp.path().join("test_metadata.vdb");
    vdb.save(&path).expect("Failed to save VDB");

    let loaded = VdbFile::load(&path).expect("Failed to load VDB");
    assert!(
        loaded.is_pico_gk_compatible(),
        "Saved VDB should be PicoGK-compatible"
    );

    let vsize = loaded.pico_gk_voxel_size_mm();
    assert!(
        (vsize - Library::voxel_size_mm()).abs() < 1e-6,
        "Voxel size from metadata should match Library voxel size"
    );

    let field0 = loaded.get_voxels(0).expect("Failed to load field 0 voxels");
    let md = field0.metadata().expect("Failed to read field metadata");
    assert_eq!(
        md.get_string("PicoGK.Library")
            .expect("Failed to read PicoGK.Library")
            .as_deref(),
        Some(Library::name().as_str())
    );
    assert_eq!(
        md.get_string("PicoGK.Version")
            .expect("Failed to read PicoGK.Version")
            .as_deref(),
        Some(Library::version().as_str())
    );
    let v_m = md
        .get_float("PicoGK.VoxelSize")
        .expect("Failed to read PicoGK.VoxelSize")
        .unwrap_or(0.0);
    assert!(
        (v_m * 1000.0 - Library::voxel_size_mm()).abs() < 1e-6,
        "Voxel size stored in VDB should be in meters (converted back to mm)"
    );
}
