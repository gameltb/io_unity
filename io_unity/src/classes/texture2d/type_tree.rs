use super::{Texture2D, Texture2DObject, TextureFormat};
use crate::classes::SerializedFileRef;

use crate::type_tree::convert::TryCastFrom;
use crate::unity_asset_view::UnityAssetViewer;
use num_enum::TryFromPrimitive;
use std::io::{prelude::*, SeekFrom};

impl Texture2DObject for Texture2D<'_> {
    fn get_width(&self) -> anyhow::Result<u64> {
        self.get_width().map(|i| i as u64)
    }
    fn get_height(&self) -> anyhow::Result<u64> {
        self.get_height().map(|i| i as u64)
    }

    fn get_texture_format(&self) -> anyhow::Result<TextureFormat> {
        Ok(TextureFormat::try_from_primitive(
            self.get_texture_format()? as u32,
        )?)
    }

    fn get_image_data(&self, viewer: &UnityAssetViewer) -> anyhow::Result<Vec<u8>> {
        if let Ok(data) = self.get_image_data() {
            if !data.is_empty() {
                return Ok(data);
            }
        }

        if let Some(mut file) = viewer.get_resource_file_by_serialized_file_id_and_path(
            self.get_serialized_file_id(),
            &self.get_stream_data_path()?,
        ) {
            file.seek(SeekFrom::Start(self.get_stream_data_offset()?))?;
            let mut data = vec![0u8; self.get_stream_data_size()? as usize];
            file.read_exact(&mut data)?;
            return Ok(data);
        }
        Err(anyhow!("cannot find image data"))
    }
}

impl Texture2D<'_> {
    fn get_width(&self) -> anyhow::Result<i64> {
        i64::try_cast_from(self.inner, "/Base/m_Width")
    }

    fn get_height(&self) -> anyhow::Result<i64> {
        i64::try_cast_from(self.inner, "/Base/m_Height")
    }

    fn get_texture_format(&self) -> anyhow::Result<i64> {
        i64::try_cast_from(self.inner, "/Base/m_TextureFormat")
    }

    fn get_image_data(&self) -> anyhow::Result<Vec<u8>> {
        <Vec<u8>>::try_cast_from(self.inner, "/Base/image data")
    }

    fn get_stream_data_path(&self) -> anyhow::Result<String> {
        String::try_cast_from(self.inner, "/Base/m_StreamData/path")
    }

    fn get_stream_data_offset(&self) -> anyhow::Result<u64> {
        u64::try_cast_from(self.inner, "/Base/m_StreamData/offset")
    }

    fn get_stream_data_size(&self) -> anyhow::Result<u64> {
        u64::try_cast_from(self.inner, "/Base/m_StreamData/size")
    }
}
