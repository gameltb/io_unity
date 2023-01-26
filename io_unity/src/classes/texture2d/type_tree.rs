use super::{Texture2DObject, TextureFormat};
use crate::classes::named_object::{self, NamedObjectObject};
use crate::def_type_tree_class;
use crate::type_tree::convert::TryCastFrom;
use crate::type_tree::convert::TryCastRefFrom;
use crate::type_tree::TypeTreeObject;
use crate::unity_asset_view::UnityAssetViewer;
use num_enum::TryFromPrimitive;
use std::borrow::Cow;
use std::io::{prelude::*, SeekFrom};
use supercow::Supercow;

def_type_tree_class!(Texture2D);

impl named_object::DownCast for Texture2D<'_> {
    fn downcast<'a>(&'a self) -> Supercow<Box<dyn NamedObjectObject + Send + 'a>> {
        Supercow::owned(Box::new(named_object::type_tree::NamedObject::new(
            &*self.inner,
        )))
    }
}

impl Texture2DObject for Texture2D<'_> {
    fn get_width(&self) -> Option<u64> {
        self.get_width().and_then(|i| Some(i as u64))
    }
    fn get_height(&self) -> Option<u64> {
        self.get_height().and_then(|i| Some(i as u64))
    }

    fn get_texture_format(&self) -> Option<TextureFormat> {
        TextureFormat::try_from_primitive(self.get_texture_format()? as u32).ok()
    }

    fn get_image_data(&self, viewer: &UnityAssetViewer) -> Option<Cow<Vec<u8>>> {
        if let Some(data) = self.get_image_data() {
            return Some(data);
        } else {
            if let Some(mut file) = viewer.get_resource_file_by_serialized_file_id_and_path(
                self.get_serialized_file_id(),
                &self.get_stream_data_path()?,
            ) {
                file.seek(SeekFrom::Start(self.get_stream_data_offset()?))
                    .ok()?;
                let mut data = vec![0u8; self.get_stream_data_size()? as usize];
                file.read_exact(&mut data).ok()?;
                return Some(Cow::Owned(data));
            }
        }
        None
    }
}

impl Texture2D<'_> {
    fn get_width(&self) -> Option<i64> {
        i64::try_cast_from(&self.inner, "/Base/m_Width").ok()
    }

    fn get_height(&self) -> Option<i64> {
        i64::try_cast_from(&self.inner, "/Base/m_Height").ok()
    }

    fn get_texture_format(&self) -> Option<i64> {
        i64::try_cast_from(&self.inner, "/Base/m_TextureFormat").ok()
    }

    fn get_image_data(&self) -> Option<Cow<Vec<u8>>> {
        Some(Cow::Borrowed(
            <Vec<u8>>::try_cast_as_from(&self.inner, "/Base/image data").ok()?,
        ))
    }

    fn get_stream_data_path(&self) -> Option<String> {
        String::try_cast_from(&self.inner, "/Base/m_StreamData/path").ok()
    }

    fn get_stream_data_offset(&self) -> Option<u64> {
        u64::try_cast_from(&self.inner, "/Base/m_StreamData/offset").ok()
    }

    fn get_stream_data_size(&self) -> Option<u64> {
        u64::try_cast_from(&self.inner, "/Base/m_StreamData/size").ok()
    }
}
