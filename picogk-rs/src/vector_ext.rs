//! Vector3 extensions

use nalgebra::{Matrix4, Point3, Vector3};

/// Extension methods for Vector3
pub trait Vector3Ext {
    /// Return the normalized vector (zero if too small)
    fn normalized(self) -> Vector3<f32>;

    /// Mirror the vector across a plane
    fn mirrored(self, plane_point: Vector3<f32>, plane_normal_unit: Vector3<f32>) -> Vector3<f32>;

    /// Apply a transformation matrix to the vector
    fn transformed(self, matrix: &Matrix4<f32>) -> Vector3<f32>;
}

impl Vector3Ext for Vector3<f32> {
    fn normalized(self) -> Vector3<f32> {
        let norm = self.norm();
        if norm <= f32::EPSILON {
            Vector3::zeros()
        } else {
            self / norm
        }
    }

    fn mirrored(self, plane_point: Vector3<f32>, plane_normal_unit: Vector3<f32>) -> Vector3<f32> {
        debug_assert!((plane_normal_unit.norm() - 1.0).abs() < 1e-6);
        self - 2.0 * (self - plane_point).dot(&plane_normal_unit) * plane_normal_unit
    }

    fn transformed(self, matrix: &Matrix4<f32>) -> Vector3<f32> {
        matrix.transform_point(&Point3::from(self)).coords
    }
}
