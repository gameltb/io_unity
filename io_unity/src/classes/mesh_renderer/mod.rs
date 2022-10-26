pub mod mesh_renderer;
pub mod type_tree;

use std::{
    fmt,
    io::{Read, Seek, Write},
};

use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};

use crate::{def_unity_class, type_tree::TypeTreeObject, SerializedFileMetadata};

def_unity_class!(MeshRenderer, MeshRendererObject);

pub trait MeshRendererObject: fmt::Debug {}

impl BinRead for MeshRenderer {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        return Ok(MeshRenderer(Box::new(
            mesh_renderer::MeshRenderer::read_options(reader, options, args)?,
        )));
    }
}

impl BinWrite for MeshRenderer {
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
