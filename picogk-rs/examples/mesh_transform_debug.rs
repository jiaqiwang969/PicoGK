use nalgebra::Vector3;
use picogk::{Library, Mesh, Triangle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _lib = Library::init(0.5)?;

    // Create a simple triangle mesh
    let mut mesh = Mesh::new()?;
    let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    let v1 = mesh.add_vertex(Vector3::new(1.0, 0.0, 0.0));
    let v2 = mesh.add_vertex(Vector3::new(0.0, 1.0, 0.0));
    mesh.add_triangle(Triangle::new(v0, v1, v2));

    println!("Original mesh:");
    println!("  Vertices: {}", mesh.vertex_count());
    println!("  Triangles: {}", mesh.triangle_count());

    // Get the triangle vertices
    let (a, b, c) = mesh.get_triangle_vertices(0)?;
    println!("  Triangle 0:");
    println!("    A: ({}, {}, {})", a.x, a.y, a.z);
    println!("    B: ({}, {}, {})", b.x, b.y, b.z);
    println!("    C: ({}, {}, {})", c.x, c.y, c.z);

    // Transform: scale 2x and offset by (10, 0, 0)
    let transformed =
        mesh.create_transformed(Vector3::new(2.0, 2.0, 2.0), Vector3::new(10.0, 0.0, 0.0))?;

    println!("\nTransformed mesh:");
    println!("  Vertices: {}", transformed.vertex_count());
    println!("  Triangles: {}", transformed.triangle_count());

    let (a, b, c) = transformed.get_triangle_vertices(0)?;
    println!("  Triangle 0:");
    println!("    A: ({}, {}, {})", a.x, a.y, a.z);
    println!("    B: ({}, {}, {})", b.x, b.y, b.z);
    println!("    C: ({}, {}, {})", c.x, c.y, c.z);

    Ok(())
}
