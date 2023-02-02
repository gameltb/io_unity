use super::{AudioClip, AudioClipObject};
use crate::classes::SerializedFileRef;
use crate::type_tree::convert::TryCastFrom;
use crate::unity_asset_view::UnityAssetViewer;
use binrw::binrw;
use num_enum::TryFromPrimitive;
use std::io::{prelude::*, ErrorKind, SeekFrom};

impl AudioClipObject for AudioClip<'_> {
    fn get_audio_data(&self, viewer: &UnityAssetViewer) -> anyhow::Result<Vec<u8>> {
        let resource_source = self
            .get_resource_source()
            .ok_or(std::io::Error::from(ErrorKind::NotFound))?;
        let resource_offset = self
            .get_resource_offset()
            .ok_or(std::io::Error::from(ErrorKind::NotFound))?;
        let resource_size = self
            .get_resource_size()
            .ok_or(std::io::Error::from(ErrorKind::NotFound))?;

        if let Some(mut file) = viewer.get_resource_file_by_serialized_file_id_and_path(
            self.get_serialized_file_id(),
            &resource_source,
        ) {
            file.seek(SeekFrom::Start(resource_offset))?;
            let mut data = vec![0u8; resource_size as usize];
            file.read_exact(&mut data)?;
            return Ok(data);
        }
        Err(std::io::Error::from(ErrorKind::NotFound).into())
    }
}

impl AudioClip<'_> {
    fn get_resource_source(&self) -> Option<String> {
        String::try_cast_from(self.inner, "/Base/m_Resource/m_Source").ok()
    }

    fn get_resource_offset(&self) -> Option<u64> {
        let offset = u64::try_cast_from(self.inner, "/Base/m_Resource/m_Offset").ok();
        if offset.is_some() {
            return offset;
        }
        Some(usize::try_cast_from(self.inner, "/Base/m_Resource/m_Offset").ok()? as u64)
    }

    fn get_resource_size(&self) -> Option<u64> {
        u64::try_cast_from(self.inner, "/Base/m_Resource/m_Size").ok()
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
