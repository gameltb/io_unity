pub mod component;
pub mod type_tree;

use std::{
    fmt,
    io::{Read, Seek, Write},
};

use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};

use crate::{def_unity_class, type_tree::TypeTreeObject, SerializedFileMetadata};

use super::p_ptr::PPtr;

def_unity_class!(Component, ComponentObject);

pub trait ComponentObject: fmt::Debug {
    fn get_game_object(&self) -> &PPtr;
}

impl BinRead for Component {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        return Ok(Component(Box::new(component::Component::read_options(
            reader, options, args,
        )?)));
    }
}

impl BinWrite for Component {
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
