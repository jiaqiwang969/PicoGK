//! Implicit surface functions

use crate::BBox3;
use nalgebra::{Vector2, Vector3};

/// Trait for implicit surface functions
///
/// Implement this trait to define custom geometry using signed distance functions.
///
/// # Example
///
/// ```rust
/// use picogk::{Implicit, BBox3};
/// use nalgebra::Vector3;
///
/// struct SphereImplicit {
///     center: Vector3<f32>,
///     radius: f32,
/// }
///
/// impl Implicit for SphereImplicit {
///     fn signed_distance(&self, point: Vector3<f32>) -> f32 {
///         (point - self.center).norm() - self.radius
///     }
/// }
/// ```
pub trait Implicit: Send + Sync {
    /// Compute signed distance to the surface
    ///
    /// Returns:
    /// - Negative values inside the object
    /// - Zero at the surface
    /// - Positive values outside the object
    fn signed_distance(&self, point: Vector3<f32>) -> f32;

    /// Get bounding box (optional)
    ///
    /// If provided, only this region will be sampled.
    fn bounds(&self) -> Option<BBox3> {
        None
    }
}

/// Gyroid triply periodic minimal surface
///
/// Formula: sin(x)cos(y) + sin(y)cos(z) + sin(z)cos(x) = 0
///
/// # Example
///
/// ```rust,no_run
/// use picogk::{GyroidImplicit, Voxels, BBox3};
/// use nalgebra::Vector3;
///
/// let bounds = BBox3::new(
///     Vector3::new(-30.0, -30.0, -30.0),
///     Vector3::new(30.0, 30.0, 30.0),
/// );
///
/// let gyroid = GyroidImplicit::new(10.0, 1.5, bounds);
/// // let vox = Voxels::from_implicit(&gyroid)?;
/// # Ok::<(), picogk::Error>(())
/// ```
pub struct GyroidImplicit {
    scale: f32,
    thickness: f32,
    bounds: BBox3,
}

impl GyroidImplicit {
    /// Create a new Gyroid implicit
    ///
    /// # Arguments
    ///
    /// * `scale` - Period size of the Gyroid pattern
    /// * `thickness` - Wall thickness
    /// * `bounds` - Bounding box for the structure
    pub fn new(scale: f32, thickness: f32, bounds: BBox3) -> Self {
        Self {
            scale,
            thickness,
            bounds,
        }
    }
}

impl Implicit for GyroidImplicit {
    fn signed_distance(&self, point: Vector3<f32>) -> f32 {
        let x = point.x / self.scale;
        let y = point.y / self.scale;
        let z = point.z / self.scale;

        let gyroid = x.sin() * y.cos() + y.sin() * z.cos() + z.sin() * x.cos();

        gyroid.abs() - self.thickness / self.scale
    }

    fn bounds(&self) -> Option<BBox3> {
        Some(self.bounds)
    }
}

/// Twisted torus
///
/// A torus with a twist along the Z axis.
///
/// # Example
///
/// ```rust,no_run
/// use picogk::{TwistedTorusImplicit, BBox3};
/// use nalgebra::Vector3;
///
/// let bounds = BBox3::new(
///     Vector3::new(-30.0, -30.0, -10.0),
///     Vector3::new(30.0, 30.0, 10.0),
/// );
///
/// let torus = TwistedTorusImplicit::new(20.0, 5.0, 3.0, bounds);
/// ```
pub struct TwistedTorusImplicit {
    major_radius: f32,
    minor_radius: f32,
    twists: f32,
    bounds: BBox3,
}

impl TwistedTorusImplicit {
    /// Create a new twisted torus
    ///
    /// # Arguments
    ///
    /// * `major_radius` - Major radius (distance from center to tube center)
    /// * `minor_radius` - Minor radius (tube radius)
    /// * `twists` - Number of twists along Z axis
    /// * `bounds` - Bounding box
    pub fn new(major_radius: f32, minor_radius: f32, twists: f32, bounds: BBox3) -> Self {
        Self {
            major_radius,
            minor_radius,
            twists,
            bounds,
        }
    }
}

impl Implicit for TwistedTorusImplicit {
    fn signed_distance(&self, point: Vector3<f32>) -> f32 {
        let _dist_to_axis = (point.x * point.x + point.y * point.y).sqrt();
        let angle = point.y.atan2(point.x);
        let twist = angle + self.twists * point.z / 10.0;

        let torus_center = Vector3::new(
            self.major_radius * angle.cos(),
            self.major_radius * angle.sin(),
            point.z,
        );

        let diff = point - torus_center;
        let rotated_x = diff.x * twist.cos() - diff.y * twist.sin();
        let rotated_y = diff.x * twist.sin() + diff.y * twist.cos();

        let dist_to_surface =
            (rotated_x * rotated_x + rotated_y * rotated_y + diff.z * diff.z).sqrt();

        dist_to_surface - self.minor_radius
    }

    fn bounds(&self) -> Option<BBox3> {
        Some(self.bounds)
    }
}

/// Standard torus (non-twisted) implicit
///
/// A ring-shaped torus around the Z axis.
pub struct TorusImplicit {
    center: Vector3<f32>,
    major_radius: f32,
    minor_radius: f32,
}

impl TorusImplicit {
    pub fn new(center: Vector3<f32>, major_radius: f32, minor_radius: f32) -> Self {
        Self {
            center,
            major_radius,
            minor_radius,
        }
    }
}

impl Implicit for TorusImplicit {
    fn signed_distance(&self, point: Vector3<f32>) -> f32 {
        let p = point - self.center;
        let q = Vector2::new((p.x * p.x + p.y * p.y).sqrt() - self.major_radius, p.z);
        q.norm() - self.minor_radius
    }

    fn bounds(&self) -> Option<BBox3> {
        let r = self.major_radius + self.minor_radius;
        let ext = Vector3::new(r, r, self.minor_radius);
        Some(BBox3::new(self.center - ext, self.center + ext))
    }
}

/// Capsule implicit (line segment + radius)
///
/// A capsule is a cylinder with hemispherical caps, defined by two endpoints.
pub struct CapsuleImplicit {
    a: Vector3<f32>,
    b: Vector3<f32>,
    radius: f32,
}

impl CapsuleImplicit {
    pub fn new(a: Vector3<f32>, b: Vector3<f32>, radius: f32) -> Self {
        Self { a, b, radius }
    }
}

impl Implicit for CapsuleImplicit {
    fn signed_distance(&self, point: Vector3<f32>) -> f32 {
        let pa = point - self.a;
        let ba = self.b - self.a;
        let baba = ba.dot(&ba);
        if baba <= f32::EPSILON {
            return pa.norm() - self.radius;
        }
        let t = (pa.dot(&ba) / baba).clamp(0.0, 1.0);
        let closest = self.a + ba * t;
        (point - closest).norm() - self.radius
    }

    fn bounds(&self) -> Option<BBox3> {
        let r = Vector3::new(self.radius, self.radius, self.radius);
        let min = Vector3::new(
            self.a.x.min(self.b.x),
            self.a.y.min(self.b.y),
            self.a.z.min(self.b.z),
        ) - r;
        let max = Vector3::new(
            self.a.x.max(self.b.x),
            self.a.y.max(self.b.y),
            self.a.z.max(self.b.z),
        ) + r;
        Some(BBox3::new(min, max))
    }
}

/// Simple sphere implicit
pub struct SphereImplicit {
    center: Vector3<f32>,
    radius: f32,
}

impl SphereImplicit {
    pub fn new(center: Vector3<f32>, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Implicit for SphereImplicit {
    fn signed_distance(&self, point: Vector3<f32>) -> f32 {
        (point - self.center).norm() - self.radius
    }

    fn bounds(&self) -> Option<BBox3> {
        let r = Vector3::new(self.radius, self.radius, self.radius);
        Some(BBox3::new(self.center - r, self.center + r))
    }
}

/// Axis-aligned box implicit (center + size)
pub struct BoxImplicit {
    center: Vector3<f32>,
    half_size: Vector3<f32>,
}

impl BoxImplicit {
    pub fn new(center: Vector3<f32>, size: Vector3<f32>) -> Self {
        Self {
            center,
            half_size: size * 0.5,
        }
    }
}

impl Implicit for BoxImplicit {
    fn signed_distance(&self, point: Vector3<f32>) -> f32 {
        let p = point - self.center;
        let d = Vector3::new(p.x.abs(), p.y.abs(), p.z.abs()) - self.half_size;
        let outside = Vector3::new(d.x.max(0.0), d.y.max(0.0), d.z.max(0.0));
        let inside = d.x.max(d.y.max(d.z)).min(0.0);
        outside.norm() + inside
    }

    fn bounds(&self) -> Option<BBox3> {
        Some(BBox3::new(
            self.center - self.half_size,
            self.center + self.half_size,
        ))
    }
}

/// Axis-aligned cylinder implicit (Z axis)
pub struct CylinderImplicit {
    center: Vector3<f32>,
    radius: f32,
    height: f32,
}

impl CylinderImplicit {
    pub fn new(center: Vector3<f32>, radius: f32, height: f32) -> Self {
        Self {
            center,
            radius,
            height,
        }
    }
}

impl Implicit for CylinderImplicit {
    fn signed_distance(&self, point: Vector3<f32>) -> f32 {
        let p = point - self.center;
        let d = Vector2::new(
            (p.x * p.x + p.y * p.y).sqrt() - self.radius,
            p.z.abs() - self.height * 0.5,
        );
        let outside = Vector2::new(d.x.max(0.0), d.y.max(0.0));
        let inside = d.x.max(d.y).min(0.0);
        outside.norm() + inside
    }

    fn bounds(&self) -> Option<BBox3> {
        let half = Vector3::new(self.radius, self.radius, self.height * 0.5);
        Some(BBox3::new(self.center - half, self.center + half))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_implicit() {
        let sphere = SphereImplicit::new(Vector3::zeros(), 10.0);

        // Inside
        assert!(sphere.signed_distance(Vector3::zeros()) < 0.0);

        // On surface
        assert!((sphere.signed_distance(Vector3::new(10.0, 0.0, 0.0))).abs() < 0.001);

        // Outside
        assert!(sphere.signed_distance(Vector3::new(20.0, 0.0, 0.0)) > 0.0);
    }

    #[test]
    fn test_box_implicit() {
        let box_imp = BoxImplicit::new(Vector3::zeros(), Vector3::new(2.0, 2.0, 2.0));
        assert!(box_imp.signed_distance(Vector3::zeros()) < 0.0);
        assert!(box_imp.signed_distance(Vector3::new(2.0, 0.0, 0.0)) > 0.0);
    }

    #[test]
    fn test_cylinder_implicit() {
        let cyl = CylinderImplicit::new(Vector3::zeros(), 1.0, 2.0);
        assert!(cyl.signed_distance(Vector3::zeros()) < 0.0);
        assert!(cyl.signed_distance(Vector3::new(2.0, 0.0, 0.0)) > 0.0);
    }

    #[test]
    fn test_gyroid_implicit() {
        let bounds = BBox3::new(
            Vector3::new(-10.0, -10.0, -10.0),
            Vector3::new(10.0, 10.0, 10.0),
        );
        let gyroid = GyroidImplicit::new(10.0, 1.0, bounds);

        // Just test that it computes without panicking
        let _dist = gyroid.signed_distance(Vector3::zeros());
    }

    #[test]
    fn test_torus_implicit() {
        let torus = TorusImplicit::new(Vector3::zeros(), 10.0, 2.0);
        // Center of tube is inside
        assert!(torus.signed_distance(Vector3::new(10.0, 0.0, 0.0)) < 0.0);
        // On surface (outermost point along X)
        assert!(torus.signed_distance(Vector3::new(12.0, 0.0, 0.0)).abs() < 1e-3);
        // Origin is outside for R > r
        assert!(torus.signed_distance(Vector3::zeros()) > 0.0);
    }

    #[test]
    fn test_capsule_implicit() {
        let cap = CapsuleImplicit::new(Vector3::zeros(), Vector3::new(0.0, 0.0, 10.0), 1.0);
        assert!(cap.signed_distance(Vector3::new(0.0, 0.0, 5.0)) < 0.0);
        assert!(cap.signed_distance(Vector3::new(2.0, 0.0, 5.0)) > 0.0);
        assert!(cap.signed_distance(Vector3::new(0.0, 0.0, -1.0)).abs() < 1e-3);
    }
}
