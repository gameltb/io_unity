use super::{Texture2DObject, TextureFormat};
use crate::classes::named_object::{self, NamedObject, NamedObjectObject};
use crate::unity_asset_view::UnityAssetViewer;
use crate::until::binrw_parser::{AlignedString, U8Bool};
use crate::SerializedFileMetadata;
use binrw::binrw;
use std::borrow::Cow;
use std::io::{prelude::*, SeekFrom};
use supercow::Supercow;

impl named_object::DownCast for Texture2D {
    fn downcast<'a>(&'a self) -> Supercow<Box<dyn NamedObjectObject + Send + 'a>> {
        Supercow::borrowed(&*self.name)
    }
}

impl Texture2DObject for Texture2D {
    fn get_width(&self) -> Option<u64> {
        Some(self.width as u64)
    }
    fn get_height(&self) -> Option<u64> {
        Some(self.height as u64)
    }

    fn get_texture_format(&self) -> Option<TextureFormat> {
        Some(self.texture_format.clone())
    }

    fn get_image_data(&self, viewer: &UnityAssetViewer) -> Option<Cow<Vec<u8>>> {
        if let Some(data) = &self.image_data {
            return Some(Cow::Borrowed(data));
        } else {
            if let Some(mut file) = viewer.get_resource_file_by_serialized_file_id_and_path(
                0,
                &self.stream_data.path.to_string(),
            ) {
                file.seek(SeekFrom::Start(self.stream_data.offset as u64))
                    .ok()?;
                let mut data = vec![0u8; self.stream_data.size as usize];
                file.read_exact(&mut data).ok()?;
                return Some(Cow::Owned(data));
            }
        }
        None
    }
}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Texture2D {
    #[brw(args_raw = args)]
    name: NamedObject,

    forced_fallback_format: i32,
    #[br(align_after(4))]
    downscale_fallback: U8Bool,

    width: i32,
    height: i32,
    complete_image_size: i32,
    texture_format: TextureFormat,
    mip_count: i32,
    is_readable: U8Bool,
    streaming_mipmaps: U8Bool,
    #[br(align_before(4))]
    streaming_mipmaps_priority: i32,
    image_count: i32,
    texture_dimension: i32,
    texture_settings: GLTextureSettings,
    lightmap_format: i32,
    color_space: i32,
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
    offset: u32,
    size: u32,
    path: AlignedString,
}
