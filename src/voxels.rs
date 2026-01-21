//! Voxel field representation

mod io;

use crate::{
    ffi, CliFormat, CliIo, Error, FieldMetadata, ImageGrayScale, Implicit, Lattice, Library, Mesh,
    PolySlice, PolySliceStack, Result, ScalarField, VoxelDimensions,
};
use nalgebra::Vector3;
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

/// Slice rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceMode {
    SignedDistance,
    BlackWhite,
    Antialiased,
}

/// Slice data returned from voxel queries
pub struct VoxelSlice {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
    pub background: f32,
}

struct ImplicitCallbackData {
    ctx: *mut c_void,
    call: fn(*mut c_void, Vector3<f32>) -> f32,
}

static IMPLICIT_CALLBACK_DATA: AtomicPtr<ImplicitCallbackData> =
    AtomicPtr::new(std::ptr::null_mut());

unsafe extern "C" fn implicit_trampoline(point: *const crate::types::Vector3f) -> f32 {
    if point.is_null() {
        return 0.0;
    }
    let data_ptr = IMPLICIT_CALLBACK_DATA.load(Ordering::SeqCst);
    if data_ptr.is_null() {
        return 0.0;
    }
    let data = &mut *data_ptr;
    let pos = Vector3::from(*point);
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (data.call)(data.ctx, pos)))
        .unwrap_or(0.0)
}

/// Voxel field representation
///
/// This is the core data structure in PicoGK, representing geometry
/// as a signed distance field stored in a sparse voxel grid (OpenVDB).
///
/// # Example
///
/// ```rust,no_run
/// use picogk::{Library, Voxels};
/// use nalgebra::Vector3;
///
/// let _lib = Library::init(0.5)?;
/// let sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
/// # Ok::<(), picogk::Error>(())
/// ```
pub struct Voxels {
    handle: *mut ffi::CVoxels,
}

impl Voxels {
    /// Create an empty voxel field
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Voxels;
    ///
    /// let vox = Voxels::new()?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::Voxels_hCreate() });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Create a sphere
    ///
    /// # Arguments
    ///
    /// * `center` - Center position of the sphere
    /// * `radius` - Radius in millimeters
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Voxels;
    /// use nalgebra::Vector3;
    ///
    /// let sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn sphere(center: Vector3<f32>, radius: f32) -> Result<Self> {
        if radius <= 0.0 {
            return Err(Error::InvalidParameter(
                "radius must be positive".to_string(),
            ));
        }

        let mut lattice = Lattice::new()?;
        lattice.add_sphere(center, radius);
        Self::from_lattice(&lattice)
    }

    /// Create from lattice
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Voxels, Lattice};
    /// use nalgebra::Vector3;
    ///
    /// let mut lattice = Lattice::new()?;
    /// lattice.add_sphere(Vector3::zeros(), 10.0);
    /// let vox = Voxels::from_lattice(&lattice)?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn from_lattice(lattice: &Lattice) -> Result<Self> {
        let vox = Self::new()?;
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_RenderLattice(vox.handle, lattice.handle());
        });
        Ok(vox)
    }

    /// Render a lattice into the voxel field, combining it with existing content.
    pub fn render_lattice(&mut self, lattice: &Lattice) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_RenderLattice(self.handle, lattice.handle());
        });
    }

    /// Create from mesh
    pub fn from_mesh(mesh: &Mesh) -> Result<Self> {
        let vox = Self::new()?;
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_RenderMesh(vox.handle, mesh.handle());
        });
        Ok(vox)
    }

    /// Render a mesh into the voxel field, combining it with existing content.
    ///
    /// The mesh needs to be a closed surface for correct results.
    pub fn render_mesh(&mut self, mesh: &Mesh) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_RenderMesh(self.handle, mesh.handle());
        });
    }

    /// Create from scalar field
    pub fn from_scalar_field(field: &ScalarField) -> Result<Self> {
        let mut vox = Self::new()?;
        let bounds = field.bounding_box();
        vox.render_implicit(field, bounds)?;
        Ok(vox)
    }

    /// Create from implicit function using its bounds
    pub fn from_implicit(implicit: &dyn Implicit) -> Result<Self> {
        let bounds = implicit
            .bounds()
            .ok_or_else(|| Error::InvalidParameter("Implicit bounds are required".to_string()))?;
        Self::from_implicit_with_bounds(implicit, bounds)
    }

    /// Create from implicit function and explicit bounds
    pub fn from_implicit_with_bounds(
        implicit: &dyn Implicit,
        bounds: crate::BBox3,
    ) -> Result<Self> {
        let mut vox = Self::new()?;
        vox.render_implicit(implicit, bounds)?;
        Ok(vox)
    }

    /// Boolean union (add)
    ///
    /// Adds the operand to this voxel field.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Voxels;
    /// use nalgebra::Vector3;
    ///
    /// let mut vox1 = Voxels::sphere(Vector3::new(-5.0, 0.0, 0.0), 10.0)?;
    /// let vox2 = Voxels::sphere(Vector3::new(5.0, 0.0, 0.0), 10.0)?;
    /// vox1.bool_add(&vox2);
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn bool_add(&mut self, operand: &Voxels) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_BoolAdd(self.handle, operand.handle);
        });
    }

    /// Boolean union with multiple voxel fields
    pub fn bool_add_all<'a, I>(&mut self, voxels: I)
    where
        I: IntoIterator<Item = &'a Voxels>,
    {
        for vox in voxels {
            self.bool_add(vox);
        }
    }

    /// Boolean union with smoothing
    pub fn bool_add_smooth(&mut self, operand: &Voxels, smooth_distance_mm: f32) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_BoolAddSmooth(self.handle, operand.handle, smooth_distance_mm);
        });
    }

    /// Boolean difference (subtract)
    ///
    /// Subtracts the operand from this voxel field.
    pub fn bool_subtract(&mut self, operand: &Voxels) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_BoolSubtract(self.handle, operand.handle);
        });
    }

    /// Boolean subtraction with multiple voxel fields
    pub fn bool_subtract_all<'a, I>(&mut self, voxels: I)
    where
        I: IntoIterator<Item = &'a Voxels>,
    {
        for vox in voxels {
            self.bool_subtract(vox);
        }
    }

    /// Boolean intersection
    ///
    /// Intersects this voxel field with the operand.
    pub fn bool_intersect(&mut self, operand: &Voxels) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_BoolIntersect(self.handle, operand.handle);
        });
    }

    /// Combine two voxel fields into a new one
    pub fn combine(a: &Voxels, b: &Voxels) -> Result<Voxels> {
        a.vox_bool_add(b)
    }

    /// Combine a collection of voxel fields into a new one
    pub fn combine_all<'a, I>(voxels: I) -> Result<Voxels>
    where
        I: IntoIterator<Item = &'a Voxels>,
    {
        let mut result = Voxels::new()?;
        result.bool_add_all(voxels);
        Ok(result)
    }

    /// Offset the surface
    ///
    /// Positive values expand the surface outward,
    /// negative values shrink it inward.
    ///
    /// # Arguments
    ///
    /// * `dist_mm` - Distance to offset in millimeters
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Voxels;
    /// use nalgebra::Vector3;
    ///
    /// let mut sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
    /// sphere.offset(5.0);  // Expand by 5mm
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn offset(&mut self, dist_mm: f32) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_Offset(self.handle, dist_mm);
        });
    }

    /// Double offset operation
    ///
    /// Applies two offset operations in sequence.
    pub fn double_offset(&mut self, dist1_mm: f32, dist2_mm: f32) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_DoubleOffset(self.handle, dist1_mm, dist2_mm);
        });
    }

    /// Smooth the surface
    ///
    /// Applies triple offset operation to smooth the geometry.
    /// This is equivalent to: offset(d), offset(-2d), offset(d)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Voxels;
    /// use nalgebra::Vector3;
    ///
    /// let mut sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
    /// sphere.smoothen(2.0);
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn smoothen(&mut self, dist_mm: f32) {
        self.triple_offset(dist_mm);
    }

    /// Triple offset operation
    ///
    /// Offsets the voxel field three times by the specified distance.
    /// First it offsets inwards by the specified distance,
    /// then it offsets twice the distance outwards,
    /// and finally offsets inwards again by the specified distance.
    /// This creates a smoothing effect.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Voxels;
    /// use nalgebra::Vector3;
    ///
    /// let mut sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
    /// sphere.triple_offset(2.0);
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn triple_offset(&mut self, dist_mm: f32) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_TripleOffset(self.handle, dist_mm);
        });
    }

    /// Apply Gaussian blur
    pub fn gaussian(&mut self, size_mm: f32) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_Gaussian(self.handle, size_mm);
        });
    }

    /// Apply median filter
    pub fn median(&mut self, size_mm: f32) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_Median(self.handle, size_mm);
        });
    }

    /// Apply mean filter
    pub fn mean(&mut self, size_mm: f32) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_Mean(self.handle, size_mm);
        });
    }

    /// Over offset operation
    ///
    /// Similar to double_offset, but allows you to specify the final
    /// surface distance to the original surface as the second parameter.
    ///
    /// # Arguments
    ///
    /// * `first_offset_mm` - First offset distance
    /// * `final_surface_dist_mm` - Final distance from original surface (default: 0)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Voxels;
    /// use nalgebra::Vector3;
    ///
    /// let mut sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
    /// sphere.over_offset(2.0, 0.0);
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn over_offset(&mut self, first_offset_mm: f32, final_surface_dist_mm: f32) {
        self.double_offset(first_offset_mm, -(first_offset_mm - final_surface_dist_mm));
    }

    /// Fillet operation
    ///
    /// Creates a fillet-like effect by applying over_offset with
    /// final surface distance of 0. This rounds sharp edges.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Voxels;
    /// use nalgebra::Vector3;
    ///
    /// let mut sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
    /// sphere.fillet(2.0);
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn fillet(&mut self, rounding_mm: f32) {
        self.over_offset(rounding_mm, 0.0);
    }

    /// Create a shell
    ///
    /// Returns a hollow version of this voxel field by offsetting and
    /// subtracting the original.
    ///
    /// # Arguments
    ///
    /// * `offset_mm` - Offset distance in millimeters
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Voxels;
    /// use nalgebra::Vector3;
    ///
    /// let sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
    /// let shell = sphere.shell(3.0)?;  // 3mm shell
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn shell(&self, offset_mm: f32) -> Result<Self> {
        let mut outer = self.duplicate()?;

        if offset_mm < 0.0 {
            let mut inner = self.duplicate()?;
            inner.offset(offset_mm);
            outer.bool_subtract(&inner);
            return Ok(outer);
        }

        outer.offset(offset_mm);
        outer.bool_subtract(self);
        Ok(outer)
    }

    /// Create a shell using separate inner and outer offsets
    pub fn shell_with_offsets(
        &self,
        neg_offset_mm: f32,
        pos_offset_mm: f32,
        smooth_inner_mm: f32,
    ) -> Result<Self> {
        let (mut neg, mut pos) = (neg_offset_mm, pos_offset_mm);
        if neg > pos {
            std::mem::swap(&mut neg, &mut pos);
        }

        let mut inner = self.duplicate()?;
        inner.offset(neg);
        if smooth_inner_mm > 0.0 {
            inner.triple_offset(smooth_inner_mm);
        }

        let mut outer = self.duplicate()?;
        outer.offset(pos);
        outer.bool_subtract(&inner);
        Ok(outer)
    }

    /// Convert to mesh
    ///
    /// Generates a triangle mesh from the voxel field using Marching Cubes.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Voxels;
    /// use nalgebra::Vector3;
    ///
    /// let sphere = Voxels::sphere(Vector3::zeros(), 20.0)?;
    /// let mesh = sphere.as_mesh()?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn as_mesh(&self) -> Result<Mesh> {
        Mesh::from_voxels(self)
    }

    /// Duplicate the voxel field
    ///
    /// Creates a deep copy of this voxel field.
    pub fn duplicate(&self) -> Result<Self> {
        let handle =
            crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::Voxels_hCreateCopy(self.handle) });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Fallible clone of the voxel field.
    ///
    /// This is an alias for `duplicate()` and exists because `Clone` cannot be
    /// implemented safely for a fallible native duplication operation.
    pub fn try_clone(&self) -> Result<Self> {
        self.duplicate()
    }

    /// Check if the voxel field is valid
    pub fn is_valid(&self) -> bool {
        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::Voxels_bIsValid(self.handle) })
    }

    /// Check if a point is inside the voxel field
    pub fn is_inside(&self, point: Vector3<f32>) -> bool {
        let point_ffi = crate::types::Vector3f::from(point);
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_bIsInside(self.handle, &point_ffi as *const _)
        })
    }

    /// Check if two voxel fields are equal
    pub fn is_equal(&self, other: &Voxels) -> bool {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_bIsEqual(self.handle, other.handle)
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
            ffi::Voxels_GetVoxelDimensions(
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

    /// Get slice count in Z direction
    pub fn slice_count(&self) -> i32 {
        self.voxel_dimensions().size.z
    }

    /// Get the origin of a Z slice in millimeters
    pub fn z_slice_origin(&self, z_slice: i32) -> Vector3<f32> {
        let dims = self.voxel_dimensions();
        let origin = dims.origin;
        Library::voxels_to_mm(Vector3::new(
            origin.x as f32,
            origin.y as f32,
            (origin.z + z_slice) as f32,
        ))
    }

    /// Get a signed-distance slice at the specified Z index
    pub fn get_voxel_slice(&self, z_slice: i32, mode: SliceMode) -> Result<VoxelSlice> {
        let dims = self.voxel_dimensions();
        let width = dims.size.x.max(0) as usize;
        let height = dims.size.y.max(0) as usize;
        let len = width.saturating_mul(height);
        if len == 0 {
            return Err(Error::OperationFailed(
                "Voxel slice has zero size".to_string(),
            ));
        }

        let mut values = vec![0.0f32; len];
        let mut background = 0.0f32;
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_GetSlice(
                self.handle,
                z_slice,
                values.as_mut_ptr(),
                &mut background as *mut f32,
            );
        });
        apply_slice_mode(&mut values, mode, background);
        Ok(VoxelSlice {
            width,
            height,
            values,
            background,
        })
    }

    /// Get a signed-distance slice at the interpolated Z position
    pub fn get_interpolated_voxel_slice(
        &self,
        z_slice: f32,
        mode: SliceMode,
    ) -> Result<VoxelSlice> {
        let dims = self.voxel_dimensions();
        let width = dims.size.x.max(0) as usize;
        let height = dims.size.y.max(0) as usize;
        let len = width.saturating_mul(height);
        if len == 0 {
            return Err(Error::OperationFailed(
                "Voxel slice has zero size".to_string(),
            ));
        }

        let mut values = vec![0.0f32; len];
        let mut background = 0.0f32;
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_GetInterpolatedSlice(
                self.handle,
                z_slice,
                values.as_mut_ptr(),
                &mut background as *mut f32,
            );
        });
        apply_slice_mode(&mut values, mode, background);
        Ok(VoxelSlice {
            width,
            height,
            values,
            background,
        })
    }

    /// Vectorize the voxel field into a stack of polygon slices
    pub fn vectorize(
        &self,
        layer_height_mm: f32,
        use_abs_xy_origin: bool,
    ) -> Result<PolySliceStack> {
        let voxel_size = Library::voxel_size_mm();
        let layer_height = if layer_height_mm == 0.0 {
            voxel_size
        } else {
            layer_height_mm
        };
        if layer_height <= 0.0 {
            return Err(Error::InvalidParameter(
                "layer_height_mm must be positive".to_string(),
            ));
        }

        let z_step = layer_height / voxel_size;

        let dims = self.voxel_dimensions();
        let origin = dims.origin;
        let size = dims.size;
        let width = size.x.max(0) as usize;
        let height = size.y.max(0) as usize;
        let depth = size.z.max(0) as usize;

        if width == 0 || height == 0 || depth == 0 {
            return Err(Error::InvalidParameter(
                "Voxel field is empty - cannot vectorize".to_string(),
            ));
        }

        let mut img = ImageGrayScale::new(width, height);
        let mut slices: Vec<PolySlice> = Vec::new();

        let mut origin_offset = nalgebra::Vector2::zeros();
        if use_abs_xy_origin {
            origin_offset =
                nalgebra::Vector2::new(origin.x as f32 * voxel_size, origin.y as f32 * voxel_size);
        }

        let last_layer = depth as f32 - 1.0;
        let mut z = 0.0f32;
        let mut layer_z = layer_height;

        while z <= last_layer {
            let slice = self.get_interpolated_voxel_slice(z, SliceMode::SignedDistance)?;
            if slice.values.len() == img.values.len() {
                img.values.copy_from_slice(&slice.values);
            } else {
                return Err(Error::OperationFailed(
                    "Interpolated slice has unexpected size".to_string(),
                ));
            }

            let mut poly_slice = PolySlice::from_sdf(&img, layer_z, origin_offset, voxel_size);

            if (layer_z - layer_height).abs() < f32::EPSILON && poly_slice.is_empty() {
                z += z_step;
                layer_z += layer_height;
                continue;
            }

            poly_slice.close();
            slices.push(poly_slice);

            layer_z += layer_height;
            z += z_step;
        }

        if slices.is_empty() {
            return Err(Error::OperationFailed(
                "Voxel field is empty - cannot write .CLI file".to_string(),
            ));
        }

        while let Some(last) = slices.last() {
            if last.is_empty() {
                slices.pop();
                if slices.is_empty() {
                    return Err(Error::OperationFailed(
                        "Voxel field is empty - cannot write .CLI file".to_string(),
                    ));
                }
            } else {
                break;
            }
        }

        Ok(PolySliceStack::from_slices(slices))
    }

    /// Save the voxel field to a .cli file
    pub fn save_cli_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        layer_height_mm: f32,
        format: CliFormat,
        use_abs_xy_origin: bool,
    ) -> Result<()> {
        let stack = self.vectorize(layer_height_mm, use_abs_xy_origin)?;
        CliIo::write_slices_to_cli_file(&stack, path, format, None, None)
    }

    /// C#-style alias for `save_cli_file`.
    pub fn save_to_cli_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        layer_height_mm: f32,
        format: CliFormat,
        use_abs_xy_origin: bool,
    ) -> Result<()> {
        self.save_cli_file(path, layer_height_mm, format, use_abs_xy_origin)
    }

    /// C#-style alias for `save_cli_file`.
    pub fn save_cli<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        layer_height_mm: f32,
        format: CliFormat,
        use_abs_xy_origin: bool,
    ) -> Result<()> {
        self.save_cli_file(path, layer_height_mm, format, use_abs_xy_origin)
    }

    /// Calculate volume and bounding box
    ///
    /// Returns a tuple of (volume in mm³, bounding box)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Library, Voxels};
    /// use nalgebra::Vector3;
    ///
    /// let _lib = Library::init(0.5)?;
    /// let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;
    /// let (volume, bbox) = sphere.calculate_properties();
    /// println!("Volume: {} mm³", volume);
    /// println!("BBox: {:?}", bbox);
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn calculate_properties(&self) -> (f32, crate::BBox3) {
        let mut volume = 0.0f32;
        let mut bbox = crate::BBox3::empty();
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_CalculateProperties(
                self.handle,
                &mut volume as *mut f32,
                &mut bbox as *mut crate::BBox3,
            );
        });
        (volume, bbox)
    }

    /// Volume of the voxel field (in mm³).
    pub fn volume_mm3(&self) -> f32 {
        self.calculate_properties().0
    }

    /// Bounding box of active voxels (in mm).
    pub fn bounding_box(&self) -> crate::BBox3 {
        self.calculate_properties().1
    }

    /// Get surface normal at a point on the surface
    ///
    /// # Arguments
    ///
    /// * `surface_point` - Point on the surface
    ///
    /// # Returns
    ///
    /// The surface normal vector at the given point
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Library, Voxels};
    /// use nalgebra::Vector3;
    ///
    /// let _lib = Library::init(0.5)?;
    /// let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;
    /// let normal = sphere.surface_normal(Vector3::new(10.0, 0.0, 0.0));
    /// println!("Normal: {:?}", normal);
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn surface_normal(&self, surface_point: Vector3<f32>) -> Vector3<f32> {
        let point_ffi = crate::types::Vector3f::from(surface_point);
        let mut normal_ffi = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_GetSurfaceNormal(
                self.handle,
                &point_ffi as *const crate::types::Vector3f,
                &mut normal_ffi as *mut crate::types::Vector3f,
            );
        });
        Vector3::from(normal_ffi)
    }

    /// Find the closest point on the surface
    ///
    /// # Arguments
    ///
    /// * `search_point` - Point to search from
    ///
    /// # Returns
    ///
    /// Some(surface_point) if found, None otherwise
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Library, Voxels};
    /// use nalgebra::Vector3;
    ///
    /// let _lib = Library::init(0.5)?;
    /// let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;
    /// if let Some(point) = sphere.closest_point_on_surface(Vector3::new(15.0, 0.0, 0.0)) {
    ///     println!("Closest point: {:?}", point);
    /// }
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn closest_point_on_surface(&self, search_point: Vector3<f32>) -> Option<Vector3<f32>> {
        let search_ffi = crate::types::Vector3f::from(search_point);
        let mut surface_ffi = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let found = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_bClosestPointOnSurface(
                self.handle,
                &search_ffi as *const crate::types::Vector3f,
                &mut surface_ffi as *mut crate::types::Vector3f,
            )
        });
        if found {
            Some(Vector3::from(surface_ffi))
        } else {
            None
        }
    }

    /// Cast a ray to the surface
    ///
    /// # Arguments
    ///
    /// * `ray_origin` - Origin of the ray
    /// * `ray_direction` - Direction of the ray (will be normalized)
    ///
    /// # Returns
    ///
    /// Some(surface_point) if the ray hits the surface, None otherwise
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Library, Voxels};
    /// use nalgebra::Vector3;
    ///
    /// let _lib = Library::init(0.5)?;
    /// let sphere = Voxels::sphere(Vector3::zeros(), 10.0)?;
    /// if let Some(point) = sphere.raycast_to_surface(
    ///     Vector3::new(20.0, 0.0, 0.0),
    ///     Vector3::new(-1.0, 0.0, 0.0)
    /// ) {
    ///     println!("Hit point: {:?}", point);
    /// }
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn raycast_to_surface(
        &self,
        ray_origin: Vector3<f32>,
        ray_direction: Vector3<f32>,
    ) -> Option<Vector3<f32>> {
        let origin_ffi = crate::types::Vector3f::from(ray_origin);
        let direction_ffi = crate::types::Vector3f::from(ray_direction);
        let mut surface_ffi = crate::types::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let found = crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_bRayCastToSurface(
                self.handle,
                &origin_ffi as *const crate::types::Vector3f,
                &direction_ffi as *const crate::types::Vector3f,
                &mut surface_ffi as *mut crate::types::Vector3f,
            )
        });
        if found {
            Some(Vector3::from(surface_ffi))
        } else {
            None
        }
    }

    /// C#-style alias for `raycast_to_surface`.
    pub fn ray_cast_to_surface(
        &self,
        ray_origin: Vector3<f32>,
        ray_direction: Vector3<f32>,
    ) -> Option<Vector3<f32>> {
        self.raycast_to_surface(ray_origin, ray_direction)
    }

    /// Render an implicit signed distance function into the voxels
    pub fn render_implicit(&mut self, implicit: &dyn Implicit, bounds: crate::BBox3) -> Result<()> {
        with_implicit_callback(implicit, |callback| {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::Voxels_RenderImplicit(self.handle, &bounds as *const _, callback);
            });
        })?;
        Ok(())
    }

    /// Intersect the voxel field with an implicit signed distance function
    pub fn intersect_implicit(&mut self, implicit: &dyn Implicit) -> Result<()> {
        with_implicit_callback(implicit, |callback| {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::Voxels_IntersectImplicit(self.handle, callback);
            });
        })?;
        Ok(())
    }

    /// Project a Z slice range
    pub fn project_z_slice(&mut self, start_z_mm: f32, end_z_mm: f32) {
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Voxels_ProjectZSlice(self.handle, start_z_mm, end_z_mm);
        });
    }

    /// Trim voxels outside the specified bounding box
    pub fn trim(&mut self, bbox: crate::BBox3) -> Result<()> {
        let mesh = Mesh::from_bbox(&bbox)?;
        let vox_trim = Voxels::from_mesh(&mesh)?;
        self.bool_intersect(&vox_trim);
        Ok(())
    }

    /// Get field metadata
    pub fn metadata(&self) -> Result<FieldMetadata> {
        FieldMetadata::from_voxels(self)
    }

    /// C#-style alias for `metadata`.
    pub fn meta_data(&self) -> Result<FieldMetadata> {
        self.metadata()
    }

    /// C#-style alias for `metadata`.
    pub fn o_meta_data(&self) -> Result<FieldMetadata> {
        self.metadata()
    }

    // ========================================
    // Functional API (returns new objects)
    // ========================================

    /// Functional: Boolean union
    ///
    /// Returns a new voxel field that is the union of this and the operand.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::{Library, Voxels};
    /// use nalgebra::Vector3;
    ///
    /// let _lib = Library::init(0.5)?;
    /// let sphere1 = Voxels::sphere(Vector3::zeros(), 10.0)?;
    /// let sphere2 = Voxels::sphere(Vector3::new(5.0, 0.0, 0.0), 10.0)?;
    /// let result = sphere1.vox_bool_add(&sphere2)?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn vox_bool_add(&self, operand: &Voxels) -> Result<Voxels> {
        let mut result = self.duplicate()?;
        result.bool_add(operand);
        Ok(result)
    }

    /// Functional: Boolean union with multiple voxel fields
    pub fn vox_bool_add_all<'a, I>(&self, voxels: I) -> Result<Voxels>
    where
        I: IntoIterator<Item = &'a Voxels>,
    {
        let mut result = self.duplicate()?;
        result.bool_add_all(voxels);
        Ok(result)
    }

    /// Functional: Boolean union with smoothing
    pub fn vox_bool_add_smooth(&self, operand: &Voxels, smooth_distance_mm: f32) -> Result<Voxels> {
        let mut result = self.duplicate()?;
        result.bool_add_smooth(operand, smooth_distance_mm);
        Ok(result)
    }

    /// Functional: Boolean difference
    ///
    /// Returns a new voxel field with the operand subtracted from this.
    pub fn vox_bool_subtract(&self, operand: &Voxels) -> Result<Voxels> {
        let mut result = self.duplicate()?;
        result.bool_subtract(operand);
        Ok(result)
    }

    /// Functional: Boolean subtraction with multiple voxel fields
    pub fn vox_bool_subtract_all<'a, I>(&self, voxels: I) -> Result<Voxels>
    where
        I: IntoIterator<Item = &'a Voxels>,
    {
        let mut result = self.duplicate()?;
        result.bool_subtract_all(voxels);
        Ok(result)
    }

    /// Functional: Boolean intersection
    ///
    /// Returns a new voxel field that is the intersection of this and the operand.
    pub fn vox_bool_intersect(&self, operand: &Voxels) -> Result<Voxels> {
        let mut result = self.duplicate()?;
        result.bool_intersect(operand);
        Ok(result)
    }

    /// Functional: Offset
    ///
    /// Returns a new voxel field with the surface offset by the specified distance.
    pub fn vox_offset(&self, dist_mm: f32) -> Result<Voxels> {
        let mut result = self.duplicate()?;
        result.offset(dist_mm);
        Ok(result)
    }

    /// Functional: Double offset
    ///
    /// Returns a new voxel field with double offset applied.
    pub fn vox_double_offset(&self, dist1_mm: f32, dist2_mm: f32) -> Result<Voxels> {
        let mut result = self.duplicate()?;
        result.double_offset(dist1_mm, dist2_mm);
        Ok(result)
    }

    /// Functional: Triple offset (smoothing)
    ///
    /// Returns a new voxel field with triple offset applied.
    pub fn vox_triple_offset(&self, dist_mm: f32) -> Result<Voxels> {
        let mut result = self.duplicate()?;
        result.triple_offset(dist_mm);
        Ok(result)
    }

    /// Functional: Smoothen
    ///
    /// Returns a new voxel field with smoothing applied.
    pub fn vox_smoothen(&self, dist_mm: f32) -> Result<Voxels> {
        self.vox_triple_offset(dist_mm)
    }

    /// Functional: Over offset
    ///
    /// Returns a new voxel field with over offset applied.
    pub fn vox_over_offset(
        &self,
        first_offset_mm: f32,
        final_surface_dist_mm: f32,
    ) -> Result<Voxels> {
        let mut result = self.duplicate()?;
        result.over_offset(first_offset_mm, final_surface_dist_mm);
        Ok(result)
    }

    /// Functional: Fillet
    ///
    /// Returns a new voxel field with fillet applied.
    pub fn vox_fillet(&self, rounding_mm: f32) -> Result<Voxels> {
        self.vox_over_offset(rounding_mm, 0.0)
    }

    /// Functional: Intersect with implicit
    pub fn vox_intersect_implicit(&self, implicit: &dyn Implicit) -> Result<Voxels> {
        let mut result = self.duplicate()?;
        result.intersect_implicit(implicit)?;
        Ok(result)
    }

    /// Functional: Project Z slice range
    pub fn vox_project_z_slice(&self, start_z_mm: f32, end_z_mm: f32) -> Result<Voxels> {
        let mut result = self.duplicate()?;
        result.project_z_slice(start_z_mm, end_z_mm);
        Ok(result)
    }

    /// Get raw handle (for internal use)
    pub(crate) fn handle(&self) -> *mut ffi::CVoxels {
        self.handle
    }

    /// Create from raw handle (for internal use)
    ///
    /// # Safety
    ///
    /// The handle must be a valid CVoxels pointer.
    /// This function takes ownership of the handle.
    pub(crate) fn from_handle(handle: *mut ffi::CVoxels) -> Self {
        Self { handle }
    }
}

impl Drop for Voxels {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::Voxels_Destroy(self.handle);
            });
        }
    }
}

// Voxels is Send + Sync because all native calls are serialized via the crate's re-entrant FFI
// lock (see `ffi_lock.rs` / `SAFETY.md`).
unsafe impl Send for Voxels {}
unsafe impl Sync for Voxels {}

// NOTE: We intentionally do not implement `Clone` for `Voxels`.
// Cloning requires an infallible operation, while duplicating a native object can
// fail (e.g. out-of-memory / null handle). Use `duplicate()` / `try_clone()`.

struct ImplicitCallbackGuard;

impl Drop for ImplicitCallbackGuard {
    fn drop(&mut self) {
        IMPLICIT_CALLBACK_DATA.store(std::ptr::null_mut(), Ordering::SeqCst);
    }
}

fn with_implicit_callback<R>(
    implicit: &dyn Implicit,
    f: impl FnOnce(ffi::ImplicitCallback) -> R,
) -> Result<R> {
    fn call_trampoline(ctx: *mut c_void, pos: Vector3<f32>) -> f32 {
        // Safety: `ctx` points to the `implicit_ref` stack slot in `with_implicit_callback`.
        let imp = unsafe { &*(ctx as *const &dyn Implicit) };
        imp.signed_distance(pos)
    }

    // We store a raw pointer in a process-global slot for the duration of the FFI call.
    // The native library must call back synchronously; we intentionally erase the reference
    // lifetime here by converting it to a raw pointer.
    let implicit_ref: &dyn Implicit = implicit;
    let ctx = (&implicit_ref as *const &dyn Implicit).cast::<c_void>() as *mut c_void;
    let mut data = ImplicitCallbackData {
        ctx,
        call: call_trampoline,
    };
    let data_ptr = &mut data as *mut ImplicitCallbackData;
    let prev = IMPLICIT_CALLBACK_DATA.compare_exchange(
        std::ptr::null_mut(),
        data_ptr,
        Ordering::SeqCst,
        Ordering::SeqCst,
    );
    if prev.is_err() {
        return Err(Error::OperationFailed(
            "Implicit callback already in use".to_string(),
        ));
    }
    let _guard = ImplicitCallbackGuard;
    Ok(f(Some(implicit_trampoline)))
}

fn apply_slice_mode(values: &mut [f32], mode: SliceMode, background: f32) {
    match mode {
        SliceMode::SignedDistance => {}
        SliceMode::BlackWhite => {
            for value in values.iter_mut() {
                *value = if *value <= 0.0 { 0.0 } else { 1.0 };
            }
        }
        SliceMode::Antialiased => {
            for value in values.iter_mut() {
                let v = *value;
                *value = if v <= 0.0 {
                    0.0
                } else if v > background {
                    1.0
                } else {
                    v / background
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_voxels_creation() {
        let _lib = Library::init(0.5).unwrap();
        let vox = Voxels::new();
        assert!(vox.is_ok());
    }

    #[test]
    #[serial]
    fn test_invalid_sphere_radius() {
        let result = Voxels::sphere(Vector3::zeros(), -1.0);
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_shell_allows_negative() {
        let _lib = Library::init(0.5).unwrap();
        let vox = Voxels::new().unwrap();
        let result = vox.shell(-1.0);
        assert!(result.is_ok());
    }
}
