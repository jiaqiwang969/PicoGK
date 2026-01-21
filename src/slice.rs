//! Slice and contour utilities

use crate::{BBox2, BBox3, ColorFloat, Image, PolyLine, Result, Viewer};
use nalgebra::{Vector2, Vector3};
use std::collections::VecDeque;
use std::f32;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Winding {
    Unknown,
    Clockwise,
    CounterClockwise,
}

impl Winding {
    pub fn as_string(self) -> &'static str {
        match self {
            Winding::CounterClockwise => "[counter-clockwise]",
            Winding::Clockwise => "[clockwise]",
            Winding::Unknown => "[unknown/degenerate]",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PolyContour {
    vertices: Vec<Vector2<f32>>,
    winding: Winding,
    bbox: BBox2,
}

impl PolyContour {
    pub fn detect_winding(vertices: &[Vector2<f32>]) -> Winding {
        if vertices.len() < 3 {
            return Winding::Unknown;
        }

        let mut area = 0.0f32;
        for i in 0..vertices.len() {
            let j = (i + 1) % vertices.len();
            area += (vertices[j].x - vertices[i].x) * (vertices[j].y + vertices[i].y);
        }

        if area > 0.0 {
            Winding::Clockwise
        } else if area < 0.0 {
            Winding::CounterClockwise
        } else {
            Winding::Unknown
        }
    }

    /// C#-style alias for `detect_winding`.
    pub fn e_detect_winding(vertices: &[Vector2<f32>]) -> Winding {
        Self::detect_winding(vertices)
    }

    /// C#-style alias for `Winding::as_string`.
    pub fn winding_as_string(winding: Winding) -> &'static str {
        winding.as_string()
    }

    /// C#-style alias for `Winding::as_string`.
    pub fn str_winding_as_string(winding: Winding) -> String {
        winding.as_string().to_string()
    }

    pub fn new(vertices: Vec<Vector2<f32>>, winding: Winding) -> Result<Self> {
        if vertices.len() < 3 {
            return Err(crate::Error::InvalidParameter(
                "Polyline with less than 3 points makes no sense".to_string(),
            ));
        }

        let mut bbox = BBox2::empty();
        for vec in &vertices {
            bbox.include_point(*vec);
        }

        let resolved = if winding == Winding::Unknown {
            Self::detect_winding(&vertices)
        } else {
            winding
        };

        Ok(Self {
            vertices,
            winding: resolved,
            bbox,
        })
    }

    pub fn add_vertex(&mut self, vec: Vector2<f32>) {
        self.bbox.include_point(vec);
        self.vertices.push(vec);
    }

    pub fn detect_winding_in_place(&mut self) {
        self.winding = Self::detect_winding(&self.vertices);
    }

    pub fn winding(&self) -> Winding {
        self.winding
    }

    /// C#-style alias for `winding`.
    pub fn e_winding(&self) -> Winding {
        self.winding()
    }

    pub fn vertices(&self) -> &[Vector2<f32>] {
        &self.vertices
    }

    pub fn close(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        let first = match self.vertices.first().copied() {
            Some(v) => v,
            None => return,
        };
        let last = match self.vertices.last().copied() {
            Some(v) => v,
            None => return,
        };
        if (first - last).norm() > f32::EPSILON {
            self.vertices.push(first);
        }
    }

    pub fn as_svg_polyline(&self) -> String {
        let mut str_out = String::from("<polyline points='");
        for vec in &self.vertices {
            str_out.push_str(&format!(" {},{}", vec.x, vec.y));
        }

        if let Some(first) = self.vertices.first() {
            str_out.push_str(&format!(" {},{}", first.x, first.y));
        }

        str_out.push_str("' ");

        match self.winding {
            Winding::Clockwise => str_out.push_str("stroke='blue' fill='none'"),
            Winding::CounterClockwise => str_out.push_str("stroke='black' fill='none'"),
            Winding::Unknown => str_out.push_str("stroke='red' fill='none'"),
        }

        str_out.push_str(" stroke-width='0.1' />\n");
        str_out
    }

    pub fn as_svg_path(&self) -> String {
        let mut str_out = String::new();
        for vec in &self.vertices {
            if str_out.is_empty() {
                str_out.push_str(" M");
            } else {
                str_out.push_str(" L");
            }
            str_out.push_str(&format!("{},{}", vec.x, vec.y));
        }
        str_out.push_str(" Z");
        str_out
    }

    pub fn bbox(&self) -> BBox2 {
        self.bbox
    }

    /// C#-style alias for `bbox`.
    pub fn o_b_box(&self) -> BBox2 {
        self.bbox()
    }

    pub fn count(&self) -> usize {
        self.vertices.len()
    }

    pub fn vertex(&self, index: usize) -> Option<Vector2<f32>> {
        self.vertices.get(index).copied()
    }
}

#[derive(Debug, Clone)]
pub struct PolySlice {
    contours: Vec<PolyContour>,
    z_pos: f32,
    bbox: BBox2,
}

impl PolySlice {
    pub fn new(z_pos: f32) -> Self {
        Self {
            contours: Vec::new(),
            z_pos,
            bbox: BBox2::empty(),
        }
    }

    pub fn add_contour(&mut self, contour: PolyContour) {
        self.bbox.include_bbox(&contour.bbox());
        self.contours.push(contour);
    }

    pub fn is_empty(&self) -> bool {
        self.contours.is_empty()
    }

    pub fn close(&mut self) {
        for contour in &mut self.contours {
            contour.close();
        }
    }

    pub fn save_to_svg_file<P: AsRef<Path>>(
        &self,
        path: P,
        solid: bool,
        bbox_to_use: Option<BBox2>,
    ) -> Result<()> {
        let bbox_view = bbox_to_use.unwrap_or(self.bbox);
        let mut file = File::create(path)?;

        writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>")?;
        writeln!(
            file,
            "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">"
        )?;

        let size = bbox_view.size();
        writeln!(
            file,
            "<svg xmlns='http://www.w3.org/2000/svg' version='1.1' viewBox='{} {} {} {}' width='{}mm' height='{}mm'>",
            bbox_view.min.x,
            bbox_view.min.y,
            size.x,
            size.y,
            size.x,
            size.y
        )?;
        writeln!(file, "<g>")?;

        if !solid {
            for contour in &self.contours {
                file.write_all(contour.as_svg_polyline().as_bytes())?;
            }
        } else {
            let mut path_data = String::from("<path d='");
            for pass in 0..2 {
                for contour in &self.contours {
                    if pass == 0 {
                        if contour.winding() != Winding::CounterClockwise {
                            continue;
                        }
                    } else if contour.winding() == Winding::CounterClockwise {
                        continue;
                    }

                    path_data.push_str(&contour.as_svg_path());
                }
            }
            path_data.push_str("' fill='black'/> ");
            file.write_all(path_data.as_bytes())?;
        }

        writeln!(file, "</g>")?;
        writeln!(file, "</svg>")?;
        Ok(())
    }

    pub fn from_sdf(img: &dyn Image, z_pos: f32, offset: Vector2<f32>, scale: f32) -> Self {
        let mut slice = PolySlice::new(z_pos);
        if img.width() < 2 || img.height() < 2 {
            return slice;
        }

        let mut segments: Vec<Segment> = Vec::new();

        for y in 0..(img.height() - 1) {
            for x in 0..(img.width() - 1) {
                let corners = [
                    img.gray_value(x, y),
                    img.gray_value(x + 1, y),
                    img.gray_value(x + 1, y + 1),
                    img.gray_value(x, y + 1),
                ];

                let mut lut_index = 0;
                if corners[0] < 0.0 {
                    lut_index |= 1;
                }
                if corners[1] < 0.0 {
                    lut_index |= 2;
                }
                if corners[2] < 0.0 {
                    lut_index |= 4;
                }
                if corners[3] < 0.0 {
                    lut_index |= 8;
                }

                let edges_crossed = EDGE_LUT[lut_index][0];
                if edges_crossed == 0 {
                    continue;
                }

                let mut crossings = [Vector2::zeros(); 4];
                if (edges_crossed & 1) != 0 {
                    crossings[0] =
                        Vector2::new(x as f32 + zero_crossing(corners[0], corners[1]), y as f32);
                }
                if (edges_crossed & 2) != 0 {
                    crossings[1] = Vector2::new(
                        x as f32 + 1.0,
                        y as f32 + zero_crossing(corners[1], corners[2]),
                    );
                }
                if (edges_crossed & 4) != 0 {
                    crossings[2] = Vector2::new(
                        x as f32 + zero_crossing(corners[3], corners[2]),
                        y as f32 + 1.0,
                    );
                }
                if (edges_crossed & 8) != 0 {
                    crossings[3] =
                        Vector2::new(x as f32, y as f32 + zero_crossing(corners[0], corners[3]));
                }

                let mut seg_index = 1;
                while seg_index < 5 {
                    let start_idx = EDGE_LUT[lut_index][seg_index];
                    if start_idx < 0 {
                        break;
                    }
                    let end_idx = EDGE_LUT[lut_index][seg_index + 1];
                    let start = offset + crossings[start_idx as usize] * scale;
                    let end = offset + crossings[end_idx as usize] * scale;
                    segments.push(Segment::new(start, end));
                    seg_index += 2;
                }
            }
        }

        let mut segments_left = segments.len();
        let mut curr_start: Option<usize> = None;
        let mut curr_end: Option<usize> = None;
        let mut unused = 0usize;
        let mut contour = VecDeque::new();

        while segments_left > 0 {
            if curr_start.is_none() {
                let start_idx = match segments
                    .iter()
                    .enumerate()
                    .skip(unused)
                    .find(|(_, seg)| !seg.used)
                    .map(|(idx, _)| idx)
                {
                    Some(idx) => idx,
                    None => break,
                };

                curr_start = Some(start_idx);
                curr_end = Some(start_idx);
                unused = start_idx + 1;

                contour.push_back(segments[start_idx].start);
                contour.push_back(segments[start_idx].end);
                segments[start_idx].used = true;
                segments_left -= 1;
            }

            let (Some(curr_start_idx), Some(curr_end_idx)) = (curr_start, curr_end) else {
                break;
            };

            let mut best_start: Option<usize> = None;
            let mut best_end: Option<usize> = None;
            let mut best_sqr_start = 1.0f32;
            let mut best_sqr_end = 1.0f32;

            if curr_end_idx != curr_start_idx {
                let sqr =
                    (segments[curr_start_idx].start - segments[curr_end_idx].end).norm_squared();
                if sqr < 1.0 {
                    best_sqr_start = sqr;
                    best_sqr_end = sqr;
                    best_start = Some(curr_end_idx);
                    best_end = Some(curr_start_idx);
                }
            }

            let f_min = (segments[curr_start_idx]
                .min_y
                .min(segments[curr_end_idx].min_y))
            .floor()
                - 1.0;
            let f_max = (segments[curr_start_idx]
                .max_y
                .max(segments[curr_end_idx].max_y))
            .ceil()
                + 1.0;

            let mut search_from = curr_start_idx.min(curr_end_idx);
            while search_from > 0 {
                if segments[search_from - 1].max_y >= f_min {
                    search_from -= 1;
                } else {
                    break;
                }
            }

            for idx in search_from..segments.len() {
                if segments[idx].used {
                    continue;
                }
                if segments[idx].min_y > f_max {
                    break;
                }

                let sqr = (segments[curr_start_idx].start - segments[idx].end).norm_squared();
                if sqr < best_sqr_start {
                    best_sqr_start = sqr;
                    best_start = Some(idx);
                }

                let sqr = (segments[curr_end_idx].end - segments[idx].start).norm_squared();
                if sqr < best_sqr_end {
                    best_sqr_end = sqr;
                    best_end = Some(idx);
                }
            }

            if best_end.is_none() && best_start.is_none() {
                contour.clear();
                curr_start = None;
                curr_end = None;
            } else if best_start == best_end
                || best_start == Some(curr_end_idx)
                || best_end == Some(curr_start_idx)
            {
                if best_start == best_end {
                    if let Some(idx) = best_end {
                        segments[idx].used = true;
                        segments_left -= 1;
                    }
                }

                if contour.len() > 2 {
                    let vertices: Vec<Vector2<f32>> = contour.iter().copied().collect();
                    if let Ok(contour_obj) = PolyContour::new(vertices, Winding::Unknown) {
                        slice.add_contour(contour_obj);
                    }
                }

                contour.clear();
                curr_start = None;
                curr_end = None;
            } else {
                if let Some(idx) = best_end {
                    contour.push_back(segments[idx].end);
                    segments[idx].used = true;
                    segments_left -= 1;
                    curr_end = Some(idx);
                }
                if let Some(idx) = best_start {
                    contour.push_front(segments[idx].start);
                    segments[idx].used = true;
                    segments_left -= 1;
                    curr_start = Some(idx);
                }
            }
        }

        slice
    }

    pub fn z_pos(&self) -> f32 {
        self.z_pos
    }

    pub fn bbox(&self) -> BBox2 {
        self.bbox
    }

    /// C#-style alias for `bbox`.
    pub fn o_b_box(&self) -> BBox2 {
        self.bbox()
    }

    pub fn contours(&self) -> &[PolyContour] {
        &self.contours
    }

    /// C#-style alias for `contour_count`.
    pub fn n_contours(&self) -> usize {
        self.contour_count()
    }

    pub fn contour_count(&self) -> usize {
        self.contours.len()
    }

    pub fn contour_at(&self, index: usize) -> Option<&PolyContour> {
        self.contours.get(index)
    }
}

struct Segment {
    start: Vector2<f32>,
    end: Vector2<f32>,
    min_y: f32,
    max_y: f32,
    used: bool,
}

impl Segment {
    fn new(start: Vector2<f32>, end: Vector2<f32>) -> Self {
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);
        Self {
            start,
            end,
            min_y,
            max_y,
            used: false,
        }
    }
}

const EDGE_LUT: [[i32; 5]; 16] = [
    [0, -1, -1, -1, -1],
    [9, 0, 3, -1, -1],
    [3, 1, 0, -1, -1],
    [10, 1, 3, -1, -1],
    [6, 2, 1, -1, -1],
    [15, 0, 1, 2, 3],
    [5, 2, 0, -1, -1],
    [12, 2, 3, -1, -1],
    [12, 3, 2, -1, -1],
    [5, 0, 2, -1, -1],
    [15, 3, 0, 1, 2],
    [6, 1, 2, -1, -1],
    [10, 3, 1, -1, -1],
    [3, 0, 1, -1, -1],
    [9, 3, 0, -1, -1],
    [0, -1, -1, -1, -1],
];

fn zero_crossing(a: f32, b: f32) -> f32 {
    (a.abs() / (a.abs() + b.abs())) + 1e-6
}

#[derive(Debug, Clone)]
pub struct PolySliceStack {
    slices: Vec<PolySlice>,
    bbox: BBox3,
}

impl PolySliceStack {
    pub fn new() -> Self {
        Self {
            slices: Vec::new(),
            bbox: BBox3::empty(),
        }
    }

    pub fn from_slices(slices: Vec<PolySlice>) -> Self {
        let mut stack = Self::new();
        stack.add_slices(slices);
        stack
    }

    pub fn add_slices(&mut self, slices: Vec<PolySlice>) {
        for slice in slices {
            self.bbox.include_bbox2(&slice.bbox(), slice.z_pos());
            self.slices.push(slice);
        }
    }

    pub fn add_to_viewer(
        &self,
        viewer: &Viewer,
        outside: Option<ColorFloat>,
        inside: Option<ColorFloat>,
        degenerate: Option<ColorFloat>,
        group: i32,
    ) {
        let degenerate = degenerate.unwrap_or_else(|| {
            ColorFloat::from_hex("AAAAAAAA").unwrap_or(ColorFloat::new(0.67, 0.67, 0.67, 0.67))
        });
        let inside = inside.unwrap_or_else(|| {
            ColorFloat::from_hex("AAAAAAAA").unwrap_or(ColorFloat::new(0.67, 0.67, 0.67, 0.67))
        });
        let outside = outside.unwrap_or_else(|| {
            ColorFloat::from_hex("FF0000AA").unwrap_or(ColorFloat::new(1.0, 0.0, 0.0, 0.67))
        });

        for slice in &self.slices {
            for contour in &slice.contours {
                let color = match contour.winding() {
                    Winding::Clockwise => inside,
                    Winding::CounterClockwise => outside,
                    Winding::Unknown => degenerate,
                };

                if let Ok(mut polyline) = PolyLine::new(color) {
                    for vec in contour.vertices() {
                        polyline.add_vertex(Vector3::new(vec.x, vec.y, slice.z_pos()));
                    }
                    viewer.add_polyline(polyline, group);
                }
            }
        }
    }

    pub fn count(&self) -> usize {
        self.slices.len()
    }

    pub fn slice_at(&self, index: usize) -> Option<&PolySlice> {
        self.slices.get(index)
    }

    pub fn bbox(&self) -> BBox3 {
        self.bbox
    }

    /// C#-style alias for `bbox`.
    pub fn o_b_box(&self) -> BBox3 {
        self.bbox()
    }
}

impl Default for PolySliceStack {
    fn default() -> Self {
        Self::new()
    }
}
