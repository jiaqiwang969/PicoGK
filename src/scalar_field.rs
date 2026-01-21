//! Scalar field representation

use crate::{
    ffi, Error, FieldMetadata, ImageGrayScale, Implicit, Library, Result, VoxelDimensions, Voxels,
};
use nalgebra::Vector3;
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

struct ScalarFieldTraverseData {
    ctx: *mut c_void,
    call: fn(*mut c_void, Vector3<f32>, f32),
}

static SCALAR_FIELD_TRAVERSE: AtomicPtr<ScalarFieldTraverseData> =
    AtomicPtr::new(std::ptr::null_mut());

unsafe extern "C" fn scalar_field_trampoline(position: *const crate::types::Vector3f, value: f32) {
    if position.is_null() {
        return;
    }
    let data_ptr = SCALAR_FIELD_TRAVERSE.load(Ordering::SeqCst);
    if data_ptr.is_null() {
        return;
    }
    let data = &mut *data_ptr;
    let pos = Vector3::from(*position);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (data.call)(data.ctx, pos, value);
    }));
}

/// Scalar field
///
/// Stores scalar values (like temperature, pressure) at voxel positions.
pub struct ScalarField {
    handle: *mut ffi::CScalarField,
}

impl ScalarField {
    /// Create an empty scalar field
    pub fn new() -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::ScalarField_hCreate() });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Create a copy of an existing scalar field
    pub fn duplicate(&self) -> Result<Self> {
        let handle =
            crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::ScalarField_hCreateCopy(self.handle) });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Fallible clone of the scalar field (alias for `duplicate()`).
    pub fn try_clone(&self) -> Result<Self> {
        self.duplicate()
    }

    /// Create from voxels
    ///
    /// Converts a voxel field to a scalar field.
    pub fn from_voxels(voxels: &Voxels) -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::ScalarField_hCreateFromVoxels(voxels.handle())
        });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Build a scalar field from voxels, assigning a constant value inside
    pub fn build_from_voxels(voxels: &Voxels, value: f32, sd_threshold: f32) -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::ScalarField_hBuildFromVoxels(voxels.handle(), value, sd_threshold)
        });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Set the value at the specified position (in mm)
    pub fn set_value(&mut self, position: Vector3<f32>, value: f32) {
        let pos = crate::types::Vector3f::from(position);
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::ScalarField_SetValue(self.handle, &pos as *const _, value);
        });
    }

    /// Get the value at the specified position (in mm)
    pub fn get_value(&self, position: Vector3<f32>) -> Option<f32> {
        let pos = crate::types::Vector3f::from(position);
        let mut value = 0.0f32;
        let ok = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::ScalarField_bGetValue(self.handle, &pos as *const _, &mut value)
        });
        if ok {
            Some(value)
        } else {
            None
        }
    }

    /// Remove the value at the specified position (in mm)
    pub fn remove_value(&mut self, position: Vector3<f32>) -> bool {
        let pos = crate::types::Vector3f::from(position);
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::ScalarField_RemoveValue(self.handle, &pos as *const _)
        })
    }

    /// Get voxel dimensions (origin and size)
    pub fn voxel_dimensions(&self) -> VoxelDimensions {
        let mut x_origin = 0;
        let mut y_origin = 0;
        let mut z_origin = 0;
        let mut x_size = 0;
        let mut y_size = 0;
        let mut z_size = 0;
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::ScalarField_GetVoxelDimensions(
                self.handle,
                &mut x_origin,
                &mut y_origin,
                &mut z_origin,
                &mut x_size,
                &mut y_size,
                &mut z_size,
            );
        });
        VoxelDimensions::new(
            Vector3::new(x_origin, y_origin, z_origin),
            Vector3::new(x_size, y_size, z_size),
        )
    }

    /// C#-style alias for `voxel_dimensions`.
    pub fn get_voxel_dimensions(&self) -> VoxelDimensions {
        self.voxel_dimensions()
    }

    /// Get a slice at the specified Z index
    pub fn get_slice(&self, z_slice: i32) -> Result<Vec<f32>> {
        let dims = self.voxel_dimensions();
        let width = dims.size.x.max(0) as usize;
        let height = dims.size.y.max(0) as usize;
        let len = width.saturating_mul(height);
        if len == 0 {
            return Err(Error::OperationFailed(
                "ScalarField slice has zero size".to_string(),
            ));
        }

        let mut values = vec![0.0f32; len];
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::ScalarField_GetSlice(self.handle, z_slice, values.as_mut_ptr());
        });
        Ok(values)
    }

    /// Get a slice as an `ImageGrayScale` (C# `GetVoxelSlice` style).
    pub fn get_voxel_slice(&self, z_slice: i32) -> Result<ImageGrayScale> {
        let dims = self.voxel_dimensions();
        let width = dims.size.x.max(0) as usize;
        let height = dims.size.y.max(0) as usize;
        let values = self.get_slice(z_slice)?;

        let mut img = ImageGrayScale::new(width, height);
        img.values = values;
        Ok(img)
    }

    /// Traverse active values in the field
    pub fn traverse_active<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(Vector3<f32>, f32),
    {
        fn call_trampoline<F: FnMut(Vector3<f32>, f32)>(
            ctx: *mut c_void,
            pos: Vector3<f32>,
            value: f32,
        ) {
            // Safety: `ctx` points to the `callback` stack slot in `traverse_active`.
            let cb = unsafe { &mut *(ctx as *mut F) };
            cb(pos, value);
        }

        let ctx = (&mut callback as *mut F).cast::<c_void>();
        let mut data = ScalarFieldTraverseData {
            ctx,
            call: call_trampoline::<F>,
        };

        let data_ptr = &mut data as *mut ScalarFieldTraverseData;
        let prev = SCALAR_FIELD_TRAVERSE.compare_exchange(
            std::ptr::null_mut(),
            data_ptr,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
        if prev.is_err() {
            return Err(Error::OperationFailed(
                "ScalarField traverse callback already in use".to_string(),
            ));
        }

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::ScalarField_TraverseActive(self.handle, Some(scalar_field_trampoline));
        });

        SCALAR_FIELD_TRAVERSE.store(std::ptr::null_mut(), Ordering::SeqCst);
        Ok(())
    }

    /// Signed distance at the specified position (in mm)
    pub fn signed_distance(&self, position: Vector3<f32>) -> f32 {
        self.get_value(position).unwrap_or(0.0) * Library::voxel_size_mm()
    }

    /// Bounding box of active voxels (in mm)
    pub fn bounding_box(&self) -> crate::BBox3 {
        let dims = self.voxel_dimensions();
        let origin = dims.origin;
        let size = dims.size;
        let min = Library::voxels_to_mm(Vector3::new(
            origin.x as f32,
            origin.y as f32,
            origin.z as f32,
        ));
        let max = Library::voxels_to_mm(Vector3::new(
            (origin.x + size.x) as f32,
            (origin.y + size.y) as f32,
            (origin.z + size.z) as f32,
        ));
        crate::BBox3::new(min, max)
    }

    /// Check if the scalar field is valid
    pub fn is_valid(&self) -> bool {
        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::ScalarField_bIsValid(self.handle) })
    }

    /// Get field metadata
    pub fn metadata(&self) -> Result<FieldMetadata> {
        FieldMetadata::from_scalar_field(self)
    }

    /// C#-style alias for `metadata`.
    pub fn meta_data(&self) -> Result<FieldMetadata> {
        self.metadata()
    }

    /// C#-style alias for `metadata`.
    pub fn o_meta_data(&self) -> Result<FieldMetadata> {
        self.metadata()
    }

    /// Get raw handle (for internal use)
    pub(crate) fn handle(&self) -> *mut ffi::CScalarField {
        self.handle
    }

    /// Create from raw handle (for internal use)
    ///
    /// # Safety
    ///
    /// The handle must be a valid CScalarField pointer.
    /// This function takes ownership of the handle.
    pub(crate) fn from_handle(handle: *mut ffi::CScalarField) -> Self {
        Self { handle }
    }
}

impl Implicit for ScalarField {
    fn signed_distance(&self, point: Vector3<f32>) -> f32 {
        self.signed_distance(point)
    }

    fn bounds(&self) -> Option<crate::BBox3> {
        Some(self.bounding_box())
    }
}

impl Drop for ScalarField {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::ScalarField_Destroy(self.handle);
            });
        }
    }
}

unsafe impl Send for ScalarField {}
unsafe impl Sync for ScalarField {}

// NOTE: We intentionally do not implement `Clone` for `ScalarField`.
// Cloning requires an infallible operation, while duplicating a native object can
// fail (e.g. out-of-memory / null handle). Use `duplicate()` / `try_clone()`.

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_scalar_field_creation() {
        let _lib = Library::init(0.5).unwrap();
        let field = ScalarField::new();
        assert!(field.is_ok());
    }
}
