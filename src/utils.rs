//! Utility helpers and mesh primitives

use crate::{BBox3, Error, Library, Matrix4x4, Mesh, Result};
use nalgebra::Vector3;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

pub struct Utils;

impl Utils {
    pub fn strip_quotes_from_path(path: &str) -> String {
        if path.starts_with('"') && path.ends_with('"') && path.len() >= 2 {
            path[1..path.len() - 1].to_string()
        } else {
            path.to_string()
        }
    }

    pub fn wait_for_file_existence<P: AsRef<Path>>(path: P, timeout_secs: f32) -> bool {
        let start = Instant::now();
        let timeout = Duration::from_secs_f32(timeout_secs.max(0.0));

        while start.elapsed() < timeout {
            if path.as_ref().exists() {
                return true;
            }
            thread::sleep(Duration::from_millis(100));
        }

        false
    }

    pub fn home_folder() -> Result<PathBuf> {
        if cfg!(unix) {
            env::var("HOME")
                .map(PathBuf::from)
                .map_err(|_| Error::OperationFailed("Could not find home folder".to_string()))
        } else if cfg!(windows) {
            let drive = env::var("HOMEDRIVE").unwrap_or_default();
            let path = env::var("HOMEPATH").unwrap_or_default();
            if drive.is_empty() && path.is_empty() {
                Err(Error::OperationFailed(
                    "Could not find home folder".to_string(),
                ))
            } else {
                Ok(PathBuf::from(format!("{}{}", drive, path)))
            }
        } else {
            Err(Error::OperationFailed(
                "Could not find home folder".to_string(),
            ))
        }
    }

    pub fn documents_folder() -> Result<PathBuf> {
        if cfg!(unix) {
            let home = Self::home_folder()?;
            Ok(home.join("Documents"))
        } else {
            Self::home_folder()
        }
    }

    pub fn project_root_folder() -> Result<PathBuf> {
        let mut path = env::current_exe()
            .map_err(|e| Error::OperationFailed(format!("Failed to get current exe: {}", e)))?;

        for _ in 0..4 {
            if !path.pop() {
                return Err(Error::OperationFailed(
                    "Failed to determine project root folder".to_string(),
                ));
            }
        }

        Ok(path)
    }

    pub fn picogk_source_code_folder() -> Result<PathBuf> {
        Ok(Self::project_root_folder()?.join("PicoGK"))
    }

    /// C#-style alias for `picogk_source_code_folder` (`strPicoGKSourceCodeFolder`).
    pub fn pico_gk_source_code_folder() -> Result<PathBuf> {
        Self::picogk_source_code_folder()
    }

    /// C#-style alias returning a string path.
    pub fn str_pico_gk_source_code_folder() -> Result<String> {
        Ok(Self::picogk_source_code_folder()?
            .to_string_lossy()
            .to_string())
    }

    pub fn executable_folder() -> Result<PathBuf> {
        env::current_exe()
            .map_err(|e| Error::OperationFailed(format!("Failed to get current exe: {}", e)))
            .and_then(|path| {
                path.parent().map(|p| p.to_path_buf()).ok_or_else(|| {
                    Error::OperationFailed("Failed to get executable folder".to_string())
                })
            })
    }

    pub fn date_time_filename(prefix: &str, postfix: &str) -> String {
        let now = chrono::Local::now();
        format!("{}{}{}", prefix, now.format("%Y%m%d_%H%M%S"), postfix)
    }

    pub fn shorten(text: &str, max_chars: usize) -> String {
        if text.chars().count() <= max_chars {
            text.to_string()
        } else {
            text.chars().take(max_chars).collect()
        }
    }

    pub fn set_matrix_row(
        mat: &mut Matrix4x4,
        row: u32,
        f1: f32,
        f2: f32,
        f3: f32,
        f4: f32,
    ) -> Result<()> {
        match row {
            0 => {
                mat.m11 = f1;
                mat.m12 = f2;
                mat.m13 = f3;
                mat.m14 = f4;
            }
            1 => {
                mat.m21 = f1;
                mat.m22 = f2;
                mat.m23 = f3;
                mat.m24 = f4;
            }
            2 => {
                mat.m31 = f1;
                mat.m32 = f2;
                mat.m33 = f3;
                mat.m34 = f4;
            }
            3 => {
                mat.m41 = f1;
                mat.m42 = f2;
                mat.m43 = f3;
                mat.m44 = f4;
            }
            _ => {
                return Err(Error::InvalidParameter(
                    "Matrix 4x4 row index must be 0..3".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn mat_look_at(eye: Vector3<f32>, look_at: Vector3<f32>) -> Matrix4x4 {
        let vec_z = Vector3::new(0.0, 0.0, 1.0);
        let view = (eye - look_at).normalize();
        let right = vec_z.cross(&view).normalize();
        let up = view.cross(&right);

        let mut mat = Matrix4x4::identity();
        let _ = Self::set_matrix_row(&mut mat, 0, right.x, up.x, view.x, 0.0);
        let _ = Self::set_matrix_row(&mut mat, 1, right.y, up.y, view.y, 0.0);
        let _ = Self::set_matrix_row(&mut mat, 2, right.z, up.z, view.z, 0.0);
        let _ = Self::set_matrix_row(
            &mut mat,
            3,
            -right.dot(&eye),
            -up.dot(&eye),
            -view.dot(&eye),
            1.0,
        );
        mat
    }

    pub fn msh_create_cube_from_bbox(bbox: &BBox3) -> Result<Mesh> {
        Mesh::from_bbox(bbox)
    }

    pub fn msh_create_cube(
        scale: Option<Vector3<f32>>,
        offset_mm: Option<Vector3<f32>>,
    ) -> Result<Mesh> {
        let vec_s = scale.unwrap_or_else(|| Vector3::new(1.0, 1.0, 1.0));
        let offset = offset_mm.unwrap_or_else(|| Vector3::new(0.0, 0.0, 0.0));
        let bbox = BBox3::from_center_size(offset, vec_s);
        Mesh::from_bbox(&bbox)
    }

    pub fn msh_create_cylinder(
        scale: Option<Vector3<f32>>,
        offset_mm: Option<Vector3<f32>>,
        sides: Option<usize>,
    ) -> Result<Mesh> {
        let vec_s = scale.unwrap_or_else(|| Vector3::new(1.0, 1.0, 1.0));
        let offset = offset_mm.unwrap_or_else(|| Vector3::new(0.0, 0.0, 0.0));

        let mut sides = sides.unwrap_or(0) as i32;
        let f_a = vec_s.x * 0.5;
        let f_b = vec_s.y * 0.5;

        if sides <= 0 {
            let voxel = Library::voxel_size_mm().max(1e-6);
            let f_vox_a = f_a / voxel;
            let f_vox_b = f_b / voxel;
            let f_p = std::f32::consts::PI
                * (3.0 * (f_vox_a + f_vox_b)
                    - ((3.0 * f_vox_a + f_vox_b) * (f_vox_a + 3.0 * f_vox_b)).sqrt());
            sides = 2 * f_p.ceil() as i32;
        }

        if sides < 3 {
            sides = 3;
        }

        let mut mesh = Mesh::new()?;
        let mut bottom_center = offset;
        bottom_center.z -= vec_s.z * 0.5;
        let mut top_center = bottom_center;
        top_center.z += vec_s.z;

        let mut prev_bottom = Vector3::new(f_a, 0.0, 0.0) + bottom_center;
        let mut prev_top = prev_bottom;
        prev_top.z += vec_s.z;

        let step = std::f32::consts::PI * 2.0 / sides as f32;

        for i in 1..=sides {
            let angle = i as f32 * step;
            let this_bottom =
                Vector3::new(angle.cos() * f_a, angle.sin() * f_b, 0.0) + bottom_center;
            let mut this_top = this_bottom;
            this_top.z += vec_s.z;

            add_triangle(&mut mesh, top_center, prev_top, this_top);
            add_triangle(&mut mesh, prev_bottom, this_bottom, prev_top);
            add_triangle(&mut mesh, this_bottom, this_top, prev_top);
            add_triangle(&mut mesh, bottom_center, this_bottom, prev_bottom);

            prev_bottom = this_bottom;
            prev_top = this_top;
        }

        Ok(mesh)
    }

    pub fn msh_create_cone(
        scale: Option<Vector3<f32>>,
        offset_mm: Option<Vector3<f32>>,
        sides: Option<usize>,
    ) -> Result<Mesh> {
        let vec_s = scale.unwrap_or_else(|| Vector3::new(1.0, 1.0, 1.0));
        let offset = offset_mm.unwrap_or_else(|| Vector3::new(0.0, 0.0, 0.0));

        let mut sides = sides.unwrap_or(0) as i32;
        let f_a = vec_s.x * 0.5;
        let f_b = vec_s.y * 0.5;

        if sides <= 0 {
            let voxel = Library::voxel_size_mm().max(1e-6);
            let f_vox_a = f_a / voxel;
            let f_vox_b = f_b / voxel;
            let f_p = std::f32::consts::PI
                * (3.0 * (f_vox_a + f_vox_b)
                    - ((3.0 * f_vox_a + f_vox_b) * (f_vox_a + 3.0 * f_vox_b)).sqrt());
            sides = 2 * f_p.ceil() as i32;
        }

        if sides < 3 {
            sides = 3;
        }

        let mut mesh = Mesh::new()?;
        let mut bottom_center = offset;
        bottom_center.z -= vec_s.z * 0.5;
        let mut top = bottom_center;
        top.z += vec_s.z;
        let mut prev_bottom = Vector3::new(f_a, 0.0, 0.0) + bottom_center;

        let step = std::f32::consts::PI * 2.0 / sides as f32;

        for i in 1..=sides {
            let angle = i as f32 * step;
            let this_bottom =
                Vector3::new(angle.cos() * f_a, angle.sin() * f_b, 0.0) + bottom_center;

            add_triangle(&mut mesh, prev_bottom, this_bottom, top);
            add_triangle(&mut mesh, bottom_center, this_bottom, prev_bottom);

            prev_bottom = this_bottom;
        }

        Ok(mesh)
    }

    pub fn msh_create_geosphere(
        scale: Option<Vector3<f32>>,
        offset_mm: Option<Vector3<f32>>,
        subdivisions: Option<usize>,
    ) -> Result<Mesh> {
        let vec_s = scale.unwrap_or_else(|| Vector3::new(1.0, 1.0, 1.0));
        let offset = offset_mm.unwrap_or_else(|| Vector3::new(0.0, 0.0, 0.0));

        let mut mesh = Mesh::new()?;
        let vec_radii = vec_s * 0.5;
        let vec_radii2 = vec_radii.component_mul(&vec_radii);

        let f_coeff = squared(2.0 * (std::f32::consts::PI * 0.2).sin());
        let vec_penta = Vector3::new(
            (2.0 * (f_coeff * vec_radii2.x - vec_radii2.x).sqrt()) / f_coeff,
            (2.0 * (f_coeff * vec_radii2.y - vec_radii2.y).sqrt()) / f_coeff,
            (2.0 * (f_coeff * vec_radii2.z - vec_radii2.z).sqrt()) / f_coeff,
        );

        let f_penta_dz = (vec_radii2.z - squared(vec_penta.z)).sqrt();
        let mut p_offs = [Vector3::zeros(); 5];
        for (i, p_off) in p_offs.iter_mut().enumerate() {
            let angle = 0.4 * std::f32::consts::PI * i as f32;
            *p_off = Vector3::new(
                vec_penta.x * angle.cos(),
                vec_penta.y * angle.sin(),
                f_penta_dz,
            );
        }

        let mut subdivisions = subdivisions.unwrap_or(0) as i32;
        if subdivisions <= 0 {
            let target_triangles = (approx_ellipsoid_surface_area(vec_radii)
                / Library::voxel_size_mm().max(1e-6)
                / Library::voxel_size_mm().max(1e-6))
            .ceil() as i32;
            subdivisions = 1;
            let mut triangles = 80;
            while subdivisions < 8 && triangles < target_triangles {
                subdivisions += 1;
                triangles = 20 * (1 << (2 * subdivisions));
            }
        }

        let mut cap = offset;
        cap.z += vec_radii.z;

        for (&curr, &next) in p_offs
            .iter()
            .zip(p_offs.iter().skip(1).chain(std::iter::once(&p_offs[0])))
        {
            geo_sphere_triangle(
                cap,
                offset + curr,
                offset + next,
                offset,
                vec_radii,
                subdivisions,
                &mut mesh,
            );
        }

        geo_sphere_triangle(
            offset + p_offs[4],
            offset - p_offs[2],
            offset + p_offs[0],
            offset,
            vec_radii,
            subdivisions,
            &mut mesh,
        );
        geo_sphere_triangle(
            offset + p_offs[4],
            offset - p_offs[1],
            offset - p_offs[2],
            offset,
            vec_radii,
            subdivisions,
            &mut mesh,
        );
        geo_sphere_triangle(
            offset + p_offs[3],
            offset - p_offs[1],
            offset + p_offs[4],
            offset,
            vec_radii,
            subdivisions,
            &mut mesh,
        );
        geo_sphere_triangle(
            offset + p_offs[3],
            offset - p_offs[0],
            offset - p_offs[1],
            offset,
            vec_radii,
            subdivisions,
            &mut mesh,
        );
        geo_sphere_triangle(
            offset + p_offs[2],
            offset - p_offs[0],
            offset + p_offs[3],
            offset,
            vec_radii,
            subdivisions,
            &mut mesh,
        );
        geo_sphere_triangle(
            offset + p_offs[2],
            offset - p_offs[4],
            offset - p_offs[0],
            offset,
            vec_radii,
            subdivisions,
            &mut mesh,
        );
        geo_sphere_triangle(
            offset + p_offs[1],
            offset - p_offs[4],
            offset + p_offs[2],
            offset,
            vec_radii,
            subdivisions,
            &mut mesh,
        );
        geo_sphere_triangle(
            offset + p_offs[1],
            offset - p_offs[3],
            offset - p_offs[4],
            offset,
            vec_radii,
            subdivisions,
            &mut mesh,
        );
        geo_sphere_triangle(
            offset + p_offs[0],
            offset - p_offs[3],
            offset + p_offs[1],
            offset,
            vec_radii,
            subdivisions,
            &mut mesh,
        );
        geo_sphere_triangle(
            offset + p_offs[0],
            offset - p_offs[2],
            offset - p_offs[3],
            offset,
            vec_radii,
            subdivisions,
            &mut mesh,
        );

        cap.z = offset.z - vec_radii.z;
        for (&curr, &next) in p_offs
            .iter()
            .zip(p_offs.iter().skip(1).chain(std::iter::once(&p_offs[0])))
        {
            geo_sphere_triangle(
                cap,
                offset - next,
                offset - curr,
                offset,
                vec_radii,
                subdivisions,
                &mut mesh,
            );
        }

        Ok(mesh)
    }

    /// C# `mshCreateGeoSphere` alias for `msh_create_geosphere`.
    pub fn msh_create_geo_sphere(
        scale: Option<Vector3<f32>>,
        offset_mm: Option<Vector3<f32>>,
        subdivisions: Option<usize>,
    ) -> Result<Mesh> {
        Self::msh_create_geosphere(scale, offset_mm, subdivisions)
    }

    /// Convenience alias for `msh_create_geosphere`.
    pub fn create_geo_sphere(
        scale: Option<Vector3<f32>>,
        offset_mm: Option<Vector3<f32>>,
        subdivisions: Option<usize>,
    ) -> Result<Mesh> {
        Self::msh_create_geosphere(scale, offset_mm, subdivisions)
    }
}

pub struct TempFolder {
    path: PathBuf,
}

impl TempFolder {
    pub fn new() -> Result<Self> {
        let mut path = env::temp_dir();
        let unique = format!(
            "picogk_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        path.push(unique);
        fs::create_dir_all(&path)
            .map_err(|e| Error::OperationFailed(format!("Failed to create temp dir: {}", e)))?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let _ = fs::remove_file(path);
                }
            }
        }
        let _ = fs::remove_dir(&self.path);
    }
}

fn add_triangle(mesh: &mut Mesh, a: Vector3<f32>, b: Vector3<f32>, c: Vector3<f32>) {
    let i0 = mesh.add_vertex(a);
    let i1 = mesh.add_vertex(b);
    let i2 = mesh.add_vertex(c);
    mesh.add_triangle(crate::Triangle::new(i0, i1, i2));
}

fn geo_sphere_triangle(
    a: Vector3<f32>,
    b: Vector3<f32>,
    c: Vector3<f32>,
    offset: Vector3<f32>,
    radii: Vector3<f32>,
    depth: i32,
    target: &mut Mesh,
) {
    if depth > 0 {
        let mut ab = offset + (a + b) * 0.5 - offset;
        let mut bc = offset + (b + c) * 0.5 - offset;
        let mut ca = offset + (c + a) * 0.5 - offset;

        ab = ab.component_mul(&radii) / ab.norm();
        bc = bc.component_mul(&radii) / bc.norm();
        ca = ca.component_mul(&radii) / ca.norm();

        geo_sphere_triangle(a, ab, ca, offset, radii, depth - 1, target);
        geo_sphere_triangle(ab, b, bc, offset, radii, depth - 1, target);
        geo_sphere_triangle(ab, bc, ca, offset, radii, depth - 1, target);
        geo_sphere_triangle(ca, bc, c, offset, radii, depth - 1, target);
    } else {
        add_triangle(target, a, b, c);
    }
}

fn squared(x: f32) -> f32 {
    x * x
}

fn approx_ellipsoid_surface_area(vec_abc: Vector3<f32>) -> f32 {
    let term = (vec_abc.x * vec_abc.y).powf(1.6)
        + (vec_abc.y * vec_abc.z).powf(1.6)
        + (vec_abc.z * vec_abc.x).powf(1.6);
    4.0 * std::f32::consts::PI * (term / 3.0).powf(1.0 / 1.6)
}
