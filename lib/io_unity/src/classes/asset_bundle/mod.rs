pub mod version14;

use std::{
    fmt,
    io::{Read, Seek, Write},
};

use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};

use crate::{def_unity_class, SerializedFileMetadata};

def_unity_class!(AssetBundle, AssetBundleObject);

pub trait AssetBundleObject: fmt::Debug {}

impl BinRead for AssetBundle {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        Ok(AssetBundle(Box::new(version14::AssetBundle::read_options(
            reader, options, args,
        )?)))
    }
}

impl BinWrite for AssetBundle {
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
