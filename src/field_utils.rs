//! Field utilities and SDF visualization helpers

use crate::{
    ColorFloat, ColorHLS, Error, ImageColor, PolyLine, Result, ScalarField, TgaIo, VectorField,
    Viewer, Voxels,
};
use nalgebra::Vector3;
use std::path::Path;

pub struct SdfVisualizer;

impl SdfVisualizer {
    #[allow(clippy::too_many_arguments)]
    pub fn encode_from_sdf(
        field: &ScalarField,
        background_value: f32,
        slice: i32,
        background: Option<ColorFloat>,
        surface: Option<ColorFloat>,
        inside: Option<ColorFloat>,
        outside: Option<ColorFloat>,
        defect: Option<ColorFloat>,
    ) -> ImageColor {
        let background = background.unwrap_or_else(|| {
            ColorFloat::from_hex("0066ff").unwrap_or(ColorFloat::new(0.0, 0.4, 1.0, 1.0))
        });
        let surface = surface
            .unwrap_or_else(|| ColorFloat::from_hex("FF").unwrap_or(ColorFloat::gray(1.0, 1.0)));
        let inside = inside.unwrap_or_else(|| {
            ColorFloat::from_hex("cc33ff").unwrap_or(ColorFloat::new(0.8, 0.2, 1.0, 1.0))
        });
        let outside = outside.unwrap_or_else(|| {
            ColorFloat::from_hex("33cc33").unwrap_or(ColorFloat::new(0.2, 0.8, 0.2, 1.0))
        });
        let defect = defect.unwrap_or_else(|| {
            ColorFloat::from_hex("ff5400").unwrap_or(ColorFloat::new(1.0, 0.33, 0.0, 1.0))
        });

        let dims = field.voxel_dimensions();
        let width = dims.size.x.max(0) as usize;
        let height = dims.size.y.max(0) as usize;
        let depth = dims.size.z.max(0);

        let mut img = ImageColor::new(width, height);
        if slice >= depth {
            return img;
        }

        let origin = dims.origin;
        for x in 0..width {
            for y in 0..height {
                let coord = crate::Library::voxels_to_mm(Vector3::new(
                    (origin.x + x as i32) as f32,
                    (origin.y + y as i32) as f32,
                    (origin.z + slice) as f32,
                ));

                let mut value = 0.0f32;
                let is_set = if let Some(v) = field.get_value(coord) {
                    value = v;
                    true
                } else {
                    false
                };

                let mut clr: ColorHLS;
                if value.is_nan() || value.is_infinite() {
                    clr = defect.into();
                } else if value.abs() < f32::EPSILON {
                    clr = surface.into();
                } else if (value - background_value).abs() < f32::EPSILON {
                    clr = background.into();
                } else {
                    if value < 0.0 {
                        clr = inside.into();
                        value = -value;
                    } else {
                        clr = outside.into();
                    }

                    if value > background_value {
                        clr.s = 1.0;
                    } else if background_value != 0.0 {
                        clr.l = 0.7 - (value / background_value / 2.0);
                    }
                }

                if !is_set {
                    clr.s = 0.3;
                }

                img.set_value(x, y, ColorFloat::from(clr));
            }
        }

        img
    }

    pub fn does_slice_contain_defect(field: &ScalarField, slice: i32) -> bool {
        let dims = field.voxel_dimensions();
        let width = dims.size.x.max(0) as usize;
        let height = dims.size.y.max(0) as usize;
        let depth = dims.size.z.max(0);

        if slice >= depth {
            return false;
        }

        let origin = dims.origin;
        for x in 0..width {
            for y in 0..height {
                let coord = crate::Library::voxels_to_mm(Vector3::new(
                    (origin.x + x as i32) as f32,
                    (origin.y + y as i32) as f32,
                    (origin.z + slice) as f32,
                ));

                if let Some(value) = field.get_value(coord) {
                    if value.is_nan() || value.is_infinite() {
                        return true;
                    }
                }
            }
        }

        false
    }

    #[allow(clippy::too_many_arguments)]
    pub fn visualize_sdf_slices_as_tga_stack<P: AsRef<Path>>(
        field: &ScalarField,
        background_value: f32,
        path: P,
        file_prefix: &str,
        only_defective: bool,
        background: Option<ColorFloat>,
        surface: Option<ColorFloat>,
        inside: Option<ColorFloat>,
        outside: Option<ColorFloat>,
        defect: Option<ColorFloat>,
    ) -> Result<bool> {
        let dims = field.voxel_dimensions();
        let depth = dims.size.z.max(0);
        let mut contains_defects = false;

        for slice in 0..depth {
            if Self::does_slice_contain_defect(field, slice) {
                contains_defects = true;
            } else if only_defective {
                continue;
            }

            let file_name = format!("{file_prefix}{:05}.tga", slice);
            let file_path = path.as_ref().join(file_name);
            let img = Self::encode_from_sdf(
                field,
                background_value,
                slice,
                background,
                surface,
                inside,
                outside,
                defect,
            );
            TgaIo::save_tga(file_path, &img)?;
        }

        Ok(contains_defects)
    }
}

pub struct ActiveVoxelCounterScalar;

impl ActiveVoxelCounterScalar {
    pub fn count(field: &ScalarField) -> Result<usize> {
        let mut count = 0usize;
        field.traverse_active(|_, _| {
            count += 1;
        })?;
        Ok(count)
    }
}

pub struct SurfaceNormalFieldExtractor;

impl SurfaceNormalFieldExtractor {
    pub fn extract(
        voxels: &Voxels,
        surface_threshold_vx: f32,
        direction_filter: Option<Vector3<f32>>,
        direction_filter_tolerance: f32,
        scale_by: Option<Vector3<f32>>,
    ) -> Result<VectorField> {
        if !(0.0..=1.0).contains(&direction_filter_tolerance) {
            return Err(Error::InvalidParameter(
                "direction_filter_tolerance must be between 0 and 1".to_string(),
            ));
        }

        let mut field = VectorField::new()?;
        let source = ScalarField::from_voxels(voxels)?;
        let mut direction = direction_filter.unwrap_or_else(Vector3::zeros);
        let scale_by = scale_by.unwrap_or_else(|| Vector3::new(1.0, 1.0, 1.0));

        if direction != Vector3::zeros() {
            direction = direction.normalize();
        }

        source.traverse_active(|position, value| {
            if value.abs() > surface_threshold_vx {
                return;
            }

            let normal = voxels.surface_normal(position);
            if direction != Vector3::zeros() {
                let deviation = (1.0 - normal.dot(&direction)).abs();
                if deviation > direction_filter_tolerance {
                    return;
                }
            }

            field.set_value(position, normal.component_mul(&scale_by));
        })?;

        Ok(field)
    }
}

pub struct VectorFieldMerge;

impl VectorFieldMerge {
    pub fn merge(source: &VectorField, target: &mut VectorField) -> Result<()> {
        source.traverse_active(|position, value| {
            target.set_value(position, value);
        })?;
        Ok(())
    }
}

pub struct AddVectorFieldToViewer;

impl AddVectorFieldToViewer {
    pub fn add_to_viewer(
        viewer: &Viewer,
        field: &VectorField,
        color: ColorFloat,
        step: usize,
        arrow_size: f32,
        group: i32,
    ) -> Result<()> {
        if step == 0 {
            return Err(Error::InvalidParameter(
                "step must be greater than 0".to_string(),
            ));
        }

        let mut count = 0usize;
        field.traverse_active(|position, value| {
            count += 1;
            if count < step {
                return;
            }
            count = 0;

            if let Ok(mut poly) = PolyLine::new(color) {
                poly.add_vertex(position);

                if value == Vector3::zeros() {
                    poly.add_cross(arrow_size);
                } else {
                    poly.add_vertex(position + value);
                    poly.add_arrow(arrow_size, None);
                }

                viewer.add_polyline(poly, group);
            }
        })?;

        Ok(())
    }
}
