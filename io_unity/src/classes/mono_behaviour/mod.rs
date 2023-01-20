pub mod mono_behaviour;
pub mod type_tree;

use crate::{def_unity_class, SerializedFileMetadata};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use std::{
    fmt,
    io::{Read, Seek, Write},
};

def_unity_class!(MonoBehaviour, MonoBehaviourObject);

pub trait MonoBehaviourObject: fmt::Debug {}

impl BinRead for MonoBehaviour {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        return Ok(MonoBehaviour(Box::new(
            mono_behaviour::MonoBehaviour::read_options(reader, options, args)?,
        )));
    }
}

impl BinWrite for MonoBehaviour {
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
