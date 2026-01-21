use nalgebra::Vector3;
use picogk::{Library, Utils};
use serial_test::serial;

#[test]
#[serial]
fn test_msh_create_geo_sphere_triangle_count() {
    let _lib = Library::init(0.5).expect("Failed to initialize library");

    let mesh_1 = Utils::msh_create_geo_sphere(Some(Vector3::new(10.0, 10.0, 10.0)), None, Some(1))
        .expect("Failed to create geosphere (subdivisions=1)");
    assert_eq!(mesh_1.triangle_count(), 80);

    let mesh_2 = Utils::msh_create_geo_sphere(Some(Vector3::new(10.0, 10.0, 10.0)), None, Some(2))
        .expect("Failed to create geosphere (subdivisions=2)");
    assert_eq!(mesh_2.triangle_count(), 320);
}
