pub mod type_tree;
pub mod version_5_0_0;

use super::named_object;
use crate::{def_unity_class, until::UnityVersion, SerializedFileMetadata, FS};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use std::{
    borrow::Cow,
    fmt,
    io::{Read, Seek, SeekFrom, Write},
};

def_unity_class!(AudioClip, AudioClipObject);

pub trait AudioClipObject: fmt::Debug + named_object::DownCast {
    fn get_audio_data(&self, fs: &mut Box<dyn FS>) -> anyhow::Result<Cow<Vec<u8>>>;
}

impl BinRead for AudioClip {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        if args.unity_version >= UnityVersion::new(vec![5], None) {
            return Ok(AudioClip(Box::new(version_5_0_0::AudioClip::read_options(
                reader, options, args,
            )?)));
        }
        Err(binrw::Error::NoVariantMatch {
            pos: reader.seek(SeekFrom::Current(0))?,
        })
    }
}

impl BinWrite for AudioClip {
    type Args = SerializedFileMetadata;

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Ok(())
    }
}
