pub mod type_tree;
pub mod version_2018_2_0;

use crate::type_tree::TypeTreeObject;
use crate::{def_unity_class, until::UnityVersion, SerializedFileMetadata};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use std::{
    fmt,
    io::{Read, Seek, SeekFrom, Write},
};
use supercow::Supercow;

use super::named_object;

def_unity_class!(MonoScript, MonoScriptObject);

pub trait MonoScriptObject: fmt::Debug + named_object::DownCast {}

impl BinRead for MonoScript {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        if args.unity_version >= UnityVersion::new(vec![2018, 2], None) {
            return Ok(MonoScript(Box::new(
                version_2018_2_0::MonoScript::read_options(reader, options, args)?,
            )));
        }
        Err(binrw::Error::NoVariantMatch {
            pos: reader.seek(SeekFrom::Current(0))?,
        })
    }
}

impl BinWrite for MonoScript {
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
