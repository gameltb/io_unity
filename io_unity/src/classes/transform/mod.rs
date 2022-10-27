pub mod transform;
pub mod type_tree;

use super::{p_ptr::PPtr};
use crate::{def_unity_class, type_tree::TypeTreeObject, SerializedFileMetadata};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use glam::Mat4;
use std::{
    fmt,
    io::{Read, Seek, Write},
};
use supercow::Supercow;

def_unity_class!(Transform, TransformObject);

pub trait TransformObject: fmt::Debug {
    fn get_game_object(&self) -> Supercow<PPtr>;
    fn get_father(&self) -> Supercow<PPtr>;
    fn get_local_mat(&self) -> Mat4;
}

impl BinRead for Transform {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        return Ok(Transform(Box::new(transform::Transform::read_options(
            reader, options, args,
        )?)));
    }
}

impl BinWrite for Transform {
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
