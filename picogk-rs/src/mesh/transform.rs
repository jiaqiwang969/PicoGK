//! Mesh transformation and manipulation operations

use crate::{Mesh, Result, Triangle};
use nalgebra::{Matrix4, Vector3};

impl Mesh {
    /// Get the vertices of a triangle by index
    ///
    /// Returns the three vertices (A, B, C) of the triangle.
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the triangle (0-based)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Mesh;
    ///
    /// let mesh = Mesh::new()?;
    /// // ... add triangles ...
    /// let (a, b, c) = mesh.get_triangle_vertices(0)?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn get_triangle_vertices(
        &self,
        index: usize,
    ) -> Result<(Vector3<f32>, Vector3<f32>, Vector3<f32>)> {
        if index >= self.triangle_count() {
            return Err(crate::Error::InvalidParameter(format!(
                "Triangle index {} out of range",
                index
            )));
        }

        let mut a = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut b = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut c = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            crate::ffi::Mesh_GetTriangleV(
                self.handle(),
                index as i32,
                &mut a as *mut crate::types::Vector3f,
                &mut b as *mut crate::types::Vector3f,
                &mut c as *mut crate::types::Vector3f,
            );
        });

        Ok((Vector3::from(a), Vector3::from(b), Vector3::from(c)))
    }

    /// Create a transformed mesh by applying scale and offset
    ///
    /// # Arguments
    ///
    /// * `scale` - Scale factor for each axis
    /// * `offset` - Translation offset
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Mesh;
    /// use nalgebra::Vector3;
    ///
    /// let mesh = Mesh::new()?;
    /// // ... add triangles ...
    /// let scaled = mesh.create_transformed(
    ///     Vector3::new(2.0, 2.0, 2.0),  // Scale 2x
    ///     Vector3::new(10.0, 0.0, 0.0)  // Move 10mm in X
    /// )?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn create_transformed(&self, scale: Vector3<f32>, offset: Vector3<f32>) -> Result<Mesh> {
        let mut result = Mesh::new()?;

        for i in 0..self.triangle_count() {
            let (mut a, mut b, mut c) = self.get_triangle_vertices(i)?;

            // Apply scale
            a.x *= scale.x;
            a.y *= scale.y;
            a.z *= scale.z;

            b.x *= scale.x;
            b.y *= scale.y;
            b.z *= scale.z;

            c.x *= scale.x;
            c.y *= scale.y;
            c.z *= scale.z;

            // Apply offset
            a += offset;
            b += offset;
            c += offset;

            // Add triangle
            let v0 = result.add_vertex(a);
            let v1 = result.add_vertex(b);
            let v2 = result.add_vertex(c);
            result.add_triangle(Triangle::new(v0, v1, v2));
        }

        Ok(result)
    }

    /// Create a transformed mesh by applying a transformation matrix
    ///
    /// # Arguments
    ///
    /// * `matrix` - 4x4 transformation matrix
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Mesh;
    /// use nalgebra::Matrix4;
    ///
    /// let mesh = Mesh::new()?;
    /// // ... add triangles ...
    /// let matrix = Matrix4::new_translation(&nalgebra::Vector3::new(10.0, 0.0, 0.0));
    /// let transformed = mesh.create_transformed_matrix(&matrix)?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn create_transformed_matrix(&self, matrix: &Matrix4<f32>) -> Result<Mesh> {
        let mut result = Mesh::new()?;

        for i in 0..self.triangle_count() {
            let (a, b, c) = self.get_triangle_vertices(i)?;

            // Transform vertices
            let a_transformed = matrix.transform_point(&nalgebra::Point3::from(a));
            let b_transformed = matrix.transform_point(&nalgebra::Point3::from(b));
            let c_transformed = matrix.transform_point(&nalgebra::Point3::from(c));

            // Add triangle
            let v0 = result.add_vertex(a_transformed.coords);
            let v1 = result.add_vertex(b_transformed.coords);
            let v2 = result.add_vertex(c_transformed.coords);
            result.add_triangle(Triangle::new(v0, v1, v2));
        }

        Ok(result)
    }

    /// Create a mirrored mesh at the specified plane
    ///
    /// # Arguments
    ///
    /// * `plane_point` - A point through which the mirror plane passes
    /// * `plane_normal` - The normal vector of the mirror plane (will be normalized)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Mesh;
    /// use nalgebra::Vector3;
    ///
    /// let mesh = Mesh::new()?;
    /// // ... add triangles ...
    /// // Mirror across XY plane (Z=0)
    /// let mirrored = mesh.create_mirrored(
    ///     Vector3::zeros(),
    ///     Vector3::new(0.0, 0.0, 1.0)
    /// )?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn create_mirrored(
        &self,
        plane_point: Vector3<f32>,
        plane_normal: Vector3<f32>,
    ) -> Result<Mesh> {
        let mut result = Mesh::new()?;
        let normal = plane_normal.normalize();

        for i in 0..self.triangle_count() {
            let (a, b, c) = self.get_triangle_vertices(i)?;

            // Mirror each vertex
            let a_mirrored = mirror_point(a, plane_point, normal);
            let b_mirrored = mirror_point(b, plane_point, normal);
            let c_mirrored = mirror_point(c, plane_point, normal);

            // Add triangle
            let v0 = result.add_vertex(a_mirrored);
            let v1 = result.add_vertex(b_mirrored);
            let v2 = result.add_vertex(c_mirrored);
            result.add_triangle(Triangle::new(v0, v1, v2));
        }

        Ok(result)
    }

    /// Append another mesh to this mesh
    ///
    /// Adds all triangles from the other mesh to this mesh.
    ///
    /// # Arguments
    ///
    /// * `other` - The mesh to append
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Mesh;
    ///
    /// let mut mesh1 = Mesh::new()?;
    /// let mesh2 = Mesh::new()?;
    /// // ... add triangles to both ...
    /// mesh1.append(&mesh2)?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn append(&mut self, other: &Mesh) -> Result<()> {
        for i in 0..other.triangle_count() {
            let (a, b, c) = other.get_triangle_vertices(i)?;

            let v0 = self.add_vertex(a);
            let v1 = self.add_vertex(b);
            let v2 = self.add_vertex(c);
            self.add_triangle(Triangle::new(v0, v1, v2));
        }

        Ok(())
    }
}

/// Mirror a point across a plane
fn mirror_point(
    point: Vector3<f32>,
    plane_point: Vector3<f32>,
    plane_normal: Vector3<f32>,
) -> Vector3<f32> {
    let v = point - plane_point;
    let distance = v.dot(&plane_normal);
    point - 2.0 * distance * plane_normal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_point() {
        // Mirror across XY plane (Z=0)
        let point = Vector3::new(1.0, 2.0, 3.0);
        let plane_point = Vector3::zeros();
        let plane_normal = Vector3::new(0.0, 0.0, 1.0);

        let mirrored = mirror_point(point, plane_point, plane_normal);

        assert_eq!(mirrored.x, 1.0);
        assert_eq!(mirrored.y, 2.0);
        assert_eq!(mirrored.z, -3.0);
    }
}
