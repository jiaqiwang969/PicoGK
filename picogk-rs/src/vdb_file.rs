//! OpenVDB file I/O
//!
//! This module provides functionality for reading and writing OpenVDB (.vdb) files.
//! VDB files can contain multiple fields of different types (Voxels, ScalarField, etc.).

use crate::{ffi, Error, Result, ScalarField, VectorField, Voxels};
use std::ffi::CString;
use std::path::Path;

/// Field type in a VDB file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    /// Unsupported field type
    Unsupported = -1,
    /// Voxels field (GRID_LEVEL_SET)
    Voxels = 0,
    /// ScalarField
    ScalarField = 1,
    /// VectorField
    VectorField = 2,
}

impl From<i32> for FieldType {
    fn from(value: i32) -> Self {
        match value {
            0 => FieldType::Voxels,
            1 => FieldType::ScalarField,
            2 => FieldType::VectorField,
            _ => FieldType::Unsupported,
        }
    }
}

/// A typed field returned from a VDB container (C# `xField` equivalent).
pub enum VdbField {
    Voxels(Voxels),
    ScalarField(ScalarField),
    VectorField(VectorField),
}

/// OpenVDB file container
///
/// Handles reading and writing of OpenVDB (.vdb) files.
/// VDB files can contain multiple fields of different types.
///
/// # Example
///
/// ```rust,no_run
/// use picogk::{Library, Voxels, VdbFile};
/// use nalgebra::Vector3;
///
/// let _lib = Library::init(0.5)?;
///
/// // Create and save
/// let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;
/// let mut vdb = VdbFile::new()?;
/// vdb.add_voxels(&sphere, "my_sphere")?;
/// vdb.save("output.vdb")?;
///
/// // Load
/// let vdb = VdbFile::load("output.vdb")?;
/// let loaded = vdb.get_voxels(0)?;
/// # Ok::<(), picogk::Error>(())
/// ```
pub struct VdbFile {
    handle: *mut ffi::CVdbFile,
}

impl VdbFile {
    /// Create a new empty VDB file
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::VdbFile;
    ///
    /// let vdb = VdbFile::new()?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::VdbFile_hCreate() });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Load a VDB file from disk
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .vdb file
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::VdbFile;
    ///
    /// let vdb = VdbFile::load("input.vdb")?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| Error::InvalidParameter("Invalid path".to_string()))?;
        let c_path = CString::new(path_str)
            .map_err(|_| Error::InvalidParameter("Path contains null byte".to_string()))?;

        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VdbFile_hCreateFromFile(c_path.as_ptr())
        });
        if handle.is_null() {
            return Err(Error::FileLoad(format!(
                "Failed to load VDB file: {}",
                path_str
            )));
        }
        Ok(Self { handle })
    }

    /// Save the VDB file to disk
    ///
    /// # Arguments
    ///
    /// * `path` - Path where to save the .vdb file
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{VdbFile, Voxels};
    /// use nalgebra::Vector3;
    ///
    /// let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;
    /// let mut vdb = VdbFile::new()?;
    /// vdb.add_voxels(&sphere, "sphere")?;
    /// vdb.save("output.vdb")?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| Error::InvalidParameter("Invalid path".to_string()))?;
        let c_path = CString::new(path_str)
            .map_err(|_| Error::InvalidParameter("Path contains null byte".to_string()))?;

        // Match C# `OpenVdbFile.SaveToFile`: stamp PicoGK metadata for compatibility.
        //
        // OpenVDB metadata is stored in SI units, so voxel size is stored in meters.
        let lib_name = crate::Library::name();
        let version = crate::Library::version();
        let voxel_size_m = crate::Library::voxel_size_mm() / 1000.0;

        for i in 0..self.field_count() {
            match self.field_type(i) {
                FieldType::Voxels => {
                    let field = self.get_voxels(i)?;
                    let mut md = field.metadata()?;
                    md.set_string_internal("PicoGK.Library", &lib_name)?;
                    md.set_string_internal("PicoGK.Version", &version)?;
                    md.set_float_internal("PicoGK.VoxelSize", voxel_size_m)?;
                }
                FieldType::ScalarField => {
                    let field = self.get_scalar_field(i)?;
                    let mut md = field.metadata()?;
                    md.set_string_internal("PicoGK.Library", &lib_name)?;
                    md.set_string_internal("PicoGK.Version", &version)?;
                    md.set_float_internal("PicoGK.VoxelSize", voxel_size_m)?;
                }
                FieldType::VectorField => {
                    let field = self.get_vector_field(i)?;
                    let mut md = field.metadata()?;
                    md.set_string_internal("PicoGK.Library", &lib_name)?;
                    md.set_string_internal("PicoGK.Version", &version)?;
                    md.set_float_internal("PicoGK.VoxelSize", voxel_size_m)?;
                }
                FieldType::Unsupported => {}
            }
        }

        let success = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VdbFile_bSaveToFile(self.handle, c_path.as_ptr())
        });
        if !success {
            return Err(Error::FileSave(format!(
                "Failed to save VDB file: {}",
                path_str
            )));
        }
        Ok(())
    }

    /// C#-style alias for `save`.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.save(path)
    }

    /// C#-style alias for `field_type`.
    pub fn e_field_type(&self, index: usize) -> FieldType {
        self.field_type(index)
    }

    /// Return a typed field wrapper for the given index (C# `xField`).
    pub fn x_field(&self, index: usize) -> Result<VdbField> {
        if index >= self.field_count() {
            return Err(Error::InvalidParameter(format!(
                "Index {} out of range",
                index
            )));
        }

        match self.field_type(index) {
            FieldType::Voxels => Ok(VdbField::Voxels(self.get_voxels(index)?)),
            FieldType::ScalarField => Ok(VdbField::ScalarField(self.get_scalar_field(index)?)),
            FieldType::VectorField => Ok(VdbField::VectorField(self.get_vector_field(index)?)),
            FieldType::Unsupported => Err(Error::InvalidParameter(format!(
                "Unsupported field at index {}",
                index
            ))),
        }
    }

    /// Return whether this VDB file looks PicoGK-compatible (C# `bIsPicoGKCompatible`).
    pub fn is_pico_gk_compatible(&self) -> bool {
        if self.field_count() < 1 {
            return false;
        }

        let value_m = match self.x_field(0) {
            Ok(VdbField::Voxels(v)) => v
                .metadata()
                .ok()
                .and_then(|md| md.get_float("PicoGK.VoxelSize").ok().flatten()),
            Ok(VdbField::ScalarField(v)) => v
                .metadata()
                .ok()
                .and_then(|md| md.get_float("PicoGK.VoxelSize").ok().flatten()),
            Ok(VdbField::VectorField(v)) => v
                .metadata()
                .ok()
                .and_then(|md| md.get_float("PicoGK.VoxelSize").ok().flatten()),
            Err(_) => None,
        };

        value_m.is_some_and(|v| v > 0.0)
    }

    /// Return the voxel size in millimeters stored in the VDB metadata (C# `fPicoGKVoxelSizeMM`).
    ///
    /// The value is stored in the VDB as meters; this returns millimeters.
    pub fn pico_gk_voxel_size_mm(&self) -> f32 {
        if self.field_count() < 1 {
            return 0.0;
        }

        let value_m = match self.x_field(0) {
            Ok(VdbField::Voxels(v)) => v
                .metadata()
                .ok()
                .and_then(|md| md.get_float("PicoGK.VoxelSize").ok().flatten()),
            Ok(VdbField::ScalarField(v)) => v
                .metadata()
                .ok()
                .and_then(|md| md.get_float("PicoGK.VoxelSize").ok().flatten()),
            Ok(VdbField::VectorField(v)) => v
                .metadata()
                .ok()
                .and_then(|md| md.get_float("PicoGK.VoxelSize").ok().flatten()),
            Err(_) => None,
        };

        value_m.unwrap_or(0.0) * 1000.0
    }

    /// Get the number of fields in the VDB file
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::VdbFile;
    ///
    /// let vdb = VdbFile::load("input.vdb")?;
    /// println!("Fields: {}", vdb.field_count());
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn field_count(&self) -> usize {
        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::VdbFile_nFieldCount(self.handle) as usize })
    }

    /// Get the type of a field at the specified index
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the field (0-based)
    pub fn field_type(&self, index: usize) -> FieldType {
        let type_id = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VdbFile_nFieldType(self.handle, index as i32)
        });
        FieldType::from(type_id)
    }

    /// Get the name of a field at the specified index
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the field (0-based)
    pub fn field_name(&self, index: usize) -> String {
        let mut buffer = vec![0u8; 256];
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VdbFile_GetFieldName(self.handle, index as i32, buffer.as_mut_ptr() as *mut i8);
        });

        // Find the null terminator
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        String::from_utf8_lossy(&buffer[..len]).to_string()
    }

    /// Get Voxels from a field at the specified index
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the field (0-based)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::VdbFile;
    ///
    /// let vdb = VdbFile::load("input.vdb")?;
    /// let voxels = vdb.get_voxels(0)?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn get_voxels(&self, index: usize) -> Result<Voxels> {
        if index >= self.field_count() {
            return Err(Error::InvalidParameter(format!(
                "Index {} out of range",
                index
            )));
        }

        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VdbFile_hGetVoxels(self.handle, index as i32)
        });
        if handle.is_null() {
            return Err(Error::InvalidParameter(format!(
                "No voxels at index {}",
                index
            )));
        }

        Ok(Voxels::from_handle(handle))
    }

    /// C#-style alias for `get_voxels` (matches `voxGet` naming).
    pub fn vox_get(&self, index: usize) -> Result<Voxels> {
        self.get_voxels(index)
    }

    /// Get Voxels from a field with the specified name
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the field
    pub fn get_voxels_by_name(&self, name: &str) -> Result<Voxels> {
        for i in 0..self.field_count() {
            if self.field_name(i) == name {
                return self.get_voxels(i);
            }
        }
        Err(Error::InvalidParameter(format!(
            "No field named '{}'",
            name
        )))
    }

    /// Add Voxels to the VDB file
    ///
    /// # Arguments
    ///
    /// * `voxels` - Voxels to add
    /// * `name` - Name for the field (if empty, auto-generates a name)
    ///
    /// # Returns
    ///
    /// Index of the added field
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{VdbFile, Voxels};
    /// use nalgebra::Vector3;
    ///
    /// let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;
    /// let mut vdb = VdbFile::new()?;
    /// vdb.add_voxels(&sphere, "my_sphere")?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn add_voxels(&mut self, voxels: &Voxels, name: &str) -> Result<usize> {
        let field_name = if name.is_empty() {
            format!("PicoGK.Voxels.{}", self.field_count())
        } else {
            name.to_string()
        };

        let c_name = CString::new(field_name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;

        let index = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VdbFile_nAddVoxels(self.handle, c_name.as_ptr(), voxels.handle())
        });

        if index < 0 {
            return Err(Error::InvalidParameter("Failed to add voxels".to_string()));
        }

        Ok(index as usize)
    }

    /// C#-style alias for `add_voxels` (matches `nAdd` naming for voxel fields).
    pub fn n_add(&mut self, voxels: &Voxels, name: &str) -> Result<usize> {
        self.add_voxels(voxels, name)
    }

    /// Get ScalarField from a field at the specified index
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the field (0-based)
    pub fn get_scalar_field(&self, index: usize) -> Result<ScalarField> {
        if index >= self.field_count() {
            return Err(Error::InvalidParameter(format!(
                "Index {} out of range",
                index
            )));
        }

        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VdbFile_hGetScalarField(self.handle, index as i32)
        });
        if handle.is_null() {
            return Err(Error::InvalidParameter(format!(
                "No scalar field at index {}",
                index
            )));
        }

        Ok(ScalarField::from_handle(handle))
    }

    /// Add ScalarField to the VDB file
    ///
    /// # Arguments
    ///
    /// * `field` - ScalarField to add
    /// * `name` - Name for the field (if empty, auto-generates a name)
    ///
    /// # Returns
    ///
    /// Index of the added field
    pub fn add_scalar_field(&mut self, field: &ScalarField, name: &str) -> Result<usize> {
        let field_name = if name.is_empty() {
            format!("PicoGK.ScalarField.{}", self.field_count())
        } else {
            name.to_string()
        };

        let c_name = CString::new(field_name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;

        let index = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VdbFile_nAddScalarField(self.handle, c_name.as_ptr(), field.handle())
        });

        if index < 0 {
            return Err(Error::InvalidParameter(
                "Failed to add scalar field".to_string(),
            ));
        }

        Ok(index as usize)
    }

    /// Get VectorField from a field at the specified index
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the field (0-based)
    pub fn get_vector_field(&self, index: usize) -> Result<VectorField> {
        if index >= self.field_count() {
            return Err(Error::InvalidParameter(format!(
                "Index {} out of range",
                index
            )));
        }

        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VdbFile_hGetVectorField(self.handle, index as i32)
        });
        if handle.is_null() {
            return Err(Error::InvalidParameter(format!(
                "No vector field at index {}",
                index
            )));
        }

        Ok(VectorField::from_handle(handle))
    }

    /// Add VectorField to the VDB file
    ///
    /// # Arguments
    ///
    /// * `field` - VectorField to add
    /// * `name` - Name for the field (if empty, auto-generates a name)
    ///
    /// # Returns
    ///
    /// Index of the added field
    pub fn add_vector_field(&mut self, field: &VectorField, name: &str) -> Result<usize> {
        let field_name = if name.is_empty() {
            format!("PicoGK.VectorField.{}", self.field_count())
        } else {
            name.to_string()
        };

        let c_name = CString::new(field_name)
            .map_err(|_| Error::InvalidParameter("Name contains null byte".to_string()))?;

        let index = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::VdbFile_nAddVectorField(self.handle, c_name.as_ptr(), field.handle())
        });

        if index < 0 {
            return Err(Error::InvalidParameter(
                "Failed to add vector field".to_string(),
            ));
        }

        Ok(index as usize)
    }

    /// Check if the VDB file is valid
    pub fn is_valid(&self) -> bool {
        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::VdbFile_bIsValid(self.handle) })
    }
}

impl Drop for VdbFile {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::VdbFile_Destroy(self.handle);
            });
        }
    }
}

// VdbFile is Send + Sync because all native calls are serialized via the crate's re-entrant FFI
// lock (see `ffi_lock.rs` / `SAFETY.md`).
unsafe impl Send for VdbFile {}
unsafe impl Sync for VdbFile {}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_vdb_file_creation() {
        let _lib = crate::Library::init(0.5).unwrap();
        let vdb = VdbFile::new();
        assert!(vdb.is_ok());
    }
}
