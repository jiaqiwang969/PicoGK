//! Colored 3D polyline representation

use crate::{ffi, ColorFloat, Error, Result};
use nalgebra::Vector3;

/// Colored 3D polyline
pub struct PolyLine {
    handle: *mut ffi::CPolyLine,
    bbox: crate::BBox3,
}

impl PolyLine {
    /// Create a new polyline with the specified color
    pub fn new(color: ColorFloat) -> Result<Self> {
        let handle =
            crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::PolyLine_hCreate(&color as *const _) });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self {
            handle,
            bbox: crate::BBox3::empty(),
        })
    }

    /// Add a vertex to the polyline
    pub fn add_vertex(&mut self, vertex: Vector3<f32>) -> i32 {
        self.bbox.include_point(vertex);
        let vec = crate::types::Vector3f::from(vertex);
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::PolyLine_nAddVertex(self.handle, &vec as *const _)
        })
    }

    /// C#-style alias for `add_vertex`.
    pub fn add(&mut self, vertex: Vector3<f32>) -> i32 {
        self.add_vertex(vertex)
    }

    /// Add vertices from an iterator
    pub fn add_vertices<I>(&mut self, vertices: I)
    where
        I: IntoIterator<Item = Vector3<f32>>,
    {
        for vertex in vertices {
            self.add_vertex(vertex);
        }
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::PolyLine_nVertexCount(self.handle) as usize
        })
    }

    /// Get the vertex at the specified index
    pub fn vertex_at(&self, index: usize) -> Option<Vector3<f32>> {
        if index >= self.vertex_count() {
            return None;
        }
        let mut vec = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::PolyLine_GetVertex(self.handle, index as i32, &mut vec as *mut _);
        });
        Some(Vector3::from(vec))
    }

    /// Get the polyline color
    pub fn color(&self) -> ColorFloat {
        let mut color = ColorFloat::new(0.0, 0.0, 0.0, 0.0);
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::PolyLine_GetColor(self.handle, &mut color as *mut _);
        });
        color
    }

    /// C#-style alias for `color`.
    pub fn get_color(&self) -> ColorFloat {
        self.color()
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> crate::BBox3 {
        self.bbox
    }

    /// Add an arrow at the end of the polyline
    pub fn add_arrow(&mut self, size_mm: f32, direction: Option<Vector3<f32>>) {
        if self.vertex_count() < 1 {
            return;
        }
        if direction.is_none() && self.vertex_count() < 2 {
            return;
        }

        let mut dir = direction.unwrap_or(Vector3::new(0.0, 0.0, 1.0));
        if direction.is_none() {
            let start = match self.vertex_at(self.vertex_count() - 2) {
                Some(v) => v,
                None => return,
            };
            let end = match self.vertex_at(self.vertex_count() - 1) {
                Some(v) => v,
                None => return,
            };
            dir = end - start;
            if dir.norm() <= 1e-6 {
                return;
            }
        }

        dir = dir.normalize();

        let mut init = Vector3::new(1.0, 0.0, 0.0);
        if dir == Vector3::new(1.0, 0.0, 0.0) || dir == Vector3::new(-1.0, 0.0, 0.0) {
            init = Vector3::new(0.0, 1.0, 0.0);
        }

        let u = dir.cross(&init).normalize();
        let v = dir.cross(&u).normalize();

        let tip = match self.vertex_at(self.vertex_count() - 1) {
            Some(v) => v,
            None => return,
        };
        let base = tip - dir * size_mm;

        self.add_vertex(base + u * (size_mm * 0.5));
        self.add_vertex(base - u * (size_mm * 0.5));
        self.add_vertex(tip);
        self.add_vertex(base + v * (size_mm * 0.5));
        self.add_vertex(base - v * (size_mm * 0.5));
        self.add_vertex(tip);
    }

    /// Add a cross at the end of the polyline
    pub fn add_cross(&mut self, size_mm: f32) {
        if self.vertex_count() < 1 {
            return;
        }
        let center = match self.vertex_at(self.vertex_count() - 1) {
            Some(v) => v,
            None => return,
        };
        self.add_vertex(center + Vector3::new(size_mm, 0.0, 0.0));
        self.add_vertex(center - Vector3::new(size_mm, 0.0, 0.0));
        self.add_vertex(center);
        self.add_vertex(center + Vector3::new(0.0, size_mm, 0.0));
        self.add_vertex(center - Vector3::new(0.0, size_mm, 0.0));
        self.add_vertex(center);
        self.add_vertex(center + Vector3::new(0.0, 0.0, size_mm));
        self.add_vertex(center - Vector3::new(0.0, 0.0, size_mm));
        self.add_vertex(center);
    }

    /// Check if the polyline is valid
    pub fn is_valid(&self) -> bool {
        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::PolyLine_bIsValid(self.handle) })
    }

    /// Get raw handle (for internal use)
    pub(crate) fn handle(&self) -> *mut ffi::CPolyLine {
        self.handle
    }
}

impl Drop for PolyLine {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::PolyLine_Destroy(self.handle);
            });
        }
    }
}

unsafe impl Send for PolyLine {}
unsafe impl Sync for PolyLine {}
