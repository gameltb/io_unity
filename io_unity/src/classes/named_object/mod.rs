pub mod named_object;
pub mod type_tree;

use crate::{def_unity_class, type_tree::TypeTreeObject, SerializedFileMetadata};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use std::{
    fmt,
    io::{Read, Seek, Write},
};
use supercow::Supercow;

def_unity_class!(NamedObject, NamedObjectObject);

pub trait NamedObjectObject: fmt::Debug {
    fn get_name(&self) -> Option<String>;
}

impl BinRead for NamedObject {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        Ok(NamedObject(Box::new(
            named_object::NamedObject::read_options(reader, options, args)?,
        )))
    }
}

impl BinWrite for NamedObject {
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
