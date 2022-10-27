pub mod type_tree;
pub mod version_2020_0_0;

use super::p_ptr::PPtr;
use crate::type_tree::TypeTreeObject;
use crate::{def_unity_class, until::UnityVersion, SerializedFileMetadata};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use std::{
    fmt,
    io::{Read, Seek, SeekFrom, Write},
};


def_unity_class!(Renderer, RendererObject);

pub trait RendererObject: fmt::Debug {
    fn get_materials(&self) -> &Vec<PPtr>;
}

impl BinRead for Renderer {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        if args.unity_version >= UnityVersion::new(vec![2020], None) {
            return Ok(Renderer(Box::new(
                version_2020_0_0::Renderer::read_options(reader, options, args)?,
            )));
        }
        Err(binrw::Error::NoVariantMatch {
            pos: reader.seek(SeekFrom::Current(0)).unwrap(),
        })
    }
}

impl BinWrite for Renderer {
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
