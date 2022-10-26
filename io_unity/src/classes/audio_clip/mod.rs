pub mod type_tree;
pub mod version_5_0_0;

use std::{
    borrow::Cow,
    fmt,
    io::{Read, Seek, SeekFrom, Write},
};

use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};

use crate::{
    def_unity_class, type_tree::TypeTreeObject, until::UnityVersion, SerializedFileMetadata, FS,
};

def_unity_class!(AudioClip, AudioClipObject);

pub trait AudioClipObject: fmt::Debug {
    fn get_audio_data(&self, fs: &mut Box<dyn FS>) -> std::io::Result<Cow<Vec<u8>>>;
    fn get_name(&self) -> String;
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
            pos: reader.seek(SeekFrom::Current(0)).unwrap(),
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
