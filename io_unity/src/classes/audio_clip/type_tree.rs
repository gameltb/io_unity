use std::borrow::Cow;

use std::io::{prelude::*, ErrorKind, SeekFrom};

use binrw::binrw;

use num_enum::TryFromPrimitive;

use crate::type_tree::{TypeTreeObject};

use crate::{FS};

use super::AudioClipObject;

impl AudioClipObject for AudioClip {
    fn get_audio_data(&self, fs: &mut Box<dyn FS>) -> std::io::Result<Cow<Vec<u8>>> {
        if let Some(mut file) =
            fs.get_resource_file_by_path(self.get_resource_source().unwrap(), None)
        {
            file.seek(SeekFrom::Start(self.get_resource_offset().unwrap()))?;
            let mut data = vec![0u8; self.get_resource_size().unwrap() as usize];
            file.read_exact(&mut data)?;
            return Ok(Cow::Owned(data));
        }
        Err(std::io::Error::from(ErrorKind::NotFound))
    }

    fn get_name(&self) -> String {
        self.get_name().unwrap()
    }
}

#[derive(Debug)]
pub struct AudioClip {
    inner: TypeTreeObject,
}

impl AudioClip {
    pub fn new(inner: TypeTreeObject) -> Self {
        Self { inner }
    }

    fn get_name(&self) -> Option<String> {
        self.inner.get_string_by_path("/Base/m_Name")
    }

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
