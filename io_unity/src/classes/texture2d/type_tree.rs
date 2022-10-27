use super::{Texture2DObject, TextureFormat};
use crate::type_tree::TypeTreeObject;
use crate::{def_type_tree_class, FS};
use num_enum::TryFromPrimitive;
use std::borrow::Cow;
use std::io::{prelude::*, SeekFrom};
use supercow::Supercow;

def_type_tree_class!(Texture2D);

impl Texture2DObject for Texture2D<'_> {
    fn get_width(&self) -> u64 {
        self.get_width().unwrap() as u64
    }
    fn get_height(&self) -> u64 {
        self.get_height().unwrap() as u64
    }

    fn get_texture_format(&self) -> TextureFormat {
        TextureFormat::try_from_primitive(self.get_texture_format().unwrap() as u32).unwrap()
    }

    fn get_image_data(&self, fs: &mut Box<dyn FS>) -> Option<Cow<Vec<u8>>> {
        if let Some(data) = self.get_image_data() {
            return Some(data);
        } else {
            if let Some(mut file) =
                fs.get_resource_file_by_path(self.get_stream_data_path().unwrap(), None)
            {
                file.seek(SeekFrom::Start(self.get_stream_data_offset().unwrap()));
                let mut data = vec![0u8; self.get_stream_data_size().unwrap() as usize];
                file.read_exact(&mut data);
                return Some(Cow::Owned(data));
            }
        }
        None
    }

    fn get_image_name(&self) -> String {
        self.get_name().unwrap()
    }
}

impl Texture2D<'_> {
    fn get_name(&self) -> Option<String> {
        self.inner.get_string_by_path("/Base/m_Name")
    }

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
