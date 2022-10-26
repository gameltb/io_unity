use std::borrow::Cow;

use std::io::{prelude::*, SeekFrom};

use binrw::binrw;

use crate::until::binrw_parser::{AlignedString, U8Bool};
use crate::{SerializedFileMetadata, FS};

use super::{Texture2DObject, TextureFormat};

impl Texture2DObject for Texture2D {
    fn get_width(&self) -> u64 {
        self.width as u64
    }
    fn get_height(&self) -> u64 {
        self.height as u64
    }

    fn get_texture_format(&self) -> TextureFormat {
        self.texture_format.clone()
    }

    fn get_image_data(&self, fs: &mut Box<dyn FS>) -> Option<Cow<Vec<u8>>> {
        if let Some(data) = &self.image_data {
            return Some(Cow::Borrowed(data));
        } else {
            if let Some(mut file) =
                fs.get_resource_file_by_path(self.stream_data.path.to_string(), None)
            {
                file.seek(SeekFrom::Start(self.stream_data.offset as u64));
                let mut data = vec![0u8; self.stream_data.size as usize];
                file.read_exact(&mut data);
                return Some(Cow::Owned(data));
            }
        }
        None
    }

    fn get_image_name(&self) -> String {
        self.name.to_string()
    }
}

#[binrw]
#[brw(import_raw(_args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Texture2D {
    name: AlignedString,

    forced_fallback_format: i32,
    downscale_fallback: U8Bool,
    #[br(align_after(4))]
    is_alpha_channel_optional: U8Bool,

    width: i32,
    height: i32,
    complete_image_size: i32,
    mips_stripped: i32,
    texture_format: TextureFormat,
    mip_count: i32,
    is_readable: U8Bool,
    is_pre_processed: U8Bool,
    ignore_master_texture_limit: U8Bool,
    streaming_mipmaps: U8Bool,
    #[br(align_before(4))]
    streaming_mipmaps_priority: i32,
    image_count: i32,
    texture_dimension: i32,
    texture_settings: GLTextureSettings,
    lightmap_format: i32,
    color_space: i32,
    platform_blob_size: i32,
    #[br(count(platform_blob_size))]
    platform_blob: Vec<u8>,
    #[br(align_before(4))]
    image_data_size: i32,
    #[br(if(image_data_size > 0), count(image_data_size))]
    image_data: Option<Vec<u8>>,
    stream_data: StreamingInfo,
}

#[binrw]
#[derive(Debug)]
pub struct GLTextureSettings {
    filter_mode: i32,
    aniso: i32,
    mip_bias: f32,
    wrap_mode: i32,
    wrap_v: i32,
    wrap_w: i32,
}

#[binrw]
#[derive(Debug)]
pub struct StreamingInfo {
    offset: u64,
    size: u32,
    path: AlignedString,
}
