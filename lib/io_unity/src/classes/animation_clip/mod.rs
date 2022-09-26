pub mod version_2018_3_0;

use std::{
    fmt,
    io::{Read, Seek, SeekFrom, Write},
};

use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};

use crate::{def_unity_class, until::UnityVersion, SerializedFileMetadata};

def_unity_class!(AnimationClip, AnimationClipObject);

pub trait AnimationClipObject: fmt::Debug {
    fn get_name(&self) -> String;
}

impl BinRead for AnimationClip {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        if args.unity_version >= UnityVersion::new(vec![2018, 3], None) {
            return Ok(AnimationClip(Box::new(
                version_2018_3_0::AnimationClip::read_options(reader, options, args)?,
            )));
        }
        Err(binrw::Error::NoVariantMatch {
            pos: reader.seek(SeekFrom::Current(0)).unwrap(),
        })
    }
}

impl BinWrite for AnimationClip {
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
