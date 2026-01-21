//! TGA image I/O

use crate::{Error, Image, ImageColor, ImageData, ImageGrayScale, ImageType, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub struct TgaIo;

impl TgaIo {
    pub fn save_tga<P: AsRef<Path>>(path: P, img: &dyn Image) -> Result<()> {
        let mut file = File::create(path)?;
        Self::save_tga_writer(&mut file, img)
    }

    pub fn save_tga_writer<W: Write>(mut writer: W, img: &dyn Image) -> Result<()> {
        if img.width() > u16::MAX as usize {
            return Err(Error::InvalidParameter(
                "Image width too large for TGA".to_string(),
            ));
        }
        if img.height() > u16::MAX as usize {
            return Err(Error::InvalidParameter(
                "Image height too large for TGA".to_string(),
            ));
        }

        let mut header = TgaHeader::new(img.width() as u16, img.height() as u16);
        let is_color = matches!(img.image_type(), ImageType::Color);
        if is_color {
            header.image_type = 2;
            header.pixel_depth = 24;
        } else {
            header.image_type = 3;
            header.pixel_depth = 8;
        }

        writer.write_all(&header.to_bytes())?;

        if is_color {
            for y in 0..img.height() {
                for x in 0..img.width() {
                    let bgr = img.bgr24_value(x, y);
                    writer.write_all(&[bgr.b, bgr.g, bgr.r])?;
                }
            }
        } else {
            for y in 0..img.height() {
                for x in 0..img.width() {
                    writer.write_all(&[img.byte_value(x, y)])?;
                }
            }
        }

        Ok(())
    }

    pub fn get_file_info<P: AsRef<Path>>(path: P) -> Result<(ImageType, usize, usize)> {
        let mut file = File::open(path)?;
        Self::get_file_info_reader(&mut file)
    }

    pub fn get_file_info_reader<R: Read>(mut reader: R) -> Result<(ImageType, usize, usize)> {
        let header = TgaHeader::read(&mut reader)?;
        let width = header.width as usize;
        let height = header.height as usize;

        let image_type = match header.image_type {
            2 => ImageType::Color,
            3 => ImageType::Gray,
            _ => {
                return Err(Error::InvalidParameter(
                    "TGA has unsupported format (expecting grayscale or color)".to_string(),
                ))
            }
        };

        Ok((image_type, width, height))
    }

    pub fn load_tga<P: AsRef<Path>>(path: P) -> Result<ImageData> {
        let mut file = File::open(path)?;
        Self::load_tga_reader(&mut file)
    }

    pub fn load_tga_reader<R: Read>(mut reader: R) -> Result<ImageData> {
        let header = TgaHeader::read(&mut reader)?;

        let is_color = match header.image_type {
            2 => true,
            3 => false,
            _ => {
                return Err(Error::InvalidParameter(
                    "TGA has unsupported format (expecting grayscale or color)".to_string(),
                ))
            }
        };

        if is_color && header.pixel_depth != 24 {
            return Err(Error::InvalidParameter(
                "TGA has unsupported bit depth (expecting 24) for color TGAs".to_string(),
            ));
        }
        if !is_color && header.pixel_depth != 8 {
            return Err(Error::InvalidParameter(
                "TGA has unsupported bit depth (expecting 8) for grayscale TGAs".to_string(),
            ));
        }

        let width = header.width as usize;
        let height = header.height as usize;
        let flipped = header.y_axis_flipped();

        if is_color {
            let mut img = ImageColor::new(width, height);
            let mut buf = [0u8; 3];
            for y in 0..height {
                let iy = if flipped { height - y - 1 } else { y };
                for x in 0..width {
                    reader.read_exact(&mut buf)?;
                    img.set_bgr24(
                        x,
                        iy,
                        crate::ColorBgr24 {
                            b: buf[0],
                            g: buf[1],
                            r: buf[2],
                        },
                    );
                }
            }
            Ok(ImageData::Color(img))
        } else {
            let mut img = ImageGrayScale::new(width, height);
            let mut buf = [0u8; 1];
            for y in 0..height {
                let iy = if flipped { height - y - 1 } else { y };
                for x in 0..width {
                    reader.read_exact(&mut buf)?;
                    img.set_value(x, iy, buf[0] as f32 / 255.0);
                }
            }
            Ok(ImageData::Gray(img))
        }
    }
}

struct TgaHeader {
    image_type: u8,
    width: u16,
    height: u16,
    pixel_depth: u8,
    image_desc: u8,
}

impl TgaHeader {
    fn new(width: u16, height: u16) -> Self {
        Self {
            image_type: 3,
            width,
            height,
            pixel_depth: 8,
            image_desc: 32,
        }
    }

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = [0u8; 18];
        reader.read_exact(&mut bytes)?;
        Ok(Self::from_bytes(bytes))
    }

    fn y_axis_flipped(&self) -> bool {
        (self.image_desc & 0x20) == 0
    }

    fn from_bytes(bytes: [u8; 18]) -> Self {
        let width = u16::from_le_bytes([bytes[12], bytes[13]]);
        let height = u16::from_le_bytes([bytes[14], bytes[15]]);
        Self {
            image_type: bytes[2],
            width,
            height,
            pixel_depth: bytes[16],
            image_desc: bytes[17],
        }
    }

    fn to_bytes(&self) -> [u8; 18] {
        let mut bytes = [0u8; 18];
        bytes[2] = self.image_type;
        let width = self.width.to_le_bytes();
        let height = self.height.to_le_bytes();
        bytes[12] = width[0];
        bytes[13] = width[1];
        bytes[14] = height[0];
        bytes[15] = height[1];
        bytes[16] = self.pixel_depth;
        bytes[17] = self.image_desc;
        bytes
    }
}
