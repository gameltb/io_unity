pub mod type_tree;
pub mod version14;

use supercow::Supercow;

use crate::type_tree::TypeTreeObject;
use crate::{def_unity_class, SerializedFileMetadata};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use std::{
    fmt,
    io::{Read, Seek, Write},
};

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
