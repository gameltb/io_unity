use super::{Texture2DObject, TextureFormat};
use crate::classes::named_object::{self, NamedObjectObject};
use crate::type_tree::TypeTreeObject;
use crate::{def_type_tree_class, FS};
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

    fn get_image_data(&self, fs: &dyn FS) -> Option<Cow<Vec<u8>>> {
        if let Some(data) = self.get_image_data() {
            return Some(data);
        } else {
            if let Some(mut file) = fs.get_resource_file_by_path(self.get_stream_data_path()?, None)
            {
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
        self.inner.get_int_by_path("/Base/m_Width")
    }

    fn get_height(&self) -> Option<i64> {
        self.inner.get_int_by_path("/Base/m_Height")
    }

    fn get_texture_format(&self) -> Option<i64> {
        self.inner.get_int_by_path("/Base/m_TextureFormat")
    }

    fn get_image_data(&self) -> Option<Cow<Vec<u8>>> {
        if let Some(crate::type_tree::Value::ByteArray(data)) =
            self.inner.get_value_by_path("/Base/image data")
        {
            return Some(data);
        }
        None
    }

    fn get_stream_data_path(&self) -> Option<String> {
        self.inner.get_string_by_path("/Base/m_StreamData/path")
    }

    fn get_stream_data_offset(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/m_StreamData/offset")
    }

    fn get_stream_data_size(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/m_StreamData/size")
    }
}
