pub mod version_2018_2_0;
pub mod version_2020_2_0;

use std::{
    borrow::Cow,
    fmt,
    io::{Read, Seek, SeekFrom, Write},
};

use binrw::{binrw, BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use image::{DynamicImage, RgbaImage};
use num_enum::TryFromPrimitive;

use crate::{def_unity_class, until::UnityVersion, SerializedFileMetadata, FS};

def_unity_class!(Texture2D, Texture2DObject);

pub trait Texture2DObject: fmt::Debug {
    fn get_width(&self) -> u64;
    fn get_height(&self) -> u64;
    fn get_texture_format(&self) -> &TextureFormat;
    fn get_image_data(&self, fs: &mut Box<dyn FS>) -> Option<Cow<Vec<u8>>>;
    fn get_image_name(&self) -> String;

    fn get_image(&self, fs: &mut Box<dyn FS>) -> Option<DynamicImage> {
        if let Some(data) = self.get_image_data(fs) {
            match self.get_texture_format() {
                TextureFormat::DXT1
                | TextureFormat::DXT3
                | TextureFormat::DXT5
                | TextureFormat::BC4
                | TextureFormat::BC5
                | TextureFormat::BC6H
                | TextureFormat::BC7
                | TextureFormat::DXT1Crunched
                | TextureFormat::DXT5Crunched => {
                    let width = self.get_width() as usize;
                    let height = self.get_height() as usize;
                    let size = width * height * 4;
                    let mut output = vec![0; size];
                    match self.get_texture_format() {
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
                        TextureFormat::BC6H => todo!(),
                        TextureFormat::BC7 => todo!(),
                        TextureFormat::DXT1Crunched => todo!(),
                        TextureFormat::DXT5Crunched => todo!(),
                        _ => todo!(),
                    }
                    let result = RgbaImage::from_raw(width as u32, height as u32, output).unwrap();
                    return Some(DynamicImage::ImageRgba8(result));
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
                    let width = self.get_width() as usize;
                    let height = self.get_height() as usize;
                    let size = width * height;
                    let mut output = vec![[0u8; 4]; size];
                    let footprint = match self.get_texture_format() {
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
                        _ => todo!(),
                    };
                    astc_decode::astc_decode(
                        &**data,
                        width as u32,
                        height as u32,
                        footprint,
                        |x, y, color| {
                            output[(x as usize + y as usize * width)] = color;
                        },
                    )
                    .unwrap();
                    let result =
                        RgbaImage::from_raw(width as u32, height as u32, output.concat()).unwrap();
                    return Some(DynamicImage::ImageRgba8(result));
                }
                _ => println!("{:?}", self.get_texture_format()),
            }
        }
        None
    }
}

impl BinRead for Texture2D {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        if args.unity_version >= UnityVersion::new(vec![2020, 2], None) {
            return Ok(Texture2D(Box::new(
                version_2020_2_0::Texture2D::read_options(reader, options, args)?,
            )));
        } else if args.unity_version >= UnityVersion::new(vec![2018, 2], None) {
            return Ok(Texture2D(Box::new(
                version_2018_2_0::Texture2D::read_options(reader, options, args)?,
            )));
        }
        Err(binrw::Error::NoVariantMatch {
            pos: reader.seek(SeekFrom::Current(0)).unwrap(),
        })
    }
}

impl BinWrite for Texture2D {
    type Args = SerializedFileMetadata;

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Ok(())
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
