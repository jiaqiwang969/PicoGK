//! Voxels VDB file I/O

use crate::{Error, FieldType, Result, VdbFile, Voxels};
use std::path::Path;

impl Voxels {
    /// Load Voxels from a VDB file
    ///
    /// This function searches for the first compatible Voxels field in the .vdb file.
    /// For more sophisticated .vdb file handling, use `VdbFile` directly.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .vdb file
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Library, Voxels};
    ///
    /// let _lib = Library::init(0.5)?;
    /// let voxels = Voxels::load_vdb("input.vdb")?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn load_vdb<P: AsRef<Path>>(path: P) -> Result<Self> {
        let vdb = VdbFile::load(path.as_ref())?;

        if vdb.field_count() == 0 {
            return Err(Error::FileLoad(format!(
                "No fields contained in OpenVDB file {}",
                path.as_ref().display()
            )));
        }

        // Find the first voxel field
        for i in 0..vdb.field_count() {
            if vdb.field_type(i) == FieldType::Voxels {
                return vdb.get_voxels(i);
            }
        }

        // Build error message with field info
        let mut field_info = String::from("Fields found:\n");
        for i in 0..vdb.field_count() {
            field_info.push_str(&format!(
                "- {} ({:?})\n",
                vdb.field_name(i),
                vdb.field_type(i)
            ));
        }

        Err(Error::FileLoad(format!(
            "No voxel field (GRID_LEVEL_SET) found in VDB file {}\n{}",
            path.as_ref().display(),
            field_info
        )))
    }

    /// Save Voxels to a VDB file
    ///
    /// Creates a new .vdb file and saves the voxel field to it.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where to save the .vdb file
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Library, Voxels};
    /// use nalgebra::Vector3;
    ///
    /// let _lib = Library::init(0.5)?;
    /// let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;
    /// sphere.save_vdb("output.vdb")?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn save_vdb<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut vdb = VdbFile::new()?;
        vdb.add_voxels(self, "")?;
        vdb.save(path)
    }
}
