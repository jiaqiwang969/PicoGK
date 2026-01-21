use picogk::{Library, Mesh};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize library
    let _lib = Library::init(0.5)?;

    // Load STL file
    let input_path = "triangle.stl";
    println!("Loading STL file: {}", input_path);

    let mesh = Mesh::load_stl(input_path)?;

    println!("✓ STL file loaded successfully");
    println!("  Vertices: {}", mesh.vertex_count());
    println!("  Triangles: {}", mesh.triangle_count());

    // Print first few vertices
    println!("\nFirst 3 vertices:");
    for i in 0..3.min(mesh.vertex_count()) {
        if let Some(v) = mesh.get_vertex(i) {
            println!("  v{}: ({:.2}, {:.2}, {:.2})", i, v.x, v.y, v.z);
        }
    }

    Ok(())
}
