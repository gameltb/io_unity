pub mod type_tree;

use crate::{def_unity_class, unity_asset_view::UnityAssetViewer};
use binrw::binrw;
use image::{DynamicImage, GrayAlphaImage, RgbImage, RgbaImage};
use num_enum::TryFromPrimitive;
use std::{borrow::Cow, fmt};

def_unity_class!(Texture2D);

pub trait Texture2DObject: fmt::Debug {
    fn get_width(&self) -> Option<u64>;
    fn get_height(&self) -> Option<u64>;
    fn get_texture_format(&self) -> Option<TextureFormat>;
    fn get_image_data(&self, viewer: &UnityAssetViewer) -> Option<Vec<u8>>;

    fn get_image(&self, viewer: &UnityAssetViewer) -> anyhow::Result<DynamicImage> {
        let data = self.get_image_data(viewer).ok_or(anyhow!("data"))?;
        let texture_format = self.get_texture_format().ok_or(anyhow!("texture_format"))?;
        let width = self.get_width().ok_or(anyhow!("width"))? as usize;
        let height = self.get_height().ok_or(anyhow!("height"))? as usize;

        match &texture_format {
            TextureFormat::DXT1
            | TextureFormat::DXT3
            | TextureFormat::DXT5
            | TextureFormat::BC4
            | TextureFormat::BC5
            | TextureFormat::BC6H
            | TextureFormat::BC7
            | TextureFormat::DXT1Crunched
            | TextureFormat::DXT5Crunched => {
                let size = width * height * 4;
                let mut output = vec![0; size];
                match &texture_format {
                    TextureFormat::DXT1 => {
                        texpresso::Format::Bc1.decompress(&data, width, height, &mut output)
                    }
                    TextureFormat::DXT3 => {
                        texpresso::Format::Bc2.decompress(&data, width, height, &mut output)
                    }
                    TextureFormat::DXT5 => {
                        texpresso::Format::Bc3.decompress(&data, width, height, &mut output)
                    }

                    TextureFormat::BC4 => {
                        texpresso::Format::Bc4.decompress(&data, width, height, &mut output)
                    }
                    TextureFormat::BC5 => {
                        texpresso::Format::Bc5.decompress(&data, width, height, &mut output)
                    }
                    TextureFormat::BC6H
                    | TextureFormat::BC7
                    | TextureFormat::DXT1Crunched
                    | TextureFormat::DXT5Crunched => {
                        return Err(anyhow!("unsupport {:?}", self.get_texture_format()))
                    }
                    _ => unreachable!(),
                }
                let result = RgbaImage::from_raw(width as u32, height as u32, output)
                    .ok_or(anyhow!("from_raw"))?;
                Ok(DynamicImage::ImageRgba8(result))
            }
            TextureFormat::ASTC_RGB_4x4
            | TextureFormat::ASTC_RGB_5x5
            | TextureFormat::ASTC_RGB_6x6
            | TextureFormat::ASTC_RGB_8x8
            | TextureFormat::ASTC_RGB_10x10
            | TextureFormat::ASTC_RGB_12x12
            | TextureFormat::ASTC_RGBA_4x4
            | TextureFormat::ASTC_RGBA_5x5
            | TextureFormat::ASTC_RGBA_6x6
            | TextureFormat::ASTC_RGBA_8x8
            | TextureFormat::ASTC_RGBA_10x10
            | TextureFormat::ASTC_RGBA_12x12
            | TextureFormat::ASTC_HDR_4x4
            | TextureFormat::ASTC_HDR_5x5
            | TextureFormat::ASTC_HDR_6x6
            | TextureFormat::ASTC_HDR_8x8
            | TextureFormat::ASTC_HDR_10x10
            | TextureFormat::ASTC_HDR_12x12 => {
                let size = width * height;
                let mut output = vec![[0u8; 4]; size];
                let footprint = match &texture_format {
                    TextureFormat::ASTC_RGB_4x4
                    | TextureFormat::ASTC_RGBA_4x4
                    | TextureFormat::ASTC_HDR_4x4 => astc_decode::Footprint::new(4, 4),
                    TextureFormat::ASTC_RGB_5x5
                    | TextureFormat::ASTC_RGBA_5x5
                    | TextureFormat::ASTC_HDR_5x5 => astc_decode::Footprint::new(5, 5),
                    TextureFormat::ASTC_RGB_6x6
                    | TextureFormat::ASTC_RGBA_6x6
                    | TextureFormat::ASTC_HDR_6x6 => astc_decode::Footprint::new(6, 6),
                    TextureFormat::ASTC_RGB_8x8
                    | TextureFormat::ASTC_RGBA_8x8
                    | TextureFormat::ASTC_HDR_8x8 => astc_decode::Footprint::new(8, 8),
                    TextureFormat::ASTC_RGB_10x10
                    | TextureFormat::ASTC_RGBA_10x10
                    | TextureFormat::ASTC_HDR_10x10 => astc_decode::Footprint::new(10, 10),
                    TextureFormat::ASTC_RGB_12x12
                    | TextureFormat::ASTC_RGBA_12x12
                    | TextureFormat::ASTC_HDR_12x12 => astc_decode::Footprint::new(12, 12),
                    _ => unreachable!(),
                };
                astc_decode::astc_decode(
                    &*data,
                    width as u32,
                    height as u32,
                    footprint,
                    |x, y, color| {
                        output[(x as usize + y as usize * width)] = color;
                    },
                )?;

                let result = RgbaImage::from_raw(width as u32, height as u32, output.concat())
                    .ok_or(anyhow!("from_raw"))?;
                Ok(DynamicImage::ImageRgba8(result))
            }
            TextureFormat::Alpha8 => {
                let buff: Vec<[u8; 2]> = data.into_iter().map(|f| [0, f]).collect();
                let result = GrayAlphaImage::from_raw(width as u32, height as u32, buff.concat())
                    .ok_or(anyhow!("from_raw"))?;
                Ok(DynamicImage::ImageLumaA8(result))
            }
            TextureFormat::RGB24 => {
                let result = RgbImage::from_raw(width as u32, height as u32, data.to_vec())
                    .ok_or(anyhow!("from_raw"))?;
                Ok(DynamicImage::ImageRgb8(result))
            }
            TextureFormat::RGBA32 => {
                let result = RgbaImage::from_raw(width as u32, height as u32, data.to_vec())
                    .ok_or(anyhow!("from_raw"))?;
                Ok(DynamicImage::ImageRgba8(result))
            }
            _ => Err(anyhow!(
                "unsupport texture_format: {:?}",
                self.get_texture_format()
            )),
        }
    }
}

#[binrw]
#[brw(repr = u32)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone)]
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum TextureFormat {
    Alpha8 = 1,
    ARGB4444,
    RGB24,
    RGBA32,
    ARGB32,
    ARGBFloat,
    RGB565,
    BGR24,
    R16,
    DXT1,
    DXT3,
    DXT5,
    RGBA4444,
    BGRA32,
    RHalf,
    RGHalf,
    RGBAHalf,
    RFloat,
    RGFloat,
    RGBAFloat,
    YUY2,
    RGB9e5Float,
    RGBFloat,
    BC6H,
    BC7,
    BC4,
    BC5,
    DXT1Crunched,
    DXT5Crunched,
    PVRTC_RGB2,
    PVRTC_RGBA2,
    PVRTC_RGB4,
    PVRTC_RGBA4,
    ETC_RGB4,
    ATC_RGB4,
    ATC_RGBA8,
    EAC_R = 41,
    EAC_R_SIGNED,
    EAC_RG,
    EAC_RG_SIGNED,
    ETC2_RGB,
    ETC2_RGBA1,
    ETC2_RGBA8,
    ASTC_RGB_4x4,
    ASTC_RGB_5x5,
    ASTC_RGB_6x6,
    ASTC_RGB_8x8,
    ASTC_RGB_10x10,
    ASTC_RGB_12x12,
    ASTC_RGBA_4x4,
    ASTC_RGBA_5x5,
    ASTC_RGBA_6x6,
    ASTC_RGBA_8x8,
    ASTC_RGBA_10x10,
    ASTC_RGBA_12x12,
    ETC_RGB4_3DS,
    ETC_RGBA8_3DS,
    RG16,
    R8,
    ETC_RGB4Crunched,
    ETC2_RGBA8Crunched,
    ASTC_HDR_4x4,
    ASTC_HDR_5x5,
    ASTC_HDR_6x6,
    ASTC_HDR_8x8,
    ASTC_HDR_10x10,
    ASTC_HDR_12x12,
    RG32,
    RGB48,
    RGBA64,
}
