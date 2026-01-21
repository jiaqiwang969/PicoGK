//! # PicoGK Rust Bindings
//!
//! Rust bindings for PicoGK - A compact geometry kernel for Computational Engineering.
//!
//! ## Features
//!
//! - **Zero-cost FFI**: Direct C++ interop with no overhead
//! - **Memory safe**: Rust's ownership system prevents memory leaks
//! - **Thread safe**: Compile-time concurrency guarantees
//! - **Type safe**: Strong type system catches errors at compile time
//!
//! ## Example
//!
//! ```rust,no_run
//! use picogk::{Library, Voxels};
//! use nalgebra::Vector3;
//!
//! // Initialize library
//! let _lib = Library::init(0.5)?;
//!
//! // Create a sphere
//! let sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
//!
//! // Convert to mesh and save
//! let mesh = sphere.as_mesh()?;
//! mesh.save_stl("sphere.stl")?;
//! # Ok::<(), picogk::Error>(())
//! ```

pub mod animation;
pub mod cli;
pub mod csv;
pub mod easing;
pub mod error;
pub mod ffi;
mod ffi_lock;
pub mod field_utils;
pub mod image;
pub mod image_io;
pub mod implicit;
pub mod lattice;
pub mod log;
pub mod mesh;
pub mod metadata;
pub mod polyline;
pub mod scalar_field;
pub mod slice;
pub mod types;
pub mod utils;
pub mod vdb_file;
pub mod vector_ext;
pub mod vector_field;
pub mod viewer;
pub mod voxels;

/// Convenience imports for common traits/extensions.
pub mod prelude {
    pub use crate::{DataTable, Image, Vector3Ext};
}

// Re-exports
pub use animation::{Animation, AnimationAction, AnimationQueue, AnimationType};
pub use cli::{CliFormat, CliIo, CliResult};
pub use csv::{CsvTable, DataTable};
pub use easing::{Easing, EasingKind};
pub use error::{Error, Result};
pub use field_utils::{
    ActiveVoxelCounterScalar, AddVectorFieldToViewer, SdfVisualizer, SurfaceNormalFieldExtractor,
    VectorFieldMerge,
};
pub use image::{
    Image, ImageBW, ImageColor, ImageData, ImageGrayScale, ImageRgb24, ImageRgba32, ImageType,
};
pub use image_io::TgaIo;
pub use implicit::{
    BoxImplicit, CapsuleImplicit, CylinderImplicit, GyroidImplicit, Implicit, SphereImplicit,
    TorusImplicit, TwistedTorusImplicit,
};
pub use lattice::Lattice;
pub use log::LogFile;
pub use mesh::{Mesh, StlUnit};
pub use metadata::{FieldMetadata, MetadataType, MetadataValue};
pub use polyline::PolyLine;
pub use scalar_field::ScalarField;
pub use slice::{PolyContour, PolySlice, PolySliceStack, Winding};
pub use types::{
    BBox2, BBox3, ColorBgr24, ColorBgra32, ColorFloat, ColorHLS, ColorHSV, ColorRgb24, ColorRgba32,
    Matrix4x4, Triangle, Vector2f, Vector3f, VoxelDimensions,
};
pub use utils::{TempFolder, Utils};
pub use vdb_file::{FieldType, VdbFile};
pub use vector_ext::Vector3Ext;
pub use vector_field::VectorField;
pub use viewer::{
    AnimGroupMatrixRotate, AnimViewRotate, Key, KeyAction, KeyHandler, KeyHandlerSet,
    RotateDirection, RotateToNextRoundAngleAction, Viewer,
};
pub use voxels::{SliceMode, VoxelSlice, Voxels};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Once, OnceLock};
use std::thread;
use std::time::Duration;

static INIT: Once = Once::new();
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static VOXEL_SIZE: Mutex<f32> = Mutex::new(0.0);
static RUNNING: AtomicBool = AtomicBool::new(false);
static APP_EXIT: AtomicBool = AtomicBool::new(false);
static CONTINUE_TASK: AtomicBool = AtomicBool::new(true);
static LOG_INSTANCE: OnceLock<Mutex<Option<LogFile>>> = OnceLock::new();
static VIEWER_INSTANCE: OnceLock<Mutex<Option<Viewer>>> = OnceLock::new();
static LOG_FOLDER: OnceLock<Mutex<String>> = OnceLock::new();
static SRC_FOLDER: OnceLock<Mutex<String>> = OnceLock::new();

/// PicoGK Library handle
///
/// This must be initialized before using any other PicoGK functions.
///
/// Note: We intentionally do **not** call `Library_Destroy` on drop. The PicoGK native
/// library is treated as a process-global singleton; tearing it down would invalidate
/// all existing native handles.
///
/// Note: The library can only be initialized once per process. Subsequent
/// calls to `init()` will return a handle to the already-initialized library
/// if the voxel size matches, or an error if it differs.
pub struct Library {
    _private: (),
}

impl Library {
    /// Initialize the PicoGK library
    ///
    /// # Arguments
    ///
    /// * `voxel_size_mm` - The global voxel size in millimeters (e.g., 0.5)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Library;
    ///
    /// let lib = Library::init(0.5)?;
    /// // Use PicoGK functions...
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn init(voxel_size_mm: f32) -> Result<Self> {
        if voxel_size_mm <= 0.0 {
            return Err(Error::InvalidParameter(
                "voxel_size_mm must be positive".to_string(),
            ));
        }

        INIT.call_once(|| {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::Library_Init(voxel_size_mm);
            });
            *VOXEL_SIZE.lock().unwrap_or_else(|e| e.into_inner()) = voxel_size_mm;
            INITIALIZED.store(true, Ordering::SeqCst);
        });

        if !INITIALIZED.load(Ordering::SeqCst) {
            return Err(Error::InitializationFailed);
        }

        let stored_size = *VOXEL_SIZE.lock().unwrap_or_else(|e| e.into_inner());
        if (stored_size - voxel_size_mm).abs() > 1e-6 {
            return Err(Error::InvalidParameter(format!(
                "Library already initialized with voxel size {}, cannot reinitialize with {}",
                stored_size, voxel_size_mm
            )));
        }

        Ok(Library { _private: () })
    }

    /// Get the current voxel size
    pub fn voxel_size_mm() -> f32 {
        *VOXEL_SIZE.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Get library name
    pub fn name() -> String {
        unsafe {
            let mut buffer = vec![0u8; 256];
            crate::ffi_lock::with_ffi_lock(|| {
                ffi::Library_GetName(buffer.as_mut_ptr() as *mut i8);
            });
            String::from_utf8_lossy(&buffer)
                .trim_end_matches('\0')
                .to_string()
        }
    }

    /// Get library version
    pub fn version() -> String {
        unsafe {
            let mut buffer = vec![0u8; 256];
            crate::ffi_lock::with_ffi_lock(|| {
                ffi::Library_GetVersion(buffer.as_mut_ptr() as *mut i8);
            });
            String::from_utf8_lossy(&buffer)
                .trim_end_matches('\0')
                .to_string()
        }
    }

    /// Get library build info
    pub fn build_info() -> String {
        unsafe {
            let mut buffer = vec![0u8; 256];
            crate::ffi_lock::with_ffi_lock(|| {
                ffi::Library_GetBuildInfo(buffer.as_mut_ptr() as *mut i8);
            });
            String::from_utf8_lossy(&buffer)
                .trim_end_matches('\0')
                .to_string()
        }
    }

    /// Convert voxel coordinates to millimeters
    pub fn voxels_to_mm(voxel_coordinate: nalgebra::Vector3<f32>) -> nalgebra::Vector3<f32> {
        let voxel = crate::types::Vector3f::from(voxel_coordinate);
        let mut mm = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Library_VoxelsToMm(
                &voxel as *const crate::types::Vector3f,
                &mut mm as *mut crate::types::Vector3f,
            );
        });
        nalgebra::Vector3::from(mm)
    }

    /// Convert millimeter coordinates to voxel coordinates
    pub fn mm_to_voxels(mm_coordinate: nalgebra::Vector3<f32>) -> nalgebra::Vector3<f32> {
        let mm = crate::types::Vector3f::from(mm_coordinate);
        let mut voxel = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Library_MmToVoxels(
                &mm as *const crate::types::Vector3f,
                &mut voxel as *mut crate::types::Vector3f,
            );
        });
        nalgebra::Vector3::from(voxel)
    }

    /// Convert millimeter coordinates to voxel indices (rounded)
    pub fn mm_to_voxel_indices(mm_coordinate: nalgebra::Vector3<f32>) -> nalgebra::Vector3<i32> {
        let voxel = Self::mm_to_voxels(mm_coordinate);
        nalgebra::Vector3::new(
            (voxel.x + 0.5) as i32,
            (voxel.y + 0.5) as i32,
            (voxel.z + 0.5) as i32,
        )
    }

    /// Run PicoGK with a viewer and log file
    pub fn go<F>(
        voxel_size_mm: f32,
        task: F,
        log_folder: Option<&str>,
        log_file_name: Option<&str>,
        src_folder: Option<&str>,
        lights_file: Option<&str>,
        end_app_with_task: bool,
    ) -> Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        if RUNNING.swap(true, Ordering::SeqCst) {
            return Err(Error::OperationFailed(
                "PicoGK only supports running one library config at one time".to_string(),
            ));
        }

        struct RunGuard;
        impl Drop for RunGuard {
            fn drop(&mut self) {
                RUNNING.store(false, Ordering::SeqCst);
            }
        }
        let _guard = RunGuard;

        APP_EXIT.store(false, Ordering::SeqCst);
        CONTINUE_TASK.store(true, Ordering::SeqCst);

        let log_folder_path = match log_folder {
            Some(folder) if !folder.is_empty() => std::path::PathBuf::from(folder),
            _ => Utils::documents_folder()?,
        };
        let log_file_name = match log_file_name {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => "PicoGK.log".to_string(),
        };
        let log_path = log_folder_path.join(&log_file_name);
        let log_path_str = log_path.to_string_lossy().to_string();

        let log = LogFile::new(Some(&log_path_str), true)?;
        let log_store = LOG_INSTANCE.get_or_init(|| Mutex::new(None));
        *log_store.lock().unwrap_or_else(|e| e.into_inner()) = Some(log.clone());
        let log_folder_store = LOG_FOLDER.get_or_init(|| Mutex::new(String::new()));
        *log_folder_store.lock().unwrap_or_else(|e| e.into_inner()) =
            log_folder_path.to_string_lossy().to_string();
        let src_folder_store = SRC_FOLDER.get_or_init(|| Mutex::new(String::new()));
        *src_folder_store.lock().unwrap_or_else(|e| e.into_inner()) =
            src_folder.unwrap_or("").to_string();

        let _ = log.log("Welcome to PicoGK");

        Library::init(voxel_size_mm)?;

        let _ = log.log(format!("PicoGK:    {}", Library::name()));
        let _ = log.log(format!("           {}", Library::version()));
        let _ = log.log(format!("           {}\n", Library::build_info()));
        let _ = log.log(format!("VoxelSize: {} (mm)", Library::voxel_size_mm()));
        let _ = log.log("Happy Computational Engineering!\n\n");

        let viewer = Viewer::new(
            "PicoGK",
            nalgebra::Vector2::new(2048.0, 1024.0),
            log.clone(),
        )?;
        let viewer_store = VIEWER_INSTANCE.get_or_init(|| Mutex::new(None));
        *viewer_store.lock().unwrap_or_else(|e| e.into_inner()) = Some(viewer.clone());

        if let Some(lights_file) = lights_file {
            if !lights_file.is_empty() {
                if let Err(err) = viewer.load_light_setup(lights_file) {
                    let _ = log.log(format!("Failed to load light setup: {}", err));
                }
            }
        } else {
            let (path, searched) = Library::find_light_setup_file(src_folder);
            if std::path::Path::new(&path).exists() {
                if let Err(err) = viewer.load_light_setup(&path) {
                    let _ = log.log(format!("Failed to load light setup: {}", err));
                }
            } else {
                let _ = log.log("Could not find a lights file - your viewer will look quite dark.");
                let _ = log.log("Searched in:");
                let _ = log.log(searched);
            }
        }

        viewer.set_background_color(ColorFloat::gray(1.0, 1.0));

        let handle = thread::spawn(task);

        while viewer.poll() {
            thread::sleep(Duration::from_millis(5));
            if end_app_with_task && handle.is_finished() && viewer.is_idle() {
                break;
            }
        }

        APP_EXIT.store(true, Ordering::SeqCst);
        let _ = log.log("Viewer Window Closed");

        Ok(())
    }

    /// Query if the task started by `go` should continue
    pub fn continue_task(app_exit_only: bool) -> bool {
        !APP_EXIT.load(Ordering::SeqCst) && (app_exit_only || CONTINUE_TASK.load(Ordering::SeqCst))
    }

    /// Request the task started by `go` to end
    pub fn end_task() {
        CONTINUE_TASK.store(false, Ordering::SeqCst);
    }

    /// Cancel a pending request to end the task
    pub fn cancel_end_task_request() {
        CONTINUE_TASK.store(true, Ordering::SeqCst);
    }

    /// Thread-safe logging (requires `go` to have been called)
    pub fn log(message: impl AsRef<str>) -> Result<()> {
        let log_store = LOG_INSTANCE
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(log) = log_store.as_ref() {
            log.log(message.as_ref())
        } else {
            Err(Error::OperationFailed(
                "Trying to access Log before Library::go() was called".to_string(),
            ))
        }
    }

    /// Thread-safe access to the viewer (requires `go` to have been called)
    pub fn viewer() -> Result<Viewer> {
        let store = VIEWER_INSTANCE
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(viewer) = store.as_ref() {
            Ok(viewer.clone())
        } else {
            Err(Error::OperationFailed(
                "Trying to access Viewer before Library::go() was called".to_string(),
            ))
        }
    }

    /// Returns the log folder used by `go`
    pub fn log_folder() -> String {
        LOG_FOLDER
            .get_or_init(|| Mutex::new(String::new()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Returns the source folder set by `go`
    pub fn src_folder() -> String {
        SRC_FOLDER
            .get_or_init(|| Mutex::new(String::new()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Find the default light setup file and return (path, searched)
    pub fn find_light_setup_file(input_folder: Option<&str>) -> (String, String) {
        let mut searched = String::new();

        if let Ok(mut path) = Utils::picogk_source_code_folder() {
            path = path.join("ViewerEnvironment/PicoGKDefaultEnv.zip");
            if path.exists() {
                return (path.to_string_lossy().to_string(), searched);
            }
            searched.push_str(&format!("{}\n", path.to_string_lossy()));
        }

        let input_folder = input_folder.unwrap_or("");
        let mut path = if input_folder.is_empty() {
            Utils::documents_folder()
                .map(|p| p.join("PicoGKDefaultEnv.zip"))
                .unwrap_or_else(|_| std::path::PathBuf::from("PicoGKDefaultEnv.zip"))
        } else {
            std::path::PathBuf::from(input_folder).join("PicoGKDefaultEnv.zip")
        };
        searched.push_str(&format!("{}\n", path.to_string_lossy()));
        if path.exists() {
            return (path.to_string_lossy().to_string(), searched);
        }

        path = Utils::executable_folder()
            .map(|p| p.join("ViewerEnvironment.zip"))
            .unwrap_or_else(|_| std::path::PathBuf::from("ViewerEnvironment.zip"));
        searched.push_str(&format!("{}\n", path.to_string_lossy()));

        (path.to_string_lossy().to_string(), searched)
    }
}

// Note: We don't implement Drop for Library because the C++ library
// should remain initialized for the lifetime of the process.
// Cleaning up the library would invalidate all existing handles.

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_library_init() {
        let lib = Library::init(0.5);
        assert!(lib.is_ok());
    }

    #[test]
    #[serial]
    fn test_invalid_voxel_size() {
        let lib = Library::init(-1.0);
        assert!(lib.is_err());
    }
}
