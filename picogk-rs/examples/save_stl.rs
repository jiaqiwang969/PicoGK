use nalgebra::Vector3;
use picogk::{Library, Mesh, Triangle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize library
    let _lib = Library::init(0.5)?;

    // Create a simple triangle mesh
    let mut mesh = Mesh::new()?;

    // Add vertices for a simple triangle
    let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    let v1 = mesh.add_vertex(Vector3::new(10.0, 0.0, 0.0));
    let v2 = mesh.add_vertex(Vector3::new(5.0, 10.0, 0.0));

    // Add triangle
    mesh.add_triangle(Triangle::new(v0, v1, v2));

    println!(
        "Created mesh with {} vertices and {} triangles",
        mesh.vertex_count(),
        mesh.triangle_count()
    );

    // Save to STL file
    let output_path = "triangle.stl";
    mesh.save_stl(output_path)?;

    println!("✓ STL file saved to: {}", output_path);

    Ok(())
}
