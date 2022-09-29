use std::borrow::Cow;

use std::io::{prelude::*, SeekFrom, ErrorKind};

use binrw::binrw;

use num_enum::TryFromPrimitive;

use crate::until::binrw_parser::*;
use crate::{SerializedFileMetadata, FS};

use super::AudioClipObject;

impl AudioClipObject for AudioClip {
    fn get_audio_data(&self, fs: &mut Box<dyn FS>) -> std::io::Result<Cow<Vec<u8>>> {
        if let Some(data) = &self.audio_data {
            return Ok(Cow::Borrowed(data));
        } else {
            if let Some(mut file) = fs.get_resource_file_by_path(self.source.to_string(), None) {
                file.seek(SeekFrom::Start(self.offset as u64))?;
                let mut data = vec![0u8; self.size as usize];
                file.read_exact(&mut data)?;
                return Ok(Cow::Owned(data));
            }
        }
        Err(std::io::Error::from(ErrorKind::NotFound))
    }

    fn get_name(&self) -> String {
        self.name.to_string()
    }
}

#[binrw]
#[brw(import_raw(_args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct AudioClip {
    name: AlignedString,
    load_type: i32,
    channels: i32,
    frequency: i32,
    bits_per_sample: i32,
    length: f32,
    is_tracker_format: U8Bool,
    #[br(align_before(4))]
    subsound_index: i32,
    preload_audio_data: U8Bool,
    load_in_background: U8Bool,
    legacy3_d: U8Bool,
    #[br(align_before(4))]
    source: AlignedString,
    offset: i64,
    size: i64,
    compression_format: AudioCompressionFormat,
    #[br(if(source.to_string().len() == 0), count(size))]
    audio_data: Option<Vec<u8>>,
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
