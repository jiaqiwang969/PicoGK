//! Vector field representation

use crate::{ffi, Error, FieldMetadata, Result, Voxels};
use nalgebra::Vector3;
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

struct VectorFieldTraverseData {
    ctx: *mut c_void,
    call: fn(*mut c_void, Vector3<f32>, Vector3<f32>),
}

static VECTOR_FIELD_TRAVERSE: AtomicPtr<VectorFieldTraverseData> =
    AtomicPtr::new(std::ptr::null_mut());

unsafe extern "C" fn vector_field_trampoline(
    position: *const crate::types::Vector3f,
    value: *const crate::types::Vector3f,
) {
    if position.is_null() || value.is_null() {
        return;
    }
    let data_ptr = VECTOR_FIELD_TRAVERSE.load(Ordering::SeqCst);
    if data_ptr.is_null() {
        return;
    }
    let data = &mut *data_ptr;
    let pos = Vector3::from(*position);
    let val = Vector3::from(*value);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (data.call)(data.ctx, pos, val);
    }));
}

/// Vector field
///
/// Stores vector values at voxel positions.
pub struct VectorField {
    handle: *mut ffi::CVectorField,
}

impl VectorField {
    /// Create an empty vector field
    pub fn new() -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::VectorField_hCreate() });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Create a copy of an existing vector field
    pub fn duplicate(&self) -> Result<Self> {
        let handle =
            crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::VectorField_hCreateCopy(self.handle) });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Fallible clone of the vector field (alias for `duplicate()`).
    pub fn try_clone(&self) -> Result<Self> {
        self.duplicate()
    }

    /// Create a vector field from voxels (gradient field)
    pub fn from_voxels(voxels: &Voxels) -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VectorField_hCreateFromVoxels(voxels.handle())
        });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Build a vector field from voxels, assigning a constant value inside
    pub fn build_from_voxels(
        voxels: &Voxels,
        value: Vector3<f32>,
        sd_threshold: f32,
    ) -> Result<Self> {
        let vec = crate::types::Vector3f::from(value);
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VectorField_hBuildFromVoxels(
                voxels.handle(),
                &vec as *const crate::types::Vector3f,
                sd_threshold,
            )
        });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Set the value at the specified position (in mm)
    pub fn set_value(&mut self, position: Vector3<f32>, value: Vector3<f32>) {
        let pos = crate::types::Vector3f::from(position);
        let val = crate::types::Vector3f::from(value);
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VectorField_SetValue(
                self.handle,
                &pos as *const crate::types::Vector3f,
                &val as *const crate::types::Vector3f,
            );
        });
    }

    /// Get the value at the specified position (in mm)
    pub fn get_value(&self, position: Vector3<f32>) -> Option<Vector3<f32>> {
        let pos = crate::types::Vector3f::from(position);
        let mut val = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let ok = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VectorField_bGetValue(
                self.handle,
                &pos as *const crate::types::Vector3f,
                &mut val as *mut crate::types::Vector3f,
            )
        });
        if ok {
            Some(Vector3::from(val))
        } else {
            None
        }
    }

    /// Remove the value at the specified position (in mm)
    pub fn remove_value(&mut self, position: Vector3<f32>) -> bool {
        let pos = crate::types::Vector3f::from(position);
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VectorField_RemoveValue(self.handle, &pos as *const crate::types::Vector3f)
        })
    }

    /// Traverse active values in the field
    pub fn traverse_active<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(Vector3<f32>, Vector3<f32>),
    {
        fn call_trampoline<F: FnMut(Vector3<f32>, Vector3<f32>)>(
            ctx: *mut c_void,
            pos: Vector3<f32>,
            value: Vector3<f32>,
        ) {
            // Safety: `ctx` points to the `callback` stack slot in `traverse_active`.
            let cb = unsafe { &mut *(ctx as *mut F) };
            cb(pos, value);
        }

        let ctx = (&mut callback as *mut F).cast::<c_void>();
        let mut data = VectorFieldTraverseData {
            ctx,
            call: call_trampoline::<F>,
        };

        let data_ptr = &mut data as *mut VectorFieldTraverseData;
        let prev = VECTOR_FIELD_TRAVERSE.compare_exchange(
            std::ptr::null_mut(),
            data_ptr,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
        if prev.is_err() {
            return Err(Error::OperationFailed(
                "VectorField traverse callback already in use".to_string(),
            ));
        }

        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VectorField_TraverseActive(self.handle, Some(vector_field_trampoline));
        });

        VECTOR_FIELD_TRAVERSE.store(std::ptr::null_mut(), Ordering::SeqCst);
        Ok(())
    }

    /// Check if the vector field is valid
    pub fn is_valid(&self) -> bool {
        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::VectorField_bIsValid(self.handle) })
    }

    /// Get field metadata
    pub fn metadata(&self) -> Result<FieldMetadata> {
        FieldMetadata::from_vector_field(self)
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
    pub(crate) fn handle(&self) -> *mut ffi::CVectorField {
        self.handle
    }

    /// Create from raw handle (for internal use)
    ///
    /// # Safety
    ///
    /// The handle must be a valid CVectorField pointer.
    /// This function takes ownership of the handle.
    pub(crate) fn from_handle(handle: *mut ffi::CVectorField) -> Self {
        Self { handle }
    }
}

impl Drop for VectorField {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::VectorField_Destroy(self.handle);
            });
        }
    }
}

unsafe impl Send for VectorField {}
unsafe impl Sync for VectorField {}

// NOTE: We intentionally do not implement `Clone` for `VectorField`.
// Cloning requires an infallible operation, while duplicating a native object can
// fail (e.g. out-of-memory / null handle). Use `duplicate()` / `try_clone()`.
