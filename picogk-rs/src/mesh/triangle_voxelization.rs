//! Triangle voxelization helpers

use crate::{BBox3, Error, Implicit, Result, Voxels};
use nalgebra::Vector3;

use super::Mesh;

impl Mesh {
    /// Voxelize the mesh as a hollow shell with the specified thickness
    pub fn voxelize_hollow(&self, thickness: f32) -> Result<Voxels> {
        if thickness <= 0.0 {
            return Err(Error::InvalidParameter(
                "thickness must be positive".to_string(),
            ));
        }
        let implicit = ImplicitMesh::new(self, thickness)?;
        Voxels::from_implicit(&implicit)
    }
}

/// Treat a mesh as an implicit shell
pub struct ImplicitMesh {
    triangles: Vec<ImplicitTriangle>,
    bounds: BBox3,
}

impl ImplicitMesh {
    pub fn new(mesh: &Mesh, thickness: f32) -> Result<Self> {
        let mut triangles = Vec::with_capacity(mesh.triangle_count());
        let mut bounds = BBox3::empty();

        for index in 0..mesh.triangle_count() {
            let (a, b, c) = mesh.get_triangle_vertices(index)?;
            let triangle = ImplicitTriangle::new(a, b, c, thickness);
            bounds.include_bbox(&triangle.bounds);
            triangles.push(triangle);
        }

        Ok(Self { triangles, bounds })
    }
}

impl Implicit for ImplicitMesh {
    fn signed_distance(&self, point: Vector3<f32>) -> f32 {
        let mut dist = f32::MAX;
        for triangle in &self.triangles {
            dist = dist.min(triangle.signed_distance(point));
        }
        dist
    }

    fn bounds(&self) -> Option<BBox3> {
        Some(self.bounds)
    }
}

/// Treat a triangle as an implicit shell
pub struct ImplicitTriangle {
    a: Vector3<f32>,
    b: Vector3<f32>,
    c: Vector3<f32>,
    thickness: f32,
    bounds: BBox3,
}

impl ImplicitTriangle {
    pub fn new(a: Vector3<f32>, b: Vector3<f32>, c: Vector3<f32>, thickness: f32) -> Self {
        let mut bounds = BBox3::empty();
        bounds.include_point(a);
        bounds.include_point(b);
        bounds.include_point(c);
        bounds.grow(thickness);

        Self {
            a,
            b,
            c,
            thickness,
            bounds,
        }
    }

    fn closest_point(
        point: Vector3<f32>,
        a: Vector3<f32>,
        b: Vector3<f32>,
        c: Vector3<f32>,
    ) -> Vector3<f32> {
        let ab = b - a;
        let ac = c - a;
        let ap = point - a;

        let d1 = ab.dot(&ap);
        let d2 = ac.dot(&ap);
        if d1 <= 0.0 && d2 <= 0.0 {
            return a;
        }

        let bp = point - b;
        let d3 = ab.dot(&bp);
        let d4 = ac.dot(&bp);
        if d3 >= 0.0 && d4 <= d3 {
            return b;
        }

        let vc = d1 * d4 - d3 * d2;
        if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
            let v = d1 / (d1 - d3);
            return a + v * ab;
        }

        let cp = point - c;
        let d5 = ab.dot(&cp);
        let d6 = ac.dot(&cp);
        if d6 >= 0.0 && d5 <= d6 {
            return c;
        }

        let vb = d5 * d2 - d1 * d6;
        if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
            let w = d2 / (d2 - d6);
            return a + w * ac;
        }

        let va = d3 * d6 - d5 * d4;
        if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
            let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
            return b + w * (c - b);
        }

        let denom = 1.0 / (va + vb + vc);
        let v_ab = vb * denom;
        let v_ac = vc * denom;
        a + v_ab * ab + v_ac * ac
    }
}

impl Implicit for ImplicitTriangle {
    fn signed_distance(&self, point: Vector3<f32>) -> f32 {
        let closest = Self::closest_point(point, self.a, self.b, self.c);
        let dist = (point - closest).norm();
        dist - self.thickness
    }

    fn bounds(&self) -> Option<BBox3> {
        Some(self.bounds)
    }
}
