//! Triangle mesh representation

use crate::{ffi, BBox3, Error, Result, Triangle, Voxels};
use nalgebra::Vector3;

mod io; // STL I/O implementation
mod math; // Mesh math helpers
mod transform; // Transformation operations
mod triangle_voxelization; // Triangle voxelization utilities
pub use io::StlUnit;

/// Triangle mesh
///
/// Represents geometry as a collection of triangles.
pub struct Mesh {
    handle: *mut ffi::CMesh,
}

impl Mesh {
    /// Create an empty mesh
    pub fn new() -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::Mesh_hCreate() });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Create mesh from voxels
    ///
    /// Generates a triangle mesh from a voxel field using Marching Cubes algorithm.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Voxels, Mesh};
    /// use nalgebra::Vector3;
    ///
    /// let vox = Voxels::sphere(Vector3::zeros(), 20.0)?;
    /// let mesh = Mesh::from_voxels(&vox)?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn from_voxels(voxels: &Voxels) -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Mesh_hCreateFromVoxels(voxels.handle())
        });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Create a cube mesh from a bounding box
    pub fn from_bbox(bbox: &BBox3) -> Result<Self> {
        let mut mesh = Mesh::new()?;

        let size = bbox.size();
        let center = bbox.center();
        let half = size * 0.5;

        let vertices = [
            Vector3::new(-half.x, -half.y, -half.z) + center,
            Vector3::new(-half.x, -half.y, half.z) + center,
            Vector3::new(-half.x, half.y, -half.z) + center,
            Vector3::new(-half.x, half.y, half.z) + center,
            Vector3::new(half.x, -half.y, -half.z) + center,
            Vector3::new(half.x, -half.y, half.z) + center,
            Vector3::new(half.x, half.y, -half.z) + center,
            Vector3::new(half.x, half.y, half.z) + center,
        ];

        let indices: Vec<i32> = vertices.iter().map(|v| mesh.add_vertex(*v)).collect();

        let add_tri = |mesh: &mut Mesh, a: usize, b: usize, c: usize| {
            mesh.add_triangle(Triangle::new(indices[a], indices[b], indices[c]));
        };

        // Front face
        add_tri(&mut mesh, 0, 1, 3);
        add_tri(&mut mesh, 0, 3, 2);

        // Back face
        add_tri(&mut mesh, 4, 6, 7);
        add_tri(&mut mesh, 4, 7, 5);

        // Left face
        add_tri(&mut mesh, 0, 2, 6);
        add_tri(&mut mesh, 0, 6, 4);

        // Right face
        add_tri(&mut mesh, 1, 5, 7);
        add_tri(&mut mesh, 1, 7, 3);

        // Top face
        add_tri(&mut mesh, 2, 3, 7);
        add_tri(&mut mesh, 2, 7, 6);

        // Bottom face
        add_tri(&mut mesh, 0, 4, 5);
        add_tri(&mut mesh, 0, 5, 1);

        Ok(mesh)
    }

    /// Add a vertex
    ///
    /// Returns the vertex index.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Mesh;
    /// use nalgebra::Vector3;
    ///
    /// let mut mesh = Mesh::new()?;
    /// let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    /// let v1 = mesh.add_vertex(Vector3::new(1.0, 0.0, 0.0));
    /// let v2 = mesh.add_vertex(Vector3::new(0.0, 1.0, 0.0));
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn add_vertex(&mut self, pos: Vector3<f32>) -> i32 {
        let pos = crate::types::Vector3f::from(pos);
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Mesh_nAddVertex(self.handle, &pos as *const crate::types::Vector3f)
        })
    }

    /// Add a triangle
    ///
    /// Returns the triangle index.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Mesh, Triangle};
    /// use nalgebra::Vector3;
    ///
    /// let mut mesh = Mesh::new()?;
    /// let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    /// let v1 = mesh.add_vertex(Vector3::new(1.0, 0.0, 0.0));
    /// let v2 = mesh.add_vertex(Vector3::new(0.0, 1.0, 0.0));
    /// mesh.add_triangle(Triangle::new(v0, v1, v2));
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn add_triangle(&mut self, tri: Triangle) -> i32 {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Mesh_nAddTriangle(self.handle, &tri as *const Triangle)
        })
    }

    /// Add a triangle by vertex indices
    pub fn add_triangle_indices(&mut self, a: i32, b: i32, c: i32) -> i32 {
        self.add_triangle(Triangle::new(a, b, c))
    }

    /// Add a triangle by vertex positions
    pub fn add_triangle_vertices(
        &mut self,
        a: Vector3<f32>,
        b: Vector3<f32>,
        c: Vector3<f32>,
    ) -> i32 {
        let v0 = self.add_vertex(a);
        let v1 = self.add_vertex(b);
        let v2 = self.add_vertex(c);
        self.add_triangle(Triangle::new(v0, v1, v2))
    }

    /// Add multiple vertices and return their indices
    pub fn add_vertices<I>(&mut self, vertices: I) -> Vec<i32>
    where
        I: IntoIterator<Item = Vector3<f32>>,
    {
        let mut indices = Vec::new();
        for vertex in vertices {
            indices.push(self.add_vertex(vertex));
        }
        indices
    }

    /// Add a quad by vertex indices
    pub fn add_quad(&mut self, n0: i32, n1: i32, n2: i32, n3: i32, flipped: bool) {
        if flipped {
            self.add_triangle_indices(n0, n2, n1);
            self.add_triangle_indices(n0, n3, n2);
        } else {
            self.add_triangle_indices(n0, n1, n2);
            self.add_triangle_indices(n0, n2, n3);
        }
    }

    /// Add a quad by vertex positions
    pub fn add_quad_vertices(
        &mut self,
        v0: Vector3<f32>,
        v1: Vector3<f32>,
        v2: Vector3<f32>,
        v3: Vector3<f32>,
        flipped: bool,
    ) {
        let n0 = self.add_vertex(v0);
        let n1 = self.add_vertex(v1);
        let n2 = self.add_vertex(v2);
        let n3 = self.add_vertex(v3);
        self.add_quad(n0, n1, n2, n3, flipped);
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::Mesh_nVertexCount(self.handle) as usize })
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::Mesh_nTriangleCount(self.handle) as usize })
    }

    /// Get a vertex by index
    pub fn get_vertex(&self, index: usize) -> Option<Vector3<f32>> {
        if index >= self.vertex_count() {
            return None;
        }

        let mut v = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Mesh_GetVertex(
                self.handle,
                index as i32,
                &mut v as *mut crate::types::Vector3f,
            );
        });

        Some(Vector3::from(v))
    }

    /// Get a triangle by index
    pub fn get_triangle(&self, index: usize) -> Option<Triangle> {
        if index >= self.triangle_count() {
            return None;
        }

        let mut tri = Triangle::new(0, 0, 0);

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Mesh_GetTriangle(self.handle, index as i32, &mut tri as *mut Triangle);
        });

        Some(tri)
    }

    /// Save to STL file
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Voxels, Mesh};
    /// use nalgebra::Vector3;
    ///
    /// let vox = Voxels::sphere(Vector3::zeros(), 20.0)?;
    /// let mesh = vox.as_mesh()?;
    /// mesh.save_stl("sphere.stl")?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn save_stl<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        // Implementation is in io submodule
        io::save_stl_impl(self, path)
    }

    /// Save to STL file with unit, offset, and scale options
    pub fn save_stl_with_options<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        unit: StlUnit,
        offset_mm: Vector3<f32>,
        scale: f32,
    ) -> Result<()> {
        io::save_stl_with_options(self, path, unit, offset_mm, scale)
    }

    /// Load from STL file
    pub fn load_stl<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        io::load_stl_impl(path)
    }

    /// C#-style alias for `load_stl` (matches `mshFromStlFile` naming).
    pub fn from_stl_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Self::load_stl(path)
    }

    /// Load from STL file with unit, offset, and scale options
    pub fn load_stl_with_options<P: AsRef<std::path::Path>>(
        path: P,
        unit: StlUnit,
        offset_mm: Vector3<f32>,
        scale: f32,
    ) -> Result<Self> {
        io::load_stl_with_options(path, unit, offset_mm, scale)
    }

    /// Check if the mesh is valid
    pub fn is_valid(&self) -> bool {
        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::Mesh_bIsValid(self.handle) })
    }

    /// Get the bounding box of the mesh
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Library, Mesh, Triangle};
    /// use nalgebra::Vector3;
    ///
    /// let _lib = Library::init(0.5)?;
    /// let mut mesh = Mesh::new()?;
    /// mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
    /// mesh.add_vertex(Vector3::new(10.0, 0.0, 0.0));
    /// mesh.add_vertex(Vector3::new(5.0, 10.0, 0.0));
    ///
    /// let bbox = mesh.bounding_box();
    /// println!("BBox: {}", bbox);
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn bounding_box(&self) -> crate::BBox3 {
        let mut bbox = crate::BBox3::empty();
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Mesh_GetBoundingBox(self.handle, &mut bbox as *mut crate::BBox3);
        });
        bbox
    }

    /// C#-style alias for `get_vertex` (matches `vecVertexAt` naming).
    pub fn vertex_at(&self, index: usize) -> Option<Vector3<f32>> {
        self.get_vertex(index)
    }

    /// C#-style alias for `get_triangle` (matches `oTriangleAt` naming).
    pub fn triangle_at(&self, index: usize) -> Option<Triangle> {
        self.get_triangle(index)
    }

    /// Get raw handle (for internal use)
    pub(crate) fn handle(&self) -> *mut ffi::CMesh {
        self.handle
    }

    /// Create from raw handle (for internal use)
    ///
    /// # Safety
    ///
    /// The handle must be a valid CMesh pointer.
    /// This function takes ownership of the handle.
    pub(crate) fn from_handle(handle: *mut ffi::CMesh) -> Self {
        Self { handle }
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::Mesh_Destroy(self.handle);
            });
        }
    }
}

unsafe impl Send for Mesh {}
unsafe impl Sync for Mesh {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Library;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_mesh_creation() {
        let _lib = Library::init(0.5).unwrap();
        let mesh = Mesh::new();
        assert!(mesh.is_ok());
    }

    #[test]
    #[serial]
    fn test_add_vertex() {
        let _lib = Library::init(0.5).unwrap();
        let mut mesh = Mesh::new().unwrap();
        let v0 = mesh.add_vertex(Vector3::zeros());
        assert_eq!(v0, 0);
        assert_eq!(mesh.vertex_count(), 1);
    }
}
