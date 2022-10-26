pub mod mesh_filter;
pub mod type_tree;

use std::{
    fmt,
    io::{Read, Seek, Write},
};

use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};

use crate::{def_unity_class, type_tree::TypeTreeObject, SerializedFileMetadata};

def_unity_class!(MeshFilter, MeshFilterObject);

pub trait MeshFilterObject: fmt::Debug {}

impl BinRead for MeshFilter {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        return Ok(MeshFilter(Box::new(mesh_filter::MeshFilter::read_options(
            reader, options, args,
        )?)));
    }
}

impl BinWrite for MeshFilter {
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
