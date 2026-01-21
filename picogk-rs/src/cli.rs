//! CLI (Common Layer Interface) I/O

use crate::{BBox3, PolyContour, PolySlice, PolySliceStack, Result, Winding};
use nalgebra::{Vector2, Vector3};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliFormat {
    UseEmptyFirstLayer,
    FirstLayerWithContent,
}

#[derive(Debug, Clone)]
pub struct CliResult {
    pub slices: PolySliceStack,
    pub bbox_file: BBox3,
    pub is_binary: bool,
    pub units_header: f32,
    pub align_32bit: bool,
    pub version: u32,
    pub header_date: String,
    pub layer_count: u32,
    pub warnings: String,
}

pub struct CliIo;

impl CliIo {
    pub fn write_slices_to_cli_file<P: AsRef<Path>>(
        slices: &PolySliceStack,
        path: P,
        format: CliFormat,
        date: Option<&str>,
        units_mm: Option<f32>,
    ) -> Result<()> {
        if slices.count() < 1 || slices.bbox().is_empty() {
            return Err(crate::Error::InvalidParameter(
                "No valid slices detected (empty)".to_string(),
            ));
        }

        let units = units_mm.unwrap_or(1.0);
        if units <= 0.0 {
            return Err(crate::Error::InvalidParameter(
                "Units must be positive".to_string(),
            ));
        }

        let date = date.unwrap_or("1970-01-01");

        let mut file = File::create(path)?;

        writeln!(file, "$$HEADERSTART")?;
        writeln!(file, "$$ASCII")?;
        writeln!(file, "$$UNITS/{:08.5}", units)?;
        writeln!(file, "$$VERSION/200")?;
        writeln!(file, "$$LABEL/1,default")?;
        writeln!(file, "$$DATE/{}", date)?;

        let bbox = slices.bbox();
        let last_slice = slices.slice_at(slices.count() - 1).ok_or_else(|| {
            crate::Error::OperationFailed("SliceStack missing last slice".to_string())
        })?;

        let str_dim = format!(
            "{:08.5},{:08.5},{:08.5},{:08.5},{:08.5},{:08.5}",
            bbox.min().x,
            bbox.min().y,
            0.0,
            bbox.max().x,
            bbox.max().y,
            last_slice.z_pos()
        );

        let mut slice_count = slices.count() as u32;
        if format == CliFormat::UseEmptyFirstLayer {
            slice_count += 1;
        }

        writeln!(file, "$$DIMENSION/{}", str_dim)?;
        writeln!(file, "$$LAYERS/{:05}", slice_count)?;
        writeln!(file, "$$HEADEREND")?;
        writeln!(file, "$$GEOMETRYSTART")?;

        if format == CliFormat::UseEmptyFirstLayer {
            writeln!(file, "$$LAYER/0.0")?;
        }

        for slice_idx in 0..slices.count() {
            let slice = slices.slice_at(slice_idx).ok_or_else(|| {
                crate::Error::OperationFailed(format!("SliceStack missing slice {}", slice_idx))
            })?;
            writeln!(file, "$$LAYER/{:.5}", slice.z_pos() / units)?;

            for pass in 0..3 {
                for contour_idx in 0..slice.contour_count() {
                    let contour = slice.contour_at(contour_idx).ok_or_else(|| {
                        crate::Error::OperationFailed(format!(
                            "Slice {} missing contour {}",
                            slice_idx, contour_idx
                        ))
                    })?;

                    if pass == 0 {
                        if contour.winding() != Winding::CounterClockwise {
                            continue;
                        }
                    } else if pass == 1 {
                        if contour.winding() != Winding::Clockwise {
                            continue;
                        }
                    } else if contour.winding() != Winding::Unknown {
                        continue;
                    }

                    let winding = match contour.winding() {
                        Winding::Clockwise => 0,
                        Winding::CounterClockwise => 1,
                        Winding::Unknown => 2,
                    };

                    let mut line = format!("$$POLYLINE/1,{},{},", winding, contour.count());
                    for vertex in contour.vertices() {
                        line.push_str(&format!("{:.5},{:.5},", vertex.x / units, vertex.y / units));
                    }
                    if line.ends_with(',') {
                        line.pop();
                    }
                    writeln!(file, "{}", line)?;
                }
            }
        }

        writeln!(file, "$$GEOMETRYEND")?;
        Ok(())
    }

    pub fn slices_from_cli_file<P: AsRef<Path>>(path: P) -> Result<CliResult> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut result = CliResult {
            slices: PolySliceStack::new(),
            bbox_file: BBox3::empty(),
            is_binary: false,
            units_header: 0.0,
            align_32bit: false,
            version: 0,
            header_date: String::new(),
            layer_count: 0,
            warnings: String::new(),
        };

        let mut header_started = false;
        let mut header_ended = false;
        let mut geometry_started = false;
        let mut label_id: Option<u32> = None;

        let mut current_slice: Option<PolySlice> = None;
        let mut slices: Vec<PolySlice> = Vec::new();
        let mut prev_z = f32::MIN;

        for (line_no, line) in reader.lines().enumerate() {
            let mut line = line?;
            if let Some(idx) = line.find("//") {
                line = line[..idx].to_string();
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if !header_started {
                if let Some(idx) = line.find("$$HEADERSTART") {
                    header_started = true;
                    let remainder = &line[idx + "$$HEADERSTART".len()..];
                    if remainder.trim().is_empty() {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            if header_started && !header_ended {
                if line.starts_with("$$HEADEREND") {
                    header_ended = true;
                    continue;
                }

                if line.starts_with("$$BINARY") {
                    result.is_binary = true;
                    continue;
                }
                if line.starts_with("$$ASCII") {
                    result.is_binary = false;
                    continue;
                }
                if line.starts_with("$$ALIGN") {
                    result.align_32bit = true;
                    continue;
                }
                if let Some(mut data) = line.strip_prefix("$$UNITS") {
                    let param = extract_parameter(&mut data).ok_or_else(|| {
                        crate::Error::InvalidParameter(
                            "Missing parameter after $$UNITS".to_string(),
                        )
                    })?;
                    result.units_header = param.parse::<f32>().map_err(|_| {
                        crate::Error::InvalidParameter(format!(
                            "Invalid parameter for $$UNITS: {}",
                            param
                        ))
                    })?;
                    if result.units_header <= 0.0 {
                        return Err(crate::Error::InvalidParameter(format!(
                            "Invalid parameter for $$UNITS: {}",
                            param
                        )));
                    }
                    continue;
                }
                if line.starts_with("$$VERSION") {
                    continue;
                }
                if let Some(mut data) = line.strip_prefix("$$LABEL") {
                    let id = extract_parameter(&mut data).ok_or_else(|| {
                        crate::Error::InvalidParameter(
                            "Missing parameter after $$LABEL".to_string(),
                        )
                    })?;
                    let id = id.parse::<u32>().map_err(|_| {
                        crate::Error::InvalidParameter(format!(
                            "Invalid parameter for $$LABEL: {}",
                            id
                        ))
                    })?;
                    if label_id.is_some() {
                        return Err(crate::Error::InvalidParameter(
                            "Multiple labels not supported".to_string(),
                        ));
                    }
                    label_id = Some(id);
                    let _label = extract_parameter(&mut data).ok_or_else(|| {
                        crate::Error::InvalidParameter(
                            "Missing parameter after $$LABEL (text)".to_string(),
                        )
                    })?;
                    continue;
                }
                if let Some(mut data) = line.strip_prefix("$$DATE") {
                    let param = extract_parameter(&mut data).ok_or_else(|| {
                        crate::Error::InvalidParameter("Missing parameter after $$DATE".to_string())
                    })?;
                    result.header_date = param.trim().to_string();
                    continue;
                }
                if let Some(mut data) = line.strip_prefix("$$DIMENSION") {
                    let mut read_param = |name: &str| -> Result<f32> {
                        let value = extract_parameter(&mut data).ok_or_else(|| {
                            crate::Error::InvalidParameter(format!(
                                "Missing parameter ({}) after $$DIMENSION",
                                name
                            ))
                        })?;
                        value.parse::<f32>().map_err(|_| {
                            crate::Error::InvalidParameter(format!(
                                "Invalid parameter ({}) for $$DIMENSION: {}",
                                name, value
                            ))
                        })
                    };

                    let min = Vector3::new(
                        read_param("xMin")?,
                        read_param("yMin")?,
                        read_param("zMin")?,
                    );
                    let max = Vector3::new(
                        read_param("xMax")?,
                        read_param("yMax")?,
                        read_param("zMax")?,
                    );
                    result.bbox_file = BBox3::new(min, max);
                    continue;
                }
                if let Some(mut data) = line.strip_prefix("$$LAYERS") {
                    let param = extract_parameter(&mut data).ok_or_else(|| {
                        crate::Error::InvalidParameter(
                            "Missing parameter after $$LAYERS".to_string(),
                        )
                    })?;
                    result.layer_count = param.parse::<u32>().map_err(|_| {
                        crate::Error::InvalidParameter(format!(
                            "Invalid parameter for $$LAYERS: {}",
                            param
                        ))
                    })?;
                    continue;
                }

                continue;
            }

            if result.is_binary {
                return Err(crate::Error::InvalidParameter(
                    "Binary CLI Files are not yet supported".to_string(),
                ));
            }

            if header_ended && !geometry_started {
                if let Some(idx) = line.find("$$GEOMETRYSTART") {
                    geometry_started = true;
                    let remainder = &line[idx + "$$GEOMETRYSTART".len()..];
                    if remainder.trim().is_empty() {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            if geometry_started {
                if line.starts_with("$$GEOMETRYEND") {
                    break;
                }

                if let Some(mut data) = line.strip_prefix("$$LAYER") {
                    let param = extract_parameter(&mut data).ok_or_else(|| {
                        crate::Error::InvalidParameter(
                            "Missing parameter after $$LAYER".to_string(),
                        )
                    })?;
                    let mut z_pos = param.parse::<f32>().map_err(|_| {
                        crate::Error::InvalidParameter(format!(
                            "Invalid parameter for $$LAYER: {}",
                            param
                        ))
                    })?;
                    z_pos *= result.units_header;

                    if prev_z != f32::MIN && z_pos < prev_z {
                        return Err(crate::Error::InvalidParameter(
                            "Z position in current layer is smaller than in previous".to_string(),
                        ));
                    }
                    prev_z = z_pos;

                    if z_pos > 0.0 {
                        if let Some(slice) = current_slice.take() {
                            slices.push(slice);
                        }
                        current_slice = Some(PolySlice::new(z_pos));
                    }
                    continue;
                }

                if let Some(mut data) = line.strip_prefix("$$POLYLINE") {
                    let mut slice = current_slice.take().ok_or_else(|| {
                        crate::Error::InvalidParameter(
                            "There should not be contours at z position 0".to_string(),
                        )
                    })?;
                    let id = extract_parameter(&mut data).ok_or_else(|| {
                        crate::Error::InvalidParameter(
                            "Missing parameter after $$POLYLINE".to_string(),
                        )
                    })?;
                    let id = id.parse::<u32>().map_err(|_| {
                        crate::Error::InvalidParameter(format!(
                            "Invalid parameter for $$POLYLINE: {}",
                            id
                        ))
                    })?;

                    if label_id.is_none() {
                        label_id = Some(id);
                    }
                    if label_id != Some(id) {
                        return Err(crate::Error::InvalidParameter(
                            "Multiple labels not supported".to_string(),
                        ));
                    }

                    let winding_val = extract_parameter(&mut data).ok_or_else(|| {
                        crate::Error::InvalidParameter(
                            "Missing parameter after $$POLYLINE".to_string(),
                        )
                    })?;
                    let winding_val = winding_val.parse::<u32>().map_err(|_| {
                        crate::Error::InvalidParameter(format!(
                            "Invalid parameter for $$POLYLINE direction: {}",
                            winding_val
                        ))
                    })?;
                    let winding = match winding_val {
                        0 => Winding::Clockwise,
                        1 => Winding::CounterClockwise,
                        2 => Winding::Unknown,
                        _ => {
                            return Err(crate::Error::InvalidParameter(format!(
                                "Invalid parameter for $$POLYLINE direction: {}",
                                winding_val
                            )))
                        }
                    };

                    let count = extract_parameter(&mut data).ok_or_else(|| {
                        crate::Error::InvalidParameter(
                            "Missing parameter polygon count after $$POLYLINE".to_string(),
                        )
                    })?;
                    let mut count = count.parse::<u32>().map_err(|_| {
                        crate::Error::InvalidParameter(format!(
                            "Invalid parameter for $$POLYLINE polygon count: {}",
                            count
                        ))
                    })?;

                    let mut vertices = Vec::new();
                    while count > 0 {
                        let x = extract_parameter(&mut data).ok_or_else(|| {
                            crate::Error::InvalidParameter(
                                "Missing vertices in $$POLYLINE".to_string(),
                            )
                        })?;
                        let x = x.parse::<f32>().map_err(|_| {
                            crate::Error::InvalidParameter(format!(
                                "Invalid parameter (X) for $$POLYLINE vertex: {}",
                                x
                            ))
                        })?;

                        let y = extract_parameter(&mut data).ok_or_else(|| {
                            crate::Error::InvalidParameter(
                                "Missing vertices in $$POLYLINE".to_string(),
                            )
                        })?;
                        let y = y.parse::<f32>().map_err(|_| {
                            crate::Error::InvalidParameter(format!(
                                "Invalid parameter (Y) for $$POLYLINE vertex: {}",
                                y
                            ))
                        })?;

                        vertices.push(Vector2::new(
                            x * result.units_header,
                            y * result.units_header,
                        ));
                        count -= 1;
                    }

                    if vertices.len() < 3 {
                        result.warnings.push_str(&format!(
                            "Line: {} Discarding POLYLINE with {} vertices which is degenerate\n",
                            line_no + 1,
                            vertices.len()
                        ));
                        current_slice = Some(slice);
                        continue;
                    }

                    match PolyContour::new(vertices, winding) {
                        Ok(contour) => {
                            if contour.winding() == Winding::Unknown {
                                result.warnings.push_str(&format!(
                                    "Line: {} Discarding POLYLINE with area 0 (degenerate) - defined with winding {}\n",
                                    line_no + 1,
                                    winding.as_string()
                                ));
                            } else if contour.winding() != winding {
                                result.warnings.push_str(&format!(
                                    "Line: {} POLYLINE defined with winding {} actual winding is {} (using actual)\n",
                                    line_no + 1,
                                    winding.as_string(),
                                    contour.winding().as_string()
                                ));
                                slice.add_contour(contour);
                            } else {
                                slice.add_contour(contour);
                            }
                        }
                        Err(_) => {
                            result.warnings.push_str(&format!(
                                "Line: {} Discarding POLYLINE with invalid vertices\n",
                                line_no + 1
                            ));
                        }
                    }

                    current_slice = Some(slice);
                    continue;
                }

                if line.starts_with("$$") {
                    result.warnings.push_str(&format!(
                        "Line: {} Unsupported command {}\n",
                        line_no + 1,
                        shorten(line, 20)
                    ));
                }
            }
        }

        if let Some(slice) = current_slice.take() {
            slices.push(slice);
        }

        result.slices.add_slices(slices);
        Ok(result)
    }
}

fn extract_parameter(line: &mut &str) -> Option<String> {
    let mut data = *line;
    if data.starts_with('/') || data.starts_with(',') {
        data = &data[1..];
    } else {
        return None;
    }

    let mut end = data.len();
    for (idx, ch) in data.char_indices() {
        if ch == '$' || ch == '/' || ch == ',' {
            end = idx;
            break;
        }
    }

    let param = data[..end].trim().to_string();
    *line = &data[end..];
    Some(param)
}

fn shorten(value: &str, max_chars: usize) -> String {
    if value.len() <= max_chars {
        value.to_string()
    } else {
        value.chars().take(max_chars).collect()
    }
}
