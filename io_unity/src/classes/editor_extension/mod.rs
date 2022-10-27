pub mod editor_extension;
pub mod type_tree;

use crate::{def_unity_class, type_tree::TypeTreeObject, SerializedFileMetadata};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use std::{
    fmt,
    io::{Read, Seek, Write},
};


def_unity_class!(EditorExtension, EditorExtensionObject);

pub trait EditorExtensionObject: fmt::Debug {}

impl BinRead for EditorExtension {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        return Ok(EditorExtension(Box::new(
            editor_extension::EditorExtension::read_options(reader, options, args)?,
        )));
    }
}

impl BinWrite for EditorExtension {
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
