//! Basic types for PicoGK

use nalgebra::{Vector2, Vector3};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Type alias for Vector3<f32> for FFI compatibility
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Type alias for Vector2<f32> for FFI compatibility
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vector2f {
    pub x: f32,
    pub y: f32,
}

/// Floating-point RGBA color
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorFloat {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorFloat {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn gray(value: f32, alpha: f32) -> Self {
        Self {
            r: value,
            g: value,
            b: value,
            a: alpha,
        }
    }

    pub fn from_hex(hex: &str) -> std::result::Result<Self, String> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        let bytes = hex.as_bytes();
        let to_byte = |start: usize| -> std::result::Result<u8, String> {
            let s = std::str::from_utf8(&bytes[start..start + 2]).map_err(|e| e.to_string())?;
            u8::from_str_radix(s, 16).map_err(|e| e.to_string())
        };

        match hex.len() {
            2 | 4 => {
                let value = to_byte(0)? as f32 / 255.0;
                let alpha = if hex.len() == 4 {
                    to_byte(2)? as f32 / 255.0
                } else {
                    1.0
                };
                Ok(Self::gray(value, alpha))
            }
            6 | 8 => {
                let r = to_byte(0)? as f32 / 255.0;
                let g = to_byte(2)? as f32 / 255.0;
                let b = to_byte(4)? as f32 / 255.0;
                let a = if hex.len() == 8 {
                    to_byte(6)? as f32 / 255.0
                } else {
                    1.0
                };
                Ok(Self::new(r, g, b, a))
            }
            _ => Err(format!(
                "Invalid hex color '{}': expected 2, 4, 6, or 8 hex digits",
                hex
            )),
        }
    }

    pub fn to_hex(&self) -> String {
        let rgba = ColorRgba32::from(*self);
        if rgba.r == rgba.g && rgba.g == rgba.b {
            let mut result = format!("{:02x}", rgba.r);
            if rgba.a != 255 {
                result.push_str(&format!("{:02x}", rgba.a));
            }
            result
        } else {
            let mut result = format!("{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b);
            if rgba.a != 255 {
                result.push_str(&format!("{:02x}", rgba.a));
            }
            result
        }
    }

    pub fn to_abgr_hex(&self) -> String {
        let rgba = ColorRgba32::from(*self);
        format!("{:02x}{:02x}{:02x}{:02x}", rgba.a, rgba.b, rgba.g, rgba.r)
    }

    /// C#-style alias for `to_hex`.
    pub fn str_as_hex_code(&self) -> String {
        self.to_hex()
    }

    /// C#-style alias for `to_abgr_hex`.
    pub fn str_as_abgr_hex_code(&self) -> String {
        self.to_abgr_hex()
    }

    pub fn weighted(a: ColorFloat, b: ColorFloat, weight: f32) -> ColorFloat {
        let w = weight.clamp(0.0, 1.0);
        let inv = 1.0 - w;
        ColorFloat::new(
            a.r * inv + b.r * w,
            a.g * inv + b.g * w,
            a.b * inv + b.b * w,
            a.a * inv + b.a * w,
        )
    }

    pub fn random<F>(mut next_f32: F) -> ColorFloat
    where
        F: FnMut() -> f32,
    {
        ColorFloat::new(next_f32(), next_f32(), next_f32(), 1.0)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorRgb24 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<ColorFloat> for ColorRgb24 {
    fn from(value: ColorFloat) -> Self {
        Self {
            r: (value.r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (value.g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (value.b.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }
}

impl From<ColorRgb24> for ColorFloat {
    fn from(value: ColorRgb24) -> Self {
        ColorFloat::new(
            value.r as f32 / 255.0,
            value.g as f32 / 255.0,
            value.b as f32 / 255.0,
            1.0,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorRgba32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<ColorFloat> for ColorRgba32 {
    fn from(value: ColorFloat) -> Self {
        Self {
            r: (value.r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (value.g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (value.b.clamp(0.0, 1.0) * 255.0) as u8,
            a: (value.a.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }
}

impl From<ColorRgba32> for ColorFloat {
    fn from(value: ColorRgba32) -> Self {
        ColorFloat::new(
            value.r as f32 / 255.0,
            value.g as f32 / 255.0,
            value.b as f32 / 255.0,
            value.a as f32 / 255.0,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorBgr24 {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl From<ColorFloat> for ColorBgr24 {
    fn from(value: ColorFloat) -> Self {
        Self {
            r: (value.r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (value.g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (value.b.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }
}

impl From<ColorBgr24> for ColorFloat {
    fn from(value: ColorBgr24) -> Self {
        ColorFloat::new(
            value.r as f32 / 255.0,
            value.g as f32 / 255.0,
            value.b as f32 / 255.0,
            1.0,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorBgra32 {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

impl From<ColorFloat> for ColorBgra32 {
    fn from(value: ColorFloat) -> Self {
        Self {
            r: (value.r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (value.g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (value.b.clamp(0.0, 1.0) * 255.0) as u8,
            a: (value.a.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }
}

impl From<ColorBgra32> for ColorFloat {
    fn from(value: ColorBgra32) -> Self {
        ColorFloat::new(
            value.r as f32 / 255.0,
            value.g as f32 / 255.0,
            value.b as f32 / 255.0,
            value.a as f32 / 255.0,
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorHSV {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl From<ColorFloat> for ColorHSV {
    fn from(mut value: ColorFloat) -> Self {
        value.r = value.r.clamp(0.0, 1.0);
        value.g = value.g.clamp(0.0, 1.0);
        value.b = value.b.clamp(0.0, 1.0);

        let min = value.r.min(value.g.min(value.b));
        let max = value.r.max(value.g.max(value.b));
        let v = max;
        let delta = max - min;

        if max == 0.0 {
            return Self { h: 0.0, s: 0.0, v };
        }

        let s = if max != 0.0 { delta / max } else { 0.0 };

        let h = if delta != 0.0 {
            let mut h = if value.r == max {
                (value.g - value.b) / delta
            } else if value.g == max {
                2.0 + (value.b - value.r) / delta
            } else {
                4.0 + (value.r - value.g) / delta
            };
            h *= 60.0;
            if h < 0.0 {
                h += 360.0;
            }
            h
        } else {
            0.0
        };

        Self { h, s, v }
    }
}

impl From<ColorHSV> for ColorFloat {
    fn from(value: ColorHSV) -> Self {
        let mut h = value.h;
        let s = value.s;
        let v = value.v;

        if s == 0.0 {
            return ColorFloat::new(v, v, v, 1.0);
        }

        h /= 60.0;
        let i = h.floor() as i32;
        let f = h - i as f32;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        match i.rem_euclid(6) {
            0 => ColorFloat::new(v, t, p, 1.0),
            1 => ColorFloat::new(q, v, p, 1.0),
            2 => ColorFloat::new(p, v, t, 1.0),
            3 => ColorFloat::new(p, q, v, 1.0),
            4 => ColorFloat::new(t, p, v, 1.0),
            _ => ColorFloat::new(v, p, q, 1.0),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorHLS {
    pub h: f32,
    pub l: f32,
    pub s: f32,
}

impl From<ColorFloat> for ColorHLS {
    fn from(mut value: ColorFloat) -> Self {
        value.r = value.r.clamp(0.0, 1.0);
        value.g = value.g.clamp(0.0, 1.0);
        value.b = value.b.clamp(0.0, 1.0);

        let min = value.r.min(value.g.min(value.b));
        let max = value.r.max(value.g.max(value.b));
        let delta = max - min;

        let l = (max + min) / 2.0;

        if delta == 0.0 {
            return Self { h: 0.0, l, s: 0.0 };
        }

        let s = if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let mut h = if value.r == max {
            (value.g - value.b) / delta
        } else if value.g == max {
            2.0 + (value.b - value.r) / delta
        } else {
            4.0 + (value.r - value.g) / delta
        };

        h *= 60.0;
        if h < 0.0 {
            h += 360.0;
        }

        Self { h, l, s }
    }
}

impl From<ColorHLS> for ColorFloat {
    fn from(value: ColorHLS) -> Self {
        let h = value.h;
        let l = value.l;
        let s = value.s;

        if s == 0.0 {
            return ColorFloat::new(l, l, l, 1.0);
        }

        let temp2 = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let temp1 = 2.0 * l - temp2;

        let r = color_component(temp1, temp2, h + 120.0);
        let g = color_component(temp1, temp2, h);
        let b = color_component(temp1, temp2, h - 120.0);

        ColorFloat::new(r, g, b, 1.0)
    }
}

fn color_component(temp1: f32, temp2: f32, mut temp3: f32) -> f32 {
    temp3 = normalize_hue(temp3);
    if temp3 < 60.0 {
        temp1 + (temp2 - temp1) * temp3 / 60.0
    } else if temp3 < 180.0 {
        temp2
    } else if temp3 < 240.0 {
        temp1 + (temp2 - temp1) * (240.0 - temp3) / 60.0
    } else {
        temp1
    }
}

fn normalize_hue(mut hue: f32) -> f32 {
    hue %= 360.0;
    if hue < 0.0 {
        hue += 360.0;
    }
    hue
}

impl From<Vector3<f32>> for Vector3f {
    fn from(v: Vector3<f32>) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vector3f> for Vector3<f32> {
    fn from(v: Vector3f) -> Self {
        Vector3::new(v.x, v.y, v.z)
    }
}

impl From<Vector2<f32>> for Vector2f {
    fn from(v: Vector2<f32>) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Vector2f> for Vector2<f32> {
    fn from(v: Vector2f) -> Self {
        Vector2::new(v.x, v.y)
    }
}

/// Row-major 4x4 matrix compatible with the native viewer API
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4x4 {
    pub m11: f32,
    pub m12: f32,
    pub m13: f32,
    pub m14: f32,
    pub m21: f32,
    pub m22: f32,
    pub m23: f32,
    pub m24: f32,
    pub m31: f32,
    pub m32: f32,
    pub m33: f32,
    pub m34: f32,
    pub m41: f32,
    pub m42: f32,
    pub m43: f32,
    pub m44: f32,
}

impl Matrix4x4 {
    pub fn identity() -> Self {
        Self {
            m11: 1.0,
            m12: 0.0,
            m13: 0.0,
            m14: 0.0,
            m21: 0.0,
            m22: 1.0,
            m23: 0.0,
            m24: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
            m34: 0.0,
            m41: 0.0,
            m42: 0.0,
            m43: 0.0,
            m44: 1.0,
        }
    }

    pub fn multiply(&self, other: &Matrix4x4) -> Matrix4x4 {
        let a = [
            [self.m11, self.m12, self.m13, self.m14],
            [self.m21, self.m22, self.m23, self.m24],
            [self.m31, self.m32, self.m33, self.m34],
            [self.m41, self.m42, self.m43, self.m44],
        ];
        let b = [
            [other.m11, other.m12, other.m13, other.m14],
            [other.m21, other.m22, other.m23, other.m24],
            [other.m31, other.m32, other.m33, other.m34],
            [other.m41, other.m42, other.m43, other.m44],
        ];

        let mut c = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                c[i][j] =
                    a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
            }
        }

        Matrix4x4 {
            m11: c[0][0],
            m12: c[0][1],
            m13: c[0][2],
            m14: c[0][3],
            m21: c[1][0],
            m22: c[1][1],
            m23: c[1][2],
            m24: c[1][3],
            m31: c[2][0],
            m32: c[2][1],
            m33: c[2][2],
            m34: c[2][3],
            m41: c[3][0],
            m42: c[3][1],
            m43: c[3][2],
            m44: c[3][3],
        }
    }
}

impl Default for Matrix4x4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl From<nalgebra::Matrix4<f32>> for Matrix4x4 {
    fn from(value: nalgebra::Matrix4<f32>) -> Self {
        Self {
            m11: value[(0, 0)],
            m12: value[(0, 1)],
            m13: value[(0, 2)],
            m14: value[(0, 3)],
            m21: value[(1, 0)],
            m22: value[(1, 1)],
            m23: value[(1, 2)],
            m24: value[(1, 3)],
            m31: value[(2, 0)],
            m32: value[(2, 1)],
            m33: value[(2, 2)],
            m34: value[(2, 3)],
            m41: value[(3, 0)],
            m42: value[(3, 1)],
            m43: value[(3, 2)],
            m44: value[(3, 3)],
        }
    }
}

impl From<Matrix4x4> for nalgebra::Matrix4<f32> {
    fn from(value: Matrix4x4) -> Self {
        nalgebra::Matrix4::new(
            value.m11, value.m12, value.m13, value.m14, value.m21, value.m22, value.m23, value.m24,
            value.m31, value.m32, value.m33, value.m34, value.m41, value.m42, value.m43, value.m44,
        )
    }
}

/// 2D Bounding Box
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox2 {
    pub min: Vector2<f32>,
    pub max: Vector2<f32>,
}

impl BBox2 {
    pub fn new(min: Vector2<f32>, max: Vector2<f32>) -> Self {
        debug_assert!(min.x <= max.x);
        debug_assert!(min.y <= max.y);
        Self { min, max }
    }

    pub fn empty() -> Self {
        Self {
            min: Vector2::new(f32::MAX, f32::MAX),
            max: Vector2::new(f32::MIN, f32::MIN),
        }
    }

    pub fn size(&self) -> Vector2<f32> {
        self.max - self.min
    }

    pub fn center(&self) -> Vector2<f32> {
        (self.min + self.max) * 0.5
    }

    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y
    }

    pub fn contains(&self, point: Vector2<f32>) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    pub fn include_point(&mut self, point: Vector2<f32>) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
    }

    /// C#-style alias for including a point.
    pub fn include(&mut self, point: Vector2<f32>) {
        self.include_point(point);
    }

    pub fn include_bbox(&mut self, other: &BBox2) {
        if other.is_empty() {
            return;
        }
        self.include_point(other.min);
        self.include_point(other.max);
    }

    pub fn grow(&mut self, amount: f32) {
        if self.is_empty() {
            self.min = Vector2::new(-amount, -amount);
            self.max = Vector2::new(amount, amount);
            return;
        }
        self.min -= Vector2::new(amount, amount);
        self.max += Vector2::new(amount, amount);
    }

    pub fn fit_into(&self, bounds: &BBox2) -> Option<(BBox2, f32, Vector2<f32>)> {
        if self.is_empty() || bounds.is_empty() {
            return None;
        }
        let size = self.size();
        if size.x <= 0.0 || size.y <= 0.0 {
            return None;
        }
        let bounds_size = bounds.size();
        let scale_x = bounds_size.x / size.x;
        let scale_y = bounds_size.y / size.y;
        let scale = scale_x.min(scale_y);

        let mut new_min = self.min * scale;
        let mut new_max = self.max * scale;

        let new_bbox = BBox2::new(new_min, new_max);
        let offset = bounds.center() - new_bbox.center();
        new_min += offset;
        new_max += offset;

        Some((BBox2::new(new_min, new_max), scale, offset))
    }

    pub fn random_point_inside<F>(&self, mut next_f32: F) -> Vector2<f32>
    where
        F: FnMut() -> f32,
    {
        if self.is_empty() {
            return Vector2::zeros();
        }
        Vector2::new(
            self.min.x + next_f32() * (self.max.x - self.min.x),
            self.min.y + next_f32() * (self.max.y - self.min.y),
        )
    }
}

/// 3D Bounding Box
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BBox3 {
    /// Minimum corner (for FFI compatibility)
    pub(crate) min_ffi: Vector3f,
    /// Maximum corner (for FFI compatibility)
    pub(crate) max_ffi: Vector3f,
}

impl BBox3 {
    /// Create a new bounding box
    pub fn new(min: Vector3<f32>, max: Vector3<f32>) -> Self {
        debug_assert!(min.x <= max.x);
        debug_assert!(min.y <= max.y);
        debug_assert!(min.z <= max.z);
        Self {
            min_ffi: Vector3f::from(min),
            max_ffi: Vector3f::from(max),
        }
    }

    /// Get minimum corner
    pub fn min(&self) -> Vector3<f32> {
        Vector3::from(self.min_ffi)
    }

    /// Get maximum corner
    pub fn max(&self) -> Vector3<f32> {
        Vector3::from(self.max_ffi)
    }

    /// Create an empty bounding box
    pub fn empty() -> Self {
        Self {
            min_ffi: Vector3f {
                x: f32::MAX,
                y: f32::MAX,
                z: f32::MAX,
            },
            max_ffi: Vector3f {
                x: f32::MIN,
                y: f32::MIN,
                z: f32::MIN,
            },
        }
    }

    /// Create a bounding box from center and size
    pub fn from_center_size(center: Vector3<f32>, size: Vector3<f32>) -> Self {
        let half_size = size * 0.5;
        Self::new(center - half_size, center + half_size)
    }

    /// Get the size of the bounding box
    pub fn size(&self) -> Vector3<f32> {
        self.max() - self.min()
    }

    /// Get the center of the bounding box
    pub fn center(&self) -> Vector3<f32> {
        (self.min() + self.max()) * 0.5
    }

    /// Get the volume of the bounding box
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }

    /// Check if the bounding box is empty
    pub fn is_empty(&self) -> bool {
        self.min_ffi.x > self.max_ffi.x
            || self.min_ffi.y > self.max_ffi.y
            || self.min_ffi.z > self.max_ffi.z
    }

    /// Check if a point is inside the bounding box
    pub fn contains(&self, point: Vector3<f32>) -> bool {
        point.x >= self.min_ffi.x
            && point.x <= self.max_ffi.x
            && point.y >= self.min_ffi.y
            && point.y <= self.max_ffi.y
            && point.z >= self.min_ffi.z
            && point.z <= self.max_ffi.z
    }

    /// Expand the bounding box to include a point
    pub fn include_point(&mut self, point: Vector3<f32>) {
        self.min_ffi.x = self.min_ffi.x.min(point.x);
        self.min_ffi.y = self.min_ffi.y.min(point.y);
        self.min_ffi.z = self.min_ffi.z.min(point.z);
        self.max_ffi.x = self.max_ffi.x.max(point.x);
        self.max_ffi.y = self.max_ffi.y.max(point.y);
        self.max_ffi.z = self.max_ffi.z.max(point.z);
    }

    /// C#-style alias for including a point.
    pub fn include(&mut self, point: Vector3<f32>) {
        self.include_point(point);
    }

    /// Expand the bounding box to include another bounding box
    pub fn include_bbox(&mut self, other: &BBox3) {
        self.include_point(other.min());
        self.include_point(other.max());
    }

    /// Include a 2D bounding box at the specified Z coordinate
    pub fn include_bbox2(&mut self, other: &BBox2, z: f32) {
        if other.is_empty() {
            return;
        }
        self.include_point(Vector3::new(other.min.x, other.min.y, z));
        self.include_point(Vector3::new(other.max.x, other.max.y, z));
    }

    /// Grow the bounding box by the specified amount on each side
    pub fn grow(&mut self, amount: f32) {
        if self.is_empty() {
            let min = Vector3::new(-amount, -amount, -amount);
            let max = Vector3::new(amount, amount, amount);
            *self = BBox3::new(min, max);
            return;
        }
        let min = self.min() - Vector3::new(amount, amount, amount);
        let max = self.max() + Vector3::new(amount, amount, amount);
        *self = BBox3::new(min, max);
    }

    /// Fit this bounding box into another bounding box
    pub fn fit_into(&self, bounds: &BBox3) -> Option<(BBox3, f32, Vector3<f32>)> {
        if self.is_empty() || bounds.is_empty() {
            return None;
        }
        let size = self.size();
        if size.x <= 0.0 || size.y <= 0.0 || size.z <= 0.0 {
            return None;
        }
        let bounds_size = bounds.size();
        let scale_x = bounds_size.x / size.x;
        let scale_y = bounds_size.y / size.y;
        let scale_z = bounds_size.z / size.z;
        let scale = scale_x.min(scale_y).min(scale_z);

        let mut new_min = self.min() * scale;
        let mut new_max = self.max() * scale;
        let new_bbox = BBox3::new(new_min, new_max);
        let offset = bounds.center() - new_bbox.center();
        new_min += offset;
        new_max += offset;

        Some((BBox3::new(new_min, new_max), scale, offset))
    }

    /// Return a random point inside the bounding box
    pub fn random_point_inside<F>(&self, mut next_f32: F) -> Vector3<f32>
    where
        F: FnMut() -> f32,
    {
        if self.is_empty() {
            return Vector3::zeros();
        }
        Vector3::new(
            self.min_ffi.x + next_f32() * (self.max_ffi.x - self.min_ffi.x),
            self.min_ffi.y + next_f32() * (self.max_ffi.y - self.min_ffi.y),
            self.min_ffi.z + next_f32() * (self.max_ffi.z - self.min_ffi.z),
        )
    }

    /// C#-style alias for `random_point_inside`.
    pub fn random_vector_inside<F>(&self, next_f32: F) -> Vector3<f32>
    where
        F: FnMut() -> f32,
    {
        self.random_point_inside(next_f32)
    }

    /// Return the 2D bounding box of the XY extent
    pub fn as_bbox2(&self) -> BBox2 {
        BBox2::new(
            Vector2::new(self.min_ffi.x, self.min_ffi.y),
            Vector2::new(self.max_ffi.x, self.max_ffi.y),
        )
    }

    /// C#-style alias for `as_bbox2`.
    pub fn as_bounding_box2(&self) -> BBox2 {
        self.as_bbox2()
    }
}

impl fmt::Display for BBox3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<Min: <{:.2}, {:.2}, {:.2}> | Max: <{:.2}, {:.2}, {:.2}>>",
            self.min_ffi.x,
            self.min_ffi.y,
            self.min_ffi.z,
            self.max_ffi.x,
            self.max_ffi.y,
            self.max_ffi.z
        )
    }
}

/// Triangle defined by three vertex indices
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Triangle {
    /// First vertex index
    pub v0: i32,
    /// Second vertex index
    pub v1: i32,
    /// Third vertex index
    pub v2: i32,
}

/// Voxel grid dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelDimensions {
    /// Origin in voxel coordinates
    pub origin: Vector3<i32>,
    /// Size in voxels
    pub size: Vector3<i32>,
}

impl VoxelDimensions {
    pub fn new(origin: Vector3<i32>, size: Vector3<i32>) -> Self {
        Self { origin, size }
    }
}

impl Triangle {
    /// Create a new triangle
    pub fn new(v0: i32, v1: i32, v2: i32) -> Self {
        Self { v0, v1, v2 }
    }

    /// Get vertex indices as an array
    pub fn indices(&self) -> [i32; 3] {
        [self.v0, self.v1, self.v2]
    }
}

impl fmt::Display for Triangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Triangle({}, {}, {})", self.v0, self.v1, self.v2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox_creation() {
        let bbox = BBox3::new(Vector3::zeros(), Vector3::new(10.0, 10.0, 10.0));
        assert_eq!(bbox.size(), Vector3::new(10.0, 10.0, 10.0));
        assert_eq!(bbox.center(), Vector3::new(5.0, 5.0, 5.0));
    }

    #[test]
    fn test_bbox_contains() {
        let bbox = BBox3::new(Vector3::zeros(), Vector3::new(10.0, 10.0, 10.0));
        assert!(bbox.contains(Vector3::new(5.0, 5.0, 5.0)));
        assert!(!bbox.contains(Vector3::new(15.0, 5.0, 5.0)));
    }

    #[test]
    fn test_triangle() {
        let tri = Triangle::new(0, 1, 2);
        assert_eq!(tri.indices(), [0, 1, 2]);
    }
}
