//! Image types

use crate::{ColorBgr24, ColorBgra32, ColorFloat, ColorRgb24, ColorRgba32};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageType {
    BW,
    Gray,
    Color,
}

pub trait Image {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn image_type(&self) -> ImageType;
    fn color_value(&self, x: usize, y: usize) -> ColorFloat;
    fn gray_value(&self, x: usize, y: usize) -> f32;

    /// C#-style alias for `color_value`.
    fn clr_value(&self, x: usize, y: usize) -> ColorFloat {
        self.color_value(x, y)
    }

    /// C#-style alias for `gray_value`.
    fn f_value(&self, x: usize, y: usize) -> f32 {
        self.gray_value(x, y)
    }

    fn bool_value(&self, x: usize, y: usize) -> bool {
        self.gray_value(x, y) > 0.5
    }

    /// C#-style alias for `bool_value`.
    fn b_value(&self, x: usize, y: usize) -> bool {
        self.bool_value(x, y)
    }

    fn set_color(&mut self, x: usize, y: usize, color: ColorFloat);
    fn set_gray(&mut self, x: usize, y: usize, gray: f32);

    /// C#-style alias for setting a color value.
    fn set_value(&mut self, x: usize, y: usize, color: ColorFloat) {
        self.set_color(x, y, color);
    }

    fn set_bool(&mut self, x: usize, y: usize, value: bool) {
        self.set_gray(x, y, if value { 1.0 } else { 0.0 });
    }

    fn byte_value(&self, x: usize, y: usize) -> u8 {
        (self.gray_value(x, y).clamp(0.0, 1.0) * 255.0) as u8
    }

    /// C#-style alias for `byte_value`.
    fn by_get_value(&self, x: usize, y: usize) -> u8 {
        self.byte_value(x, y)
    }

    fn set_byte(&mut self, x: usize, y: usize, value: u8) {
        self.set_gray(x, y, value as f32 / 255.0);
    }

    fn bgr24_value(&self, x: usize, y: usize) -> ColorBgr24 {
        ColorBgr24::from(self.color_value(x, y))
    }

    /// C#-style alias for `bgr24_value`.
    fn s_get_bgr24(&self, x: usize, y: usize) -> ColorBgr24 {
        self.bgr24_value(x, y)
    }

    fn bgra32_value(&self, x: usize, y: usize) -> ColorBgra32 {
        ColorBgra32::from(self.color_value(x, y))
    }

    /// C#-style alias for `bgra32_value`.
    fn s_get_bgra32(&self, x: usize, y: usize) -> ColorBgra32 {
        self.bgra32_value(x, y)
    }

    fn rgb24_value(&self, x: usize, y: usize) -> ColorRgb24 {
        ColorRgb24::from(self.color_value(x, y))
    }

    /// C#-style alias for `rgb24_value`.
    fn s_get_rgb24(&self, x: usize, y: usize) -> ColorRgb24 {
        self.rgb24_value(x, y)
    }

    fn rgba32_value(&self, x: usize, y: usize) -> ColorRgba32 {
        ColorRgba32::from(self.color_value(x, y))
    }

    /// C#-style alias for `rgba32_value`.
    fn s_get_rgba32(&self, x: usize, y: usize) -> ColorRgba32 {
        self.rgba32_value(x, y)
    }

    fn set_bgr24(&mut self, x: usize, y: usize, color: ColorBgr24) {
        self.set_color(x, y, ColorFloat::from(color));
    }

    fn set_bgra32(&mut self, x: usize, y: usize, color: ColorBgra32) {
        self.set_color(x, y, ColorFloat::from(color));
    }

    fn set_rgb24(&mut self, x: usize, y: usize, color: ColorRgb24) {
        self.set_color(x, y, ColorFloat::from(color));
    }

    fn set_rgba32(&mut self, x: usize, y: usize, color: ColorRgba32) {
        self.set_color(x, y, ColorFloat::from(color));
    }

    /// Get interpolated color at normalized coordinates (0..1).
    fn color_at_normalized(&self, tx: f32, ty: f32) -> ColorFloat {
        if self.width() == 0 || self.height() == 0 {
            return ColorFloat::new(0.0, 0.0, 0.0, 1.0);
        }

        let width = self.width() as f32;
        let height = self.height() as f32;

        let real_x = tx * width - 1.0;
        let real_y = ty * height - 1.0;

        let mut x0 = real_x.floor() as i32;
        let mut x1 = x0 + 1;
        let mut y0 = real_y.floor() as i32;
        let mut y1 = y0 + 1;

        let max_x = self.width() as i32 - 1;
        let max_y = self.height() as i32 - 1;

        x0 = x0.clamp(0, max_x);
        x1 = x1.clamp(0, max_x);
        y0 = y0.clamp(0, max_y);
        y1 = y1.clamp(0, max_y);

        let dx = real_x - x0 as f32;
        let dy = real_y - y0 as f32;

        let c00 = self.color_value(x0 as usize, y0 as usize);
        let c10 = self.color_value(x1 as usize, y0 as usize);
        let c01 = self.color_value(x0 as usize, y1 as usize);
        let c11 = self.color_value(x1 as usize, y1 as usize);

        let c0 = ColorFloat::weighted(c00, c10, dx);
        let c1 = ColorFloat::weighted(c01, c11, dx);

        ColorFloat::weighted(c0, c1, dy)
    }

    /// C#-style alias for `color_at_normalized`.
    fn clr_get_at_normalized(&self, tx: f32, ty: f32) -> ColorFloat {
        self.color_at_normalized(tx, ty)
    }

    /// C#-style alias for `color_at_normalized`.
    fn get_at_normalized(&self, tx: f32, ty: f32) -> ColorFloat {
        self.color_at_normalized(tx, ty)
    }

    fn draw_line_color(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: ColorFloat) {
        draw_line_internal(x0, y0, x1, y1, |x, y| {
            if let Some((ux, uy)) = clamp_index(self.width(), self.height(), x, y) {
                self.set_color(ux, uy, color);
            }
        });
    }

    /// C#-style alias for `draw_line_color`.
    fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: ColorFloat) {
        self.draw_line_color(x0, y0, x1, y1, color);
    }

    fn draw_line_gray(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, gray: f32) {
        draw_line_internal(x0, y0, x1, y1, |x, y| {
            if let Some((ux, uy)) = clamp_index(self.width(), self.height(), x, y) {
                self.set_gray(ux, uy, gray);
            }
        });
    }

    fn draw_line_bool(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, value: bool) {
        draw_line_internal(x0, y0, x1, y1, |x, y| {
            if let Some((ux, uy)) = clamp_index(self.width(), self.height(), x, y) {
                self.set_bool(ux, uy, value);
            }
        });
    }
}

fn clamp_index(width: usize, height: usize, x: i32, y: i32) -> Option<(usize, usize)> {
    if x < 0 || y < 0 {
        return None;
    }
    let ux = x as usize;
    let uy = y as usize;
    if ux >= width || uy >= height {
        return None;
    }
    Some((ux, uy))
}

fn draw_line_internal(mut x0: i32, mut y0: i32, x1: i32, y1: i32, mut plot: impl FnMut(i32, i32)) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        plot(x0, y0);

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageBW {
    width: usize,
    height: usize,
    pub values: Vec<u8>,
}

impl ImageBW {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0; width * height],
        }
    }

    pub fn set_value(&mut self, x: usize, y: usize, value: bool) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.values[x + y * self.width] = if value { 1 } else { 0 };
    }

    pub fn value(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.values[x + y * self.width] != 0
    }
}

impl Image for ImageBW {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn image_type(&self) -> ImageType {
        ImageType::BW
    }

    fn color_value(&self, x: usize, y: usize) -> ColorFloat {
        let value = if self.value(x, y) { 1.0 } else { 0.0 };
        ColorFloat::gray(value, 1.0)
    }

    fn gray_value(&self, x: usize, y: usize) -> f32 {
        if self.value(x, y) {
            1.0
        } else {
            0.0
        }
    }

    fn set_color(&mut self, x: usize, y: usize, color: ColorFloat) {
        let gray = (color.r + color.g + color.b) / 3.0;
        self.set_value(x, y, gray > 0.5);
    }

    fn set_gray(&mut self, x: usize, y: usize, gray: f32) {
        self.set_value(x, y, gray > 0.5);
    }
}

#[derive(Debug, Clone)]
pub struct ImageGrayScale {
    width: usize,
    height: usize,
    pub values: Vec<f32>,
}

impl ImageGrayScale {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0.0; width * height],
        }
    }

    pub fn set_value(&mut self, x: usize, y: usize, value: f32) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.values[x + y * self.width] = value;
    }

    pub fn value(&self, x: usize, y: usize) -> f32 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        self.values[x + y * self.width]
    }

    pub fn contains_active_pixels(&self, threshold: f32) -> bool {
        self.values.iter().any(|value| *value <= threshold)
    }

    pub fn interpolated(
        a: &ImageGrayScale,
        b: &ImageGrayScale,
        weight: f32,
    ) -> crate::Result<Self> {
        if a.width != b.width || a.height != b.height {
            return Err(crate::Error::InvalidParameter(
                "Interpolation between images requires same width and height".to_string(),
            ));
        }
        if weight <= 0.0 {
            return Ok(a.clone());
        }
        if weight >= 1.0 {
            return Ok(b.clone());
        }

        let inv = 1.0 - weight;
        let mut values = vec![0.0; a.values.len()];
        for ((out, &va), &vb) in values.iter_mut().zip(a.values.iter()).zip(b.values.iter()) {
            *out = va * inv + vb * weight;
        }
        Ok(Self {
            width: a.width,
            height: a.height,
            values,
        })
    }

    /// C#-style alias for `interpolated`.
    pub fn get_interpolated(
        a: &ImageGrayScale,
        b: &ImageGrayScale,
        weight: f32,
    ) -> crate::Result<Self> {
        Self::interpolated(a, b, weight)
    }

    /// C#-style alias for `interpolated`.
    pub fn img_get_interpolated(
        a: &ImageGrayScale,
        b: &ImageGrayScale,
        weight: f32,
    ) -> crate::Result<Self> {
        Self::interpolated(a, b, weight)
    }

    pub fn color_coded_sdf(&self, background: f32) -> ImageColor {
        let mut img = ImageColor::new(self.width, self.height);
        let inside_background =
            ColorFloat::from_hex("006600").unwrap_or(ColorFloat::new(0.0, 0.4, 0.0, 1.0));
        let outside_background =
            ColorFloat::from_hex("00").unwrap_or(ColorFloat::new(0.0, 0.0, 0.0, 1.0));

        for x in 0..self.width {
            for y in 0..self.height {
                let mut value = self.value(x, y);
                if value < 0.0 {
                    if value <= -background {
                        img.set_value(x, y, inside_background);
                    } else {
                        value /= -background;
                        img.set_value(x, y, ColorFloat::new(0.0, 1.0 - value, 0.0, 1.0));
                    }
                } else if value >= background {
                    img.set_value(x, y, outside_background);
                } else {
                    value /= background;
                    img.set_value(
                        x,
                        y,
                        ColorFloat::new(1.0 - value, 1.0 - value, 1.0 - value, 1.0),
                    );
                }
            }
        }

        img
    }

    /// C#-style alias for `color_coded_sdf`.
    pub fn get_color_coded_sdf(&self, background: f32) -> ImageColor {
        self.color_coded_sdf(background)
    }

    /// C#-style alias for `color_coded_sdf`.
    pub fn img_get_color_coded_sdf(&self, background: f32) -> ImageColor {
        self.color_coded_sdf(background)
    }
}

impl Image for ImageGrayScale {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn image_type(&self) -> ImageType {
        ImageType::Gray
    }

    fn color_value(&self, x: usize, y: usize) -> ColorFloat {
        let value = self.value(x, y);
        ColorFloat::gray(value, 1.0)
    }

    fn gray_value(&self, x: usize, y: usize) -> f32 {
        self.value(x, y)
    }

    fn set_color(&mut self, x: usize, y: usize, color: ColorFloat) {
        let gray = (color.r + color.g + color.b) / 3.0;
        self.set_value(x, y, gray);
    }

    fn set_gray(&mut self, x: usize, y: usize, gray: f32) {
        self.set_value(x, y, gray);
    }
}

#[derive(Debug, Clone)]
pub struct ImageColor {
    width: usize,
    height: usize,
    pub values: Vec<ColorFloat>,
}

impl ImageColor {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![ColorFloat::new(0.0, 0.0, 0.0, 1.0); width * height],
        }
    }

    pub fn from_image(img: &dyn Image) -> Self {
        let mut result = Self::new(img.width(), img.height());
        for x in 0..img.width() {
            for y in 0..img.height() {
                result.set_value(x, y, img.color_value(x, y));
            }
        }
        result
    }

    pub fn set_value<C: Into<ColorFloat>>(&mut self, x: usize, y: usize, color: C) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.values[x + y * self.width] = color.into();
    }

    pub fn value(&self, x: usize, y: usize) -> ColorFloat {
        if x >= self.width || y >= self.height {
            return ColorFloat::new(0.0, 0.0, 0.0, 1.0);
        }
        self.values[x + y * self.width]
    }
}

impl Image for ImageColor {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn image_type(&self) -> ImageType {
        ImageType::Color
    }

    fn color_value(&self, x: usize, y: usize) -> ColorFloat {
        self.value(x, y)
    }

    fn gray_value(&self, x: usize, y: usize) -> f32 {
        let color = self.value(x, y);
        (color.r + color.g + color.b) / 3.0
    }

    fn set_color(&mut self, x: usize, y: usize, color: ColorFloat) {
        self.set_value(x, y, color);
    }

    fn set_gray(&mut self, x: usize, y: usize, gray: f32) {
        self.set_value(x, y, ColorFloat::gray(gray, 1.0));
    }
}

#[derive(Debug, Clone)]
pub struct ImageRgb24 {
    width: usize,
    height: usize,
    pub values: Vec<ColorRgb24>,
}

impl ImageRgb24 {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![ColorRgb24 { r: 0, g: 0, b: 0 }; width * height],
        }
    }

    pub fn from_image(img: &dyn Image) -> Self {
        let mut result = Self::new(img.width(), img.height());
        for x in 0..img.width() {
            for y in 0..img.height() {
                result.set_rgb24(x, y, img.rgb24_value(x, y));
            }
        }
        result
    }

    pub fn set_rgb24(&mut self, x: usize, y: usize, color: ColorRgb24) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.values[x + y * self.width] = color;
    }

    pub fn rgb24(&self, x: usize, y: usize) -> ColorRgb24 {
        if x >= self.width || y >= self.height {
            return ColorRgb24 { r: 0, g: 0, b: 0 };
        }
        self.values[x + y * self.width]
    }
}

impl Image for ImageRgb24 {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn image_type(&self) -> ImageType {
        ImageType::Color
    }

    fn color_value(&self, x: usize, y: usize) -> ColorFloat {
        ColorFloat::from(self.rgb24(x, y))
    }

    fn gray_value(&self, x: usize, y: usize) -> f32 {
        let color = self.rgb24(x, y);
        (color.r as f32 + color.g as f32 + color.b as f32) / (3.0 * 255.0)
    }

    fn set_color(&mut self, x: usize, y: usize, color: ColorFloat) {
        self.set_rgb24(x, y, ColorRgb24::from(color));
    }

    fn set_gray(&mut self, x: usize, y: usize, gray: f32) {
        self.set_color(x, y, ColorFloat::gray(gray, 1.0));
    }
}

#[derive(Debug, Clone)]
pub struct ImageRgba32 {
    width: usize,
    height: usize,
    pub values: Vec<ColorRgba32>,
}

impl ImageRgba32 {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![
                ColorRgba32 {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0
                };
                width * height
            ],
        }
    }

    pub fn from_image(img: &dyn Image) -> Self {
        let mut result = Self::new(img.width(), img.height());
        for x in 0..img.width() {
            for y in 0..img.height() {
                result.set_rgba32(x, y, img.rgba32_value(x, y));
            }
        }
        result
    }

    pub fn set_rgba32(&mut self, x: usize, y: usize, color: ColorRgba32) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.values[x + y * self.width] = color;
    }

    pub fn rgba32(&self, x: usize, y: usize) -> ColorRgba32 {
        if x >= self.width || y >= self.height {
            return ColorRgba32 {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            };
        }
        self.values[x + y * self.width]
    }
}

impl Image for ImageRgba32 {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn image_type(&self) -> ImageType {
        ImageType::Color
    }

    fn color_value(&self, x: usize, y: usize) -> ColorFloat {
        ColorFloat::from(self.rgba32(x, y))
    }

    fn gray_value(&self, x: usize, y: usize) -> f32 {
        let color = self.rgba32(x, y);
        (color.r as f32 + color.g as f32 + color.b as f32) / (3.0 * 255.0)
    }

    fn set_color(&mut self, x: usize, y: usize, color: ColorFloat) {
        self.set_rgba32(x, y, ColorRgba32::from(color));
    }

    fn set_gray(&mut self, x: usize, y: usize, gray: f32) {
        self.set_color(x, y, ColorFloat::gray(gray, 1.0));
    }
}

#[derive(Debug, Clone)]
pub enum ImageData {
    BW(ImageBW),
    Gray(ImageGrayScale),
    Color(ImageColor),
    Rgb24(ImageRgb24),
    Rgba32(ImageRgba32),
}

impl Image for ImageData {
    fn width(&self) -> usize {
        match self {
            ImageData::BW(img) => img.width(),
            ImageData::Gray(img) => img.width(),
            ImageData::Color(img) => img.width(),
            ImageData::Rgb24(img) => img.width(),
            ImageData::Rgba32(img) => img.width(),
        }
    }

    fn height(&self) -> usize {
        match self {
            ImageData::BW(img) => img.height(),
            ImageData::Gray(img) => img.height(),
            ImageData::Color(img) => img.height(),
            ImageData::Rgb24(img) => img.height(),
            ImageData::Rgba32(img) => img.height(),
        }
    }

    fn image_type(&self) -> ImageType {
        match self {
            ImageData::BW(_) => ImageType::BW,
            ImageData::Gray(_) => ImageType::Gray,
            ImageData::Color(_) => ImageType::Color,
            ImageData::Rgb24(_) => ImageType::Color,
            ImageData::Rgba32(_) => ImageType::Color,
        }
    }

    fn color_value(&self, x: usize, y: usize) -> ColorFloat {
        match self {
            ImageData::BW(img) => img.color_value(x, y),
            ImageData::Gray(img) => img.color_value(x, y),
            ImageData::Color(img) => img.color_value(x, y),
            ImageData::Rgb24(img) => img.color_value(x, y),
            ImageData::Rgba32(img) => img.color_value(x, y),
        }
    }

    fn gray_value(&self, x: usize, y: usize) -> f32 {
        match self {
            ImageData::BW(img) => img.gray_value(x, y),
            ImageData::Gray(img) => img.gray_value(x, y),
            ImageData::Color(img) => img.gray_value(x, y),
            ImageData::Rgb24(img) => img.gray_value(x, y),
            ImageData::Rgba32(img) => img.gray_value(x, y),
        }
    }

    fn set_color(&mut self, x: usize, y: usize, color: ColorFloat) {
        match self {
            ImageData::BW(img) => img.set_color(x, y, color),
            ImageData::Gray(img) => img.set_color(x, y, color),
            ImageData::Color(img) => img.set_color(x, y, color),
            ImageData::Rgb24(img) => img.set_color(x, y, color),
            ImageData::Rgba32(img) => img.set_color(x, y, color),
        }
    }

    fn set_gray(&mut self, x: usize, y: usize, gray: f32) {
        match self {
            ImageData::BW(img) => img.set_gray(x, y, gray),
            ImageData::Gray(img) => img.set_gray(x, y, gray),
            ImageData::Color(img) => img.set_gray(x, y, gray),
            ImageData::Rgb24(img) => img.set_gray(x, y, gray),
            ImageData::Rgba32(img) => img.set_gray(x, y, gray),
        }
    }
}
