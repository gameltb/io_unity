use super::AudioClipObject;
use crate::classes::named_object;
use crate::classes::named_object::NamedObjectObject;
use crate::type_tree::TypeTreeObject;
use crate::{def_type_tree_class, FS};
use binrw::binrw;
use num_enum::TryFromPrimitive;
use std::borrow::Cow;
use std::io::{prelude::*, ErrorKind, SeekFrom};
use supercow::Supercow;

def_type_tree_class!(AudioClip);

impl named_object::DownCast for AudioClip<'_> {
    fn downcast<'a>(&'a self) -> Supercow<Box<dyn NamedObjectObject + Send + 'a>> {
        Supercow::owned(Box::new(named_object::type_tree::NamedObject::new(
            &*self.inner,
        )))
    }
}

impl AudioClipObject for AudioClip<'_> {
    fn get_audio_data(&self, fs: &mut Box<dyn FS>) -> anyhow::Result<Cow<Vec<u8>>> {
        let resource_source = self
            .get_resource_source()
            .ok_or(std::io::Error::from(ErrorKind::NotFound))?;
        let resource_offset = self
            .get_resource_offset()
            .ok_or(std::io::Error::from(ErrorKind::NotFound))?;
        let resource_size = self
            .get_resource_size()
            .ok_or(std::io::Error::from(ErrorKind::NotFound))?;

        if let Some(mut file) = fs.get_resource_file_by_path(resource_source, None) {
            file.seek(SeekFrom::Start(resource_offset))?;
            let mut data = vec![0u8; resource_size as usize];
            file.read_exact(&mut data)?;
            return Ok(Cow::Owned(data));
        }
        Err(std::io::Error::from(ErrorKind::NotFound).into())
    }
}

impl AudioClip<'_> {
    fn get_resource_source(&self) -> Option<String> {
        self.inner.get_string_by_path("/Base/m_Resource/m_Source")
    }

    fn get_resource_offset(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/m_Resource/m_Offset")
    }

    fn get_resource_size(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/m_Resource/m_Size")
    }
}

#[binrw]
#[brw(repr = u32)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone)]
#[repr(u32)]
pub enum AudioCompressionFormat {
    PCM = 0,
    Vorbis = 1,
    ADPCM = 2,
    MP3 = 3,
    VAG = 4,
    HEVAG = 5,
    XMA = 6,
    AAC = 7,
    GCADPCM = 8,
    ATRAC9 = 9,
}
