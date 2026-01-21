use nalgebra::Vector3;
use picogk::{Library, Voxels};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize library
    let _lib = Library::init(0.5)?;

    // Create a sphere
    let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;
    println!("Created sphere with radius 10mm");

    // Test calculate_properties
    let (volume, bbox) = sphere.calculate_properties();
    println!("\nCalculate Properties:");
    println!("  Volume: {:.2} mm³", volume);
    let min = bbox.min();
    let max = bbox.max();
    println!("  BBox min: ({:.2}, {:.2}, {:.2})", min.x, min.y, min.z);
    println!("  BBox max: ({:.2}, {:.2}, {:.2})", max.x, max.y, max.z);

    // Test surface_normal
    let surface_point = Vector3::new(10.0, 0.0, 0.0);
    let normal = sphere.surface_normal(surface_point);
    println!(
        "\nSurface Normal at ({:.2}, {:.2}, {:.2}):",
        surface_point.x, surface_point.y, surface_point.z
    );
    println!(
        "  Normal: ({:.2}, {:.2}, {:.2})",
        normal.x, normal.y, normal.z
    );

    // Test closest_point_on_surface
    let search_point = Vector3::new(15.0, 0.0, 0.0);
    if let Some(closest) = sphere.closest_point_on_surface(search_point) {
        println!(
            "\nClosest Point to ({:.2}, {:.2}, {:.2}):",
            search_point.x, search_point.y, search_point.z
        );
        println!(
            "  Point: ({:.2}, {:.2}, {:.2})",
            closest.x, closest.y, closest.z
        );
    } else {
        println!("\nNo closest point found");
    }

    // Test raycast
    let ray_origin = Vector3::new(20.0, 0.0, 0.0);
    let ray_direction = Vector3::new(-1.0, 0.0, 0.0);
    if let Some(hit) = sphere.raycast_to_surface(ray_origin, ray_direction) {
        println!(
            "\nRaycast from ({:.2}, {:.2}, {:.2}) direction ({:.2}, {:.2}, {:.2}):",
            ray_origin.x,
            ray_origin.y,
            ray_origin.z,
            ray_direction.x,
            ray_direction.y,
            ray_direction.z
        );
        println!("  Hit: ({:.2}, {:.2}, {:.2})", hit.x, hit.y, hit.z);
    } else {
        println!("\nRaycast missed");
    }

    Ok(())
}
