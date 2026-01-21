//! Lattice structure builder

use crate::{ffi, Error, Result};
use nalgebra::Vector3;

/// Lattice structure builder
///
/// Lattices are composed of spheres (nodes) and beams (edges).
/// They are useful for creating lightweight structures.
///
/// # Example
///
/// ```rust,no_run
/// use picogk::{Lattice, Voxels};
/// use nalgebra::Vector3;
///
/// let mut lattice = Lattice::new()?;
/// lattice.add_sphere(Vector3::zeros(), 5.0);
/// lattice.add_beam(
///     Vector3::new(-10.0, 0.0, 0.0),
///     Vector3::new(10.0, 0.0, 0.0),
///     2.0,
///     2.0,
/// );
///
/// let vox = Voxels::from_lattice(&lattice)?;
/// # Ok::<(), picogk::Error>(())
/// ```
pub struct Lattice {
    handle: *mut ffi::CLattice,
}

impl Lattice {
    /// Create an empty lattice
    pub fn new() -> Result<Self> {
        let handle = crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::Lattice_hCreate() });
        if handle.is_null() {
            return Err(Error::NullPointer);
        }
        Ok(Self { handle })
    }

    /// Add a sphere node
    ///
    /// # Arguments
    ///
    /// * `center` - Center position of the sphere
    /// * `radius` - Radius in millimeters
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Lattice;
    /// use nalgebra::Vector3;
    ///
    /// let mut lattice = Lattice::new()?;
    /// lattice.add_sphere(Vector3::zeros(), 5.0);
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn add_sphere(&mut self, center: Vector3<f32>, radius: f32) {
        let center = crate::types::Vector3f::from(center);
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Lattice_AddSphere(
                self.handle,
                &center as *const crate::types::Vector3f,
                radius,
            );
        });
    }

    /// Add a beam edge
    ///
    /// Creates a tapered cylinder connecting two points.
    ///
    /// # Arguments
    ///
    /// * `start` - Start position
    /// * `end` - End position
    /// * `start_radius` - Radius at start point
    /// * `end_radius` - Radius at end point
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Lattice;
    /// use nalgebra::Vector3;
    ///
    /// let mut lattice = Lattice::new()?;
    /// lattice.add_beam(
    ///     Vector3::new(0.0, 0.0, 0.0),
    ///     Vector3::new(10.0, 0.0, 0.0),
    ///     2.0,  // start radius
    ///     1.0,  // end radius (tapered)
    /// );
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn add_beam(
        &mut self,
        start: Vector3<f32>,
        end: Vector3<f32>,
        start_radius: f32,
        end_radius: f32,
    ) {
        self.add_beam_with_cap(start, end, start_radius, end_radius, true);
    }

    /// Add a beam edge with explicit round-cap control
    pub fn add_beam_with_cap(
        &mut self,
        start: Vector3<f32>,
        end: Vector3<f32>,
        start_radius: f32,
        end_radius: f32,
        round_cap: bool,
    ) {
        let start = crate::types::Vector3f::from(start);
        let end = crate::types::Vector3f::from(end);
        crate::ffi_lock::with_ffi_lock(|| unsafe {
            ffi::Lattice_AddBeam(
                self.handle,
                &start as *const crate::types::Vector3f,
                &end as *const crate::types::Vector3f,
                start_radius,
                end_radius,
                round_cap,
            );
        });
    }

    /// Add a uniform beam (same radius at both ends)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Lattice;
    /// use nalgebra::Vector3;
    ///
    /// let mut lattice = Lattice::new()?;
    /// lattice.add_uniform_beam(
    ///     Vector3::new(0.0, 0.0, 0.0),
    ///     Vector3::new(10.0, 0.0, 0.0),
    ///     2.0,
    /// );
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn add_uniform_beam(&mut self, start: Vector3<f32>, end: Vector3<f32>, radius: f32) {
        self.add_beam_with_cap(start, end, radius, radius, true);
    }

    /// Create a cubic lattice
    ///
    /// Generates a regular cubic lattice structure.
    ///
    /// # Arguments
    ///
    /// * `grid_size` - Number of nodes in each dimension
    /// * `spacing` - Distance between nodes
    /// * `node_radius` - Radius of sphere nodes
    /// * `beam_radius` - Radius of connecting beams
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use picogk::Lattice;
    ///
    /// let lattice = Lattice::cubic(5, 10.0, 1.5, 0.8)?;
    /// # Ok::<(), picogk::Error>(())
    /// ```
    pub fn cubic(
        grid_size: usize,
        spacing: f32,
        node_radius: f32,
        beam_radius: f32,
    ) -> Result<Self> {
        let mut lattice = Self::new()?;

        let offset = (grid_size as f32 - 1.0) * spacing * 0.5;

        // Add nodes
        for x in 0..grid_size {
            for y in 0..grid_size {
                for z in 0..grid_size {
                    let pos = Vector3::new(
                        x as f32 * spacing - offset,
                        y as f32 * spacing - offset,
                        z as f32 * spacing - offset,
                    );
                    lattice.add_sphere(pos, node_radius);
                }
            }
        }

        // Add beams
        for x in 0..grid_size {
            for y in 0..grid_size {
                for z in 0..grid_size {
                    let pos = Vector3::new(
                        x as f32 * spacing - offset,
                        y as f32 * spacing - offset,
                        z as f32 * spacing - offset,
                    );

                    // X direction
                    if x < grid_size - 1 {
                        let next = pos + Vector3::new(spacing, 0.0, 0.0);
                        lattice.add_uniform_beam(pos, next, beam_radius);
                    }

                    // Y direction
                    if y < grid_size - 1 {
                        let next = pos + Vector3::new(0.0, spacing, 0.0);
                        lattice.add_uniform_beam(pos, next, beam_radius);
                    }

                    // Z direction
                    if z < grid_size - 1 {
                        let next = pos + Vector3::new(0.0, 0.0, spacing);
                        lattice.add_uniform_beam(pos, next, beam_radius);
                    }
                }
            }
        }

        Ok(lattice)
    }

    /// Create a body-centered cubic (BCC) lattice
    ///
    /// Generates a cubic lattice with an additional node at each cell center,
    /// connecting the center to the eight surrounding corners.
    pub fn body_centered_cubic(
        grid_size: usize,
        spacing: f32,
        node_radius: f32,
        beam_radius: f32,
    ) -> Result<Self> {
        let mut lattice = Self::new()?;
        if grid_size == 0 {
            return Ok(lattice);
        }

        let offset = (grid_size as f32 - 1.0) * spacing * 0.5;

        // Add corner nodes
        for x in 0..grid_size {
            for y in 0..grid_size {
                for z in 0..grid_size {
                    let pos = Vector3::new(
                        x as f32 * spacing - offset,
                        y as f32 * spacing - offset,
                        z as f32 * spacing - offset,
                    );
                    lattice.add_sphere(pos, node_radius);
                }
            }
        }

        if grid_size < 2 {
            return Ok(lattice);
        }

        // Add center nodes and diagonal beams per cell
        for x in 0..(grid_size - 1) {
            for y in 0..(grid_size - 1) {
                for z in 0..(grid_size - 1) {
                    let center = Vector3::new(
                        (x as f32 + 0.5) * spacing - offset,
                        (y as f32 + 0.5) * spacing - offset,
                        (z as f32 + 0.5) * spacing - offset,
                    );
                    lattice.add_sphere(center, node_radius);

                    for dx in 0..=1 {
                        for dy in 0..=1 {
                            for dz in 0..=1 {
                                let corner = Vector3::new(
                                    (x + dx) as f32 * spacing - offset,
                                    (y + dy) as f32 * spacing - offset,
                                    (z + dz) as f32 * spacing - offset,
                                );
                                lattice.add_uniform_beam(center, corner, beam_radius);
                            }
                        }
                    }
                }
            }
        }

        Ok(lattice)
    }

    /// Create a face-centered cubic (FCC) lattice
    ///
    /// Adds face-center nodes on each cube face and connects them to the four face corners.
    /// This is a purely "builder" convenience on top of `add_sphere` / `add_beam` and does not
    /// require additional native support.
    pub fn face_centered_cubic(
        grid_size: usize,
        spacing: f32,
        node_radius: f32,
        beam_radius: f32,
    ) -> Result<Self> {
        let mut lattice = Self::new()?;
        if grid_size == 0 {
            return Ok(lattice);
        }

        let offset = (grid_size as f32 - 1.0) * spacing * 0.5;
        let corner = |x: usize, y: usize, z: usize| -> Vector3<f32> {
            Vector3::new(
                x as f32 * spacing - offset,
                y as f32 * spacing - offset,
                z as f32 * spacing - offset,
            )
        };

        // Add corner nodes.
        for x in 0..grid_size {
            for y in 0..grid_size {
                for z in 0..grid_size {
                    lattice.add_sphere(corner(x, y, z), node_radius);
                }
            }
        }

        if grid_size < 2 {
            return Ok(lattice);
        }

        // XY faces (z fixed).
        for x in 0..(grid_size - 1) {
            for y in 0..(grid_size - 1) {
                for z in 0..grid_size {
                    let center = Vector3::new(
                        (x as f32 + 0.5) * spacing - offset,
                        (y as f32 + 0.5) * spacing - offset,
                        z as f32 * spacing - offset,
                    );
                    lattice.add_sphere(center, node_radius);
                    for c in [
                        corner(x, y, z),
                        corner(x + 1, y, z),
                        corner(x, y + 1, z),
                        corner(x + 1, y + 1, z),
                    ] {
                        lattice.add_uniform_beam(center, c, beam_radius);
                    }
                }
            }
        }

        // XZ faces (y fixed).
        for x in 0..(grid_size - 1) {
            for y in 0..grid_size {
                for z in 0..(grid_size - 1) {
                    let center = Vector3::new(
                        (x as f32 + 0.5) * spacing - offset,
                        y as f32 * spacing - offset,
                        (z as f32 + 0.5) * spacing - offset,
                    );
                    lattice.add_sphere(center, node_radius);
                    for c in [
                        corner(x, y, z),
                        corner(x + 1, y, z),
                        corner(x, y, z + 1),
                        corner(x + 1, y, z + 1),
                    ] {
                        lattice.add_uniform_beam(center, c, beam_radius);
                    }
                }
            }
        }

        // YZ faces (x fixed).
        for x in 0..grid_size {
            for y in 0..(grid_size - 1) {
                for z in 0..(grid_size - 1) {
                    let center = Vector3::new(
                        x as f32 * spacing - offset,
                        (y as f32 + 0.5) * spacing - offset,
                        (z as f32 + 0.5) * spacing - offset,
                    );
                    lattice.add_sphere(center, node_radius);
                    for c in [
                        corner(x, y, z),
                        corner(x, y + 1, z),
                        corner(x, y, z + 1),
                        corner(x, y + 1, z + 1),
                    ] {
                        lattice.add_uniform_beam(center, c, beam_radius);
                    }
                }
            }
        }

        Ok(lattice)
    }

    /// Check if the lattice is valid
    pub fn is_valid(&self) -> bool {
        crate::ffi_lock::with_ffi_lock(|| unsafe { ffi::Lattice_bIsValid(self.handle) })
    }

    /// Get raw handle (for internal use)
    pub(crate) fn handle(&self) -> *mut ffi::CLattice {
        self.handle
    }
}

impl Drop for Lattice {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            crate::ffi_lock::with_ffi_lock(|| unsafe {
                ffi::Lattice_Destroy(self.handle);
            });
        }
    }
}

unsafe impl Send for Lattice {}
unsafe impl Sync for Lattice {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Library;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_lattice_creation() {
        let _lib = Library::init(0.5).unwrap();
        let lattice = Lattice::new();
        assert!(lattice.is_ok());
    }

    #[test]
    #[serial]
    fn test_add_sphere() {
        let _lib = Library::init(0.5).unwrap();
        let mut lattice = Lattice::new().unwrap();
        lattice.add_sphere(Vector3::zeros(), 5.0);
        assert!(lattice.is_valid());
    }

    #[test]
    #[serial]
    fn test_cubic_lattice() {
        let _lib = Library::init(0.5).unwrap();
        let lattice = Lattice::cubic(3, 10.0, 1.5, 0.8);
        assert!(lattice.is_ok());
    }

    #[test]
    #[serial]
    fn test_bcc_lattice() {
        let _lib = Library::init(0.5).unwrap();
        let lattice = Lattice::body_centered_cubic(3, 10.0, 1.5, 0.8);
        assert!(lattice.is_ok());
    }

    #[test]
    #[serial]
    fn test_fcc_lattice() {
        let _lib = Library::init(0.5).unwrap();
        let lattice = Lattice::face_centered_cubic(3, 10.0, 1.5, 0.8);
        assert!(lattice.is_ok());
    }
}
