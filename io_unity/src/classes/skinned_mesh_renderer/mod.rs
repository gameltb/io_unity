pub mod type_tree;
pub mod version_4_3_0;

use super::p_ptr::PPtr;
use crate::type_tree::TypeTreeObject;
use crate::{def_unity_class, until::UnityVersion, SerializedFileMetadata};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use std::{
    fmt,
    io::{Read, Seek, SeekFrom, Write},
};
use supercow::Supercow;

def_unity_class!(SkinnedMeshRenderer, SkinnedMeshRendererObject);

pub trait SkinnedMeshRendererObject: fmt::Debug {
    fn get_bones(&self) -> Option<Supercow<Vec<PPtr>>>;
    fn get_mesh(&self) -> Option<Supercow<PPtr>>;
    fn get_materials(&self) -> Option<Supercow<Vec<PPtr>>>;
}

impl BinRead for SkinnedMeshRenderer {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        if args.unity_version >= UnityVersion::new(vec![4, 3], None) {
            return Ok(SkinnedMeshRenderer(Box::new(
                version_4_3_0::SkinnedMeshRenderer::read_options(reader, options, args)?,
            )));
        }
        Err(binrw::Error::NoVariantMatch {
            pos: reader.seek(SeekFrom::Current(0))?,
        })
    }
}

impl BinWrite for SkinnedMeshRenderer {
    type Args = SerializedFileMetadata;

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        todo!()
    }
}
