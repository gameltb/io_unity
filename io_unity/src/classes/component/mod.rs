pub mod component;
pub mod type_tree;

use super::p_ptr::PPtr;
use crate::{def_unity_class, type_tree::TypeTreeObject, SerializedFileMetadata};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use std::{
    fmt,
    io::{Read, Seek, Write},
};
use supercow::Supercow;

def_unity_class!(Component, ComponentObject);

pub trait ComponentObject: fmt::Debug {
    fn get_game_object(&self) -> Supercow<PPtr>;
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
