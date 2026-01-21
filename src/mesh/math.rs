//! Mesh math utilities

use crate::{Error, Mesh, Result};
use nalgebra::Vector3;

impl Mesh {
    /// Find the triangle index that contains the specified surface point
    pub fn find_triangle_from_surface_point(
        &self,
        surface_point: Vector3<f32>,
    ) -> Result<Option<usize>> {
        for index in 0..self.triangle_count() {
            let (a, b, c) = self.get_triangle_vertices(index)?;
            if Mesh::point_lies_on_triangle(surface_point, a, b, c) {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    /// Check if a point lies inside the triangle
    pub fn point_lies_on_triangle(
        point: Vector3<f32>,
        a: Vector3<f32>,
        b: Vector3<f32>,
        c: Vector3<f32>,
    ) -> bool {
        let a = a - point;
        let b = b - point;
        let c = c - point;

        let u = b.cross(&c);
        let v = c.cross(&a);
        let w = a.cross(&b);

        if u.dot(&v) < 0.0 {
            return false;
        }
        if u.dot(&w) < 0.0 {
            return false;
        }
        true
    }

    /// Get the normal of a triangle by index (normalized)
    pub fn triangle_normal(&self, index: usize) -> Result<Vector3<f32>> {
        let (a, b, c) = self.get_triangle_vertices(index)?;
        let normal = (b - a).cross(&(c - a));
        let norm = normal.norm();
        if norm <= f32::EPSILON {
            Ok(Vector3::zeros())
        } else {
            Ok(normal / norm)
        }
    }

    /// Get the area of a triangle by index
    pub fn triangle_area(&self, index: usize) -> Result<f32> {
        let (a, b, c) = self.get_triangle_vertices(index)?;
        Ok(0.5 * (b - a).cross(&(c - a)).norm())
    }

    /// Compute total surface area of the mesh
    pub fn surface_area(&self) -> Result<f32> {
        let mut area = 0.0;
        for i in 0..self.triangle_count() {
            area += self.triangle_area(i)?;
        }
        Ok(area)
    }

    /// Signed volume of a closed, consistently oriented mesh.
    ///
    /// Uses the standard triangle-tetrahedron decomposition against the origin.
    /// If the mesh is not closed or triangle winding is inconsistent, results may be meaningless.
    pub fn signed_volume(&self) -> Result<f32> {
        let mut v6_sum = 0.0f32;
        for i in 0..self.triangle_count() {
            let (a, b, c) = self.get_triangle_vertices(i)?;
            v6_sum += a.dot(&b.cross(&c));
        }
        Ok(v6_sum / 6.0)
    }

    /// Absolute volume of a closed mesh (helper around `signed_volume`).
    pub fn volume(&self) -> Result<f32> {
        Ok(self.signed_volume()?.abs())
    }

    /// Volume centroid (center of mass) of a closed, consistently oriented mesh.
    pub fn centroid(&self) -> Result<Vector3<f32>> {
        let mut v6_sum = 0.0f32;
        let mut c_sum = Vector3::zeros();
        for i in 0..self.triangle_count() {
            let (a, b, c) = self.get_triangle_vertices(i)?;
            let v6 = a.dot(&b.cross(&c));
            v6_sum += v6;
            c_sum += (a + b + c) * v6;
        }

        if v6_sum.abs() <= f32::EPSILON {
            return Err(Error::OperationFailed(
                "Mesh volume is zero; centroid is undefined".to_string(),
            ));
        }

        Ok(c_sum / (4.0 * v6_sum))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BBox3;
    use crate::Library;
    use crate::Triangle;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_triangle_area() {
        let _lib = Library::init(0.5).unwrap();
        let mut mesh = Mesh::new().unwrap();
        let v0 = mesh.add_vertex(Vector3::new(0.0, 0.0, 0.0));
        let v1 = mesh.add_vertex(Vector3::new(1.0, 0.0, 0.0));
        let v2 = mesh.add_vertex(Vector3::new(0.0, 1.0, 0.0));
        mesh.add_triangle(Triangle::new(v0, v1, v2));

        let area = mesh.triangle_area(0).unwrap();
        assert!((area - 0.5).abs() < 1e-6);
    }

    #[test]
    #[serial]
    fn test_volume_and_centroid_on_cube() {
        let _lib = Library::init(0.5).unwrap();
        let bbox = BBox3::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(2.0, 2.0, 2.0));
        let mesh = Mesh::from_bbox(&bbox).unwrap();

        let volume = mesh.volume().unwrap();
        assert!((volume - 8.0).abs() < 1e-3);

        let centroid = mesh.centroid().unwrap();
        assert!((centroid.x - 1.0).abs() < 1e-3);
        assert!((centroid.y - 1.0).abs() < 1e-3);
        assert!((centroid.z - 1.0).abs() < 1e-3);
    }
}
