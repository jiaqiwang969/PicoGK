//! STL file I/O support for Mesh

use super::Mesh;
use crate::{Error, Result, Triangle};
use nalgebra::Vector3;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

/// STL unit types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StlUnit {
    /// Automatically detect from file header
    Auto,
    /// Millimeters
    Mm,
    /// Centimeters
    Cm,
    /// Meters
    M,
    /// Feet
    Ft,
    /// Inches
    In,
}

impl StlUnit {
    /// Get the multiplier to convert to millimeters
    fn to_mm_multiplier(self) -> f32 {
        match self {
            StlUnit::Auto | StlUnit::Mm => 1.0,
            StlUnit::Cm => 10.0,
            StlUnit::M => 1000.0,
            StlUnit::Ft => 304.8,
            StlUnit::In => 25.4,
        }
    }

    /// Parse unit from header string
    fn from_header(header: &str) -> Self {
        let header_lower = header.to_lowercase();
        if header_lower.contains("units=mm") {
            StlUnit::Mm
        } else if header_lower.contains("units=cm") {
            StlUnit::Cm
        } else if header_lower.contains("units= m") || header_lower.contains("units=m") {
            StlUnit::M
        } else if header_lower.contains("units=ft") {
            StlUnit::Ft
        } else if header_lower.contains("units=in") {
            StlUnit::In
        } else {
            StlUnit::Mm // Default to mm
        }
    }

    /// Get unit string for header
    fn to_header_string(self) -> &'static str {
        match self {
            StlUnit::Auto | StlUnit::Mm => "UNITS=mm",
            StlUnit::Cm => "UNITS=cm",
            StlUnit::M => "UNITS= m",
            StlUnit::Ft => "UNITS=ft",
            StlUnit::In => "UNITS=in",
        }
    }
}

/// Save mesh to binary STL file
///
/// # Arguments
///
/// * `mesh` - The mesh to save
/// * `path` - File path to save to
///
/// # Example
///
/// ```rust,no_run
/// use picogk::{Voxels, Mesh};
/// use nalgebra::Vector3;
///
/// let sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
/// let mesh = sphere.as_mesh()?;
/// mesh.save_stl("sphere.stl")?;
/// # Ok::<(), picogk::Error>(())
/// ```
pub(super) fn save_stl_impl<P: AsRef<Path>>(mesh: &Mesh, path: P) -> Result<()> {
    save_stl_with_options(mesh, path, StlUnit::Mm, Vector3::zeros(), 1.0)
}

/// Save mesh to binary STL file with options
///
/// # Arguments
///
/// * `mesh` - The mesh to save
/// * `path` - File path to save to
/// * `unit` - Unit to use
/// * `offset` - Offset to apply (in mm)
/// * `scale` - Scale to apply
pub(super) fn save_stl_with_options<P: AsRef<Path>>(
    mesh: &Mesh,
    path: P,
    unit: StlUnit,
    offset: Vector3<f32>,
    scale: f32,
) -> Result<()> {
    let file = File::create(path)
        .map_err(|e| Error::OperationFailed(format!("Failed to create STL file: {}", e)))?;

    let mut writer = BufWriter::new(file);

    // Write header (80 bytes)
    let mut header = format!("PicoGK {}", unit.to_header_string());
    header.truncate(80);
    while header.len() < 80 {
        header.push(' ');
    }
    writer
        .write_all(header.as_bytes())
        .map_err(|e| Error::OperationFailed(format!("Failed to write STL header: {}", e)))?;

    // Write triangle count
    let triangle_count = mesh.triangle_count() as u32;
    writer
        .write_all(&triangle_count.to_le_bytes())
        .map_err(|e| Error::OperationFailed(format!("Failed to write triangle count: {}", e)))?;

    // Get unit multiplier
    let unit_multiplier = unit.to_mm_multiplier();

    // Write triangles
    for i in 0..mesh.triangle_count() {
        if let Some(tri) = mesh.get_triangle(i) {
            // Get vertices
            let v1 = mesh
                .get_vertex(tri.v0 as usize)
                .ok_or_else(|| Error::OperationFailed("Invalid vertex index".to_string()))?;
            let v2 = mesh
                .get_vertex(tri.v1 as usize)
                .ok_or_else(|| Error::OperationFailed("Invalid vertex index".to_string()))?;
            let v3 = mesh
                .get_vertex(tri.v2 as usize)
                .ok_or_else(|| Error::OperationFailed("Invalid vertex index".to_string()))?;

            // Apply transformations
            let v1 = transform_vertex(v1, offset, scale, unit_multiplier);
            let v2 = transform_vertex(v2, offset, scale, unit_multiplier);
            let v3 = transform_vertex(v3, offset, scale, unit_multiplier);

            // Calculate normal
            let edge1 = v2 - v1;
            let edge2 = v3 - v1;
            let cross = edge1.cross(&edge2);
            let normal = if cross.norm() > 1e-10 {
                cross.normalize()
            } else {
                Vector3::new(0.0, 0.0, 1.0) // Default normal for degenerate triangles
            };

            // Write triangle data
            write_f32_array(&mut writer, &[normal.x, normal.y, normal.z])?;
            write_f32_array(&mut writer, &[v1.x, v1.y, v1.z])?;
            write_f32_array(&mut writer, &[v2.x, v2.y, v2.z])?;
            write_f32_array(&mut writer, &[v3.x, v3.y, v3.z])?;
            writer
                .write_all(&[0u8, 0u8])
                .map_err(|e| Error::OperationFailed(format!("Failed to write attribute: {}", e)))?;
        }
    }

    Ok(())
}

/// Transform vertex for STL export
fn transform_vertex(
    v: Vector3<f32>,
    offset: Vector3<f32>,
    scale: f32,
    unit_multiplier: f32,
) -> Vector3<f32> {
    let mut result = v + offset;
    result *= scale;
    result /= unit_multiplier;
    result
}

/// Write f32 array in little-endian format
fn write_f32_array<W: Write>(writer: &mut W, values: &[f32]) -> Result<()> {
    for &value in values {
        writer
            .write_all(&value.to_le_bytes())
            .map_err(|e| Error::OperationFailed(format!("Failed to write float: {}", e)))?;
    }
    Ok(())
}

/// Read f32 array in little-endian format
fn read_f32_array<R: Read>(reader: &mut R, count: usize) -> Result<Vec<f32>> {
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let mut bytes = [0u8; 4];
        reader
            .read_exact(&mut bytes)
            .map_err(|e| Error::OperationFailed(format!("Failed to read float: {}", e)))?;
        values.push(f32::from_le_bytes(bytes));
    }
    Ok(values)
}

/// Read u32 in little-endian format
fn read_u32<R: Read>(reader: &mut R) -> Result<u32> {
    let mut bytes = [0u8; 4];
    reader
        .read_exact(&mut bytes)
        .map_err(|e| Error::OperationFailed(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_le_bytes(bytes))
}

/// Load mesh from binary STL file
///
/// # Arguments
///
/// * `mesh` - The mesh to load into
/// * `path` - File path to load from
///
/// # Example
///
/// ```rust,no_run
/// use picogk::Mesh;
///
/// let mesh = Mesh::load_stl("input.stl")?;
/// # Ok::<(), picogk::Error>(())
/// ```
pub(super) fn load_stl_impl<P: AsRef<Path>>(path: P) -> Result<Mesh> {
    load_stl_with_options(path, StlUnit::Auto, Vector3::zeros(), 1.0)
}

/// Load mesh from binary STL file with options
///
/// # Arguments
///
/// * `path` - File path to load from
/// * `unit` - Unit to interpret (Auto will read from header)
/// * `offset` - Offset to apply (in mm)
/// * `scale` - Scale to apply
pub(super) fn load_stl_with_options<P: AsRef<Path>>(
    path: P,
    unit: StlUnit,
    offset: Vector3<f32>,
    scale: f32,
) -> Result<Mesh> {
    let file = File::open(path)
        .map_err(|e| Error::OperationFailed(format!("Failed to open STL file: {}", e)))?;

    let mut reader = BufReader::new(file);

    // Read header (80 bytes)
    let mut header = [0u8; 80];
    reader
        .read_exact(&mut header)
        .map_err(|e| Error::OperationFailed(format!("Failed to read STL header: {}", e)))?;

    // Detect ASCII STL files (not supported)
    let header_str = String::from_utf8_lossy(&header);
    if header_str.trim_start().to_lowercase().starts_with("solid") {
        let peek = reader
            .fill_buf()
            .map_err(|e| Error::OperationFailed(format!("Failed to read STL body: {}", e)))?;
        let peek_str = String::from_utf8_lossy(peek).to_lowercase();
        if peek_str.contains("vertex") {
            return Err(Error::OperationFailed(
                "ASCII STL loading is not supported".to_string(),
            ));
        }
    }

    // Parse unit from header if Auto
    let unit = if unit == StlUnit::Auto {
        StlUnit::from_header(&header_str)
    } else {
        unit
    };

    let unit_multiplier = unit.to_mm_multiplier();

    // Read triangle count
    let triangle_count = read_u32(&mut reader)?;

    // Create mesh
    let mut mesh = Mesh::new()?;

    // Read triangles
    for _ in 0..triangle_count {
        // Read normal (we'll recalculate it, but need to skip it)
        let _normal = read_f32_array(&mut reader, 3)?;

        // Read vertices
        let v1_data = read_f32_array(&mut reader, 3)?;
        let v2_data = read_f32_array(&mut reader, 3)?;
        let v3_data = read_f32_array(&mut reader, 3)?;

        // Skip attribute bytes
        let mut attr = [0u8; 2];
        reader
            .read_exact(&mut attr)
            .map_err(|e| Error::OperationFailed(format!("Failed to read attribute: {}", e)))?;

        // Apply inverse transformations (STL -> internal coordinates)
        let v1 = inverse_transform_vertex(
            Vector3::new(v1_data[0], v1_data[1], v1_data[2]),
            offset,
            scale,
            unit_multiplier,
        );
        let v2 = inverse_transform_vertex(
            Vector3::new(v2_data[0], v2_data[1], v2_data[2]),
            offset,
            scale,
            unit_multiplier,
        );
        let v3 = inverse_transform_vertex(
            Vector3::new(v3_data[0], v3_data[1], v3_data[2]),
            offset,
            scale,
            unit_multiplier,
        );

        // Add vertices and triangle
        let i0 = mesh.add_vertex(v1);
        let i1 = mesh.add_vertex(v2);
        let i2 = mesh.add_vertex(v3);
        mesh.add_triangle(Triangle::new(i0, i1, i2));
    }

    Ok(mesh)
}

/// Inverse transform vertex for STL import
fn inverse_transform_vertex(
    v: Vector3<f32>,
    offset: Vector3<f32>,
    scale: f32,
    unit_multiplier: f32,
) -> Vector3<f32> {
    let mut result = v;
    result *= unit_multiplier;
    result /= scale;
    result -= offset;
    result
}
