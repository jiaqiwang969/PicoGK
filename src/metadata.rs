//! Field metadata storage

use crate::{ffi, Error, Result, ScalarField, VectorField, Voxels};
use nalgebra::Vector3;
use std::ffi::{CStr, CString};

/// Metadata value type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataType {
    Unknown = -1,
    String = 0,
    Float = 1,
    Vector = 2,
}

impl From<i32> for MetadataType {
    fn from(value: i32) -> Self {
        match value {
            0 => MetadataType::String,
            1 => MetadataType::Float,
            2 => MetadataType::Vector,
            _ => MetadataType::Unknown,
        }
    }
}

/// Metadata value container (C# `SetValue` / `bGetValueAt` equivalent).
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    String(String),
    Float(f32),
    Vector(Vector3<f32>),
}

impl From<&str> for MetadataValue {
    fn from(value: &str) -> Self {
        MetadataValue::String(value.to_string())
    }
}

impl From<String> for MetadataValue {
    fn from(value: String) -> Self {
        MetadataValue::String(value)
    }
}

impl From<f32> for MetadataValue {
    fn from(value: f32) -> Self {
        MetadataValue::Float(value)
    }
}

impl From<Vector3<f32>> for MetadataValue {
    fn from(value: Vector3<f32>) -> Self {
        MetadataValue::Vector(value)
    }
}

/// Metadata associated with fields (voxels, scalar fields, vector fields)
pub struct FieldMetadata {
    handle: *mut ffi::CMetadata,
}

fn guard_internal_fields(name: &str) -> Result<()> {
    let lower = name.to_ascii_lowercase();

    if lower.starts_with("picogk.") {
        return Err(Error::InvalidParameter(format!(
            "Fields starting with 'PicoGK.' are internal - do not set them from your code ('{}')",
            name
        )));
    }

    if lower == "class" || lower == "name" {
        return Err(Error::InvalidParameter(format!(
            "Do not set openvdb-internal fields from your code ('{}')",
            name
        )));
    }

    if lower.starts_with("file_") {
        return Err(Error::InvalidParameter(format!(
            "Field names starting with file_ are openvdb-internal - do not set from your code ('{}')",
            name
        )));
    }

    Ok(())
}

impl FieldMetadata {
    pub fn from_voxels(voxels: &Voxels) -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_hFromVoxels(voxels.handle())
        });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    pub fn from_scalar_field(field: &ScalarField) -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_hFromScalarField(field.handle())
        });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    pub fn from_vector_field(field: &VectorField) -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_hFromVectorField(field.handle())
        });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    pub fn count(&self) -> usize {
        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::Metadata_nCount(self.handle) as usize })
    }

    /// C#-style alias for `count`.
    pub fn n_count(&self) -> usize {
        self.count()
    }

    pub fn name_at(&self, index: usize) -> Result<String> {
        let len = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_nNameLengthAt(self.handle, index as i32)
        });
        if len < 0 {
            return Err(Error::InvalidParameter(format!(
                "Invalid metadata index {}",
                index
            )));
        }
        let mut buffer = vec![0u8; len as usize + 1];
        let ok = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_bGetNameAt(
                self.handle,
                index as i32,
                buffer.as_mut_ptr() as *mut i8,
                buffer.len() as i32,
            )
        });
        if !ok {
            return Err(Error::OperationFailed(format!(
                "Failed to read metadata name at index {}",
                index
            )));
        }
        let cstr = unsafe { CStr::from_ptr(buffer.as_ptr() as *const i8) };
        Ok(cstr.to_string_lossy().to_string())
    }

    /// C# `bGetNameAt`-style API.
    ///
    /// Returns `Ok(None)` when the index is out of range.
    pub fn b_get_name_at(&self, index: usize) -> Result<Option<String>> {
        match self.name_at(index) {
            Ok(name) => Ok(Some(name)),
            Err(Error::InvalidParameter(_)) | Err(Error::OperationFailed(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn names(&self) -> Result<Vec<String>> {
        let mut names = Vec::with_capacity(self.count());
        for i in 0..self.count() {
            names.push(self.name_at(i)?);
        }
        Ok(names)
    }

    pub fn value_type(&self, name: &str) -> Result<MetadataType> {
        let c_name = CString::new(name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;
        let value = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_nTypeAt(self.handle, c_name.as_ptr())
        });
        Ok(MetadataType::from(value))
    }

    /// C#-style alias for `value_type` (`eTypeAt`).
    pub fn e_type_at(&self, name: &str) -> Result<MetadataType> {
        self.value_type(name)
    }

    /// C#-style alias for `value_type` (`TypeAt` without the Hungarian prefix).
    pub fn type_at(&self, name: &str) -> Result<MetadataType> {
        self.value_type(name)
    }

    /// C# `strTypeAt` equivalent.
    pub fn str_type_at(&self, name: &str) -> Result<String> {
        Ok(self.str_type_name(self.e_type_at(name)?))
    }

    /// C# `strTypeName` equivalent.
    pub fn str_type_name(&self, ty: MetadataType) -> String {
        match ty {
            MetadataType::Unknown => "unknown".to_string(),
            MetadataType::String => "string".to_string(),
            MetadataType::Float => "float".to_string(),
            MetadataType::Vector => "vector".to_string(),
        }
    }

    /// Alias for `str_type_name`.
    pub fn type_name(&self, ty: MetadataType) -> String {
        self.str_type_name(ty)
    }

    pub fn get_string(&self, name: &str) -> Result<Option<String>> {
        let c_name = CString::new(name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;
        let len = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_nStringLengthAt(self.handle, c_name.as_ptr())
        });
        if len <= 0 {
            return Ok(None);
        }
        let mut buffer = vec![0u8; len as usize + 1];
        let ok = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_bGetStringAt(
                self.handle,
                c_name.as_ptr(),
                buffer.as_mut_ptr() as *mut i8,
                buffer.len() as i32,
            )
        });
        if !ok {
            return Ok(None);
        }
        let cstr = unsafe { CStr::from_ptr(buffer.as_ptr() as *const i8) };
        Ok(Some(cstr.to_string_lossy().to_string()))
    }

    pub fn get_float(&self, name: &str) -> Result<Option<f32>> {
        let c_name = CString::new(name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;
        let mut value = 0.0f32;
        let ok = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_bGetFloatAt(self.handle, c_name.as_ptr(), &mut value)
        });
        if ok {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn get_vector(&self, name: &str) -> Result<Option<Vector3<f32>>> {
        let c_name = CString::new(name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;
        let mut value = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let ok = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_bGetVectorAt(self.handle, c_name.as_ptr(), &mut value)
        });
        if ok {
            Ok(Some(Vector3::from(value)))
        } else {
            Ok(None)
        }
    }

    /// Return a typed value from the metadata table (C# `bGetValueAt` equivalent).
    pub fn get_value_at(&self, name: &str) -> Result<Option<MetadataValue>> {
        match self.value_type(name)? {
            MetadataType::Unknown => Ok(None),
            MetadataType::String => Ok(self.get_string(name)?.map(MetadataValue::String)),
            MetadataType::Float => Ok(self.get_float(name)?.map(MetadataValue::Float)),
            MetadataType::Vector => Ok(self.get_vector(name)?.map(MetadataValue::Vector)),
        }
    }

    /// C# `bGetValueAt` style alias for `get_value_at`.
    pub fn b_get_value_at(&self, name: &str) -> Result<Option<MetadataValue>> {
        self.get_value_at(name)
    }

    pub fn set_string(&mut self, name: &str, value: &str) -> Result<()> {
        guard_internal_fields(name)?;
        let c_name = CString::new(name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;
        let c_value = CString::new(value)
            .map_err(|_| Error::InvalidParameter("Value contains null byte".to_string()))?;
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_SetStringValue(self.handle, c_name.as_ptr(), c_value.as_ptr());
        });
        Ok(())
    }

    pub(crate) fn set_string_internal(&mut self, name: &str, value: &str) -> Result<()> {
        let c_name = CString::new(name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;
        let c_value = CString::new(value)
            .map_err(|_| Error::InvalidParameter("Value contains null byte".to_string()))?;
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_SetStringValue(self.handle, c_name.as_ptr(), c_value.as_ptr());
        });
        Ok(())
    }

    /// Set a value in the metadata table (C# `SetValue`).
    pub fn set_value<V: Into<MetadataValue>>(&mut self, name: &str, value: V) -> Result<()> {
        match value.into() {
            MetadataValue::String(s) => self.set_string(name, &s),
            MetadataValue::Float(f) => self.set_float(name, f),
            MetadataValue::Vector(v) => self.set_vector(name, v),
        }
    }

    pub fn set_float(&mut self, name: &str, value: f32) -> Result<()> {
        guard_internal_fields(name)?;
        let c_name = CString::new(name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_SetFloatValue(self.handle, c_name.as_ptr(), value);
        });
        Ok(())
    }

    pub(crate) fn set_float_internal(&mut self, name: &str, value: f32) -> Result<()> {
        let c_name = CString::new(name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_SetFloatValue(self.handle, c_name.as_ptr(), value);
        });
        Ok(())
    }

    pub fn set_vector(&mut self, name: &str, value: Vector3<f32>) -> Result<()> {
        guard_internal_fields(name)?;
        let c_name = CString::new(name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;
        let vec = crate::types::Vector3f::from(value);
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Metadata_SetVectorValue(self.handle, c_name.as_ptr(), &vec as *const _);
        });
        Ok(())
    }

    pub fn remove_value(&mut self, name: &str) -> Result<()> {
        guard_internal_fields(name)?;
        let c_name = CString::new(name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::MetaData_RemoveValue(self.handle, c_name.as_ptr());
        });
        Ok(())
    }
}

impl Drop for FieldMetadata {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::Metadata_Destroy(self.handle);
            });
        }
    }
}

unsafe impl Send for FieldMetadata {}
unsafe impl Sync for FieldMetadata {}

#[cfg(test)]
mod tests {
    use super::guard_internal_fields;

    #[test]
    fn test_guard_internal_fields() {
        assert!(guard_internal_fields("PicoGK.internal").is_err());
        assert!(guard_internal_fields("picogk.internal").is_err());
        assert!(guard_internal_fields("class").is_err());
        assert!(guard_internal_fields("Name").is_err());
        assert!(guard_internal_fields("file_foo").is_err());
        assert!(guard_internal_fields("FILE_bar").is_err());

        assert!(guard_internal_fields("user.field").is_ok());
        assert!(guard_internal_fields("foo").is_ok());
    }
}
