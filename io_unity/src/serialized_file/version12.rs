use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader, SeekFrom};

use binrw::{binrw, BinResult, Error, NullString, ReadOptions, WriteOptions}; 
use binrw::{io::Cursor, BinRead, BinWrite};
use lz4::block::{compress, decompress};
use lz4::{Decoder, EncoderBuilder};
use num_enum::TryFromPrimitive;

use crate::classes::asset_bundle::AssetBundle;
use crate::classes::audio_clip::AudioClip;
use crate::classes::mesh::Mesh;
use crate::classes::texture_2d::Texture2D;
use crate::classes::{Class, ClassIDType};
use crate::until::binrw_parser::*;
use crate::Serialized;

use super::{SerializedFileCommonHeader, COMMON_STRING}; // reading/writing utilities

#[binrw]
#[derive(Debug, PartialEq)]
pub struct SerializedFile {
    header: SerializedFileCommonHeader,
    endianess: U8Bool,
    reserved: [u8; 3],
    #[br(is_little = !*endianess)]
    content: SerializedFileContent,
}

impl Serialized for SerializedFile {
    fn get_serialized_file_header(&self) -> &SerializedFileCommonHeader {
        &self.header
    }

    fn get_endianess(&self) -> u8 {
        &self.endianess
    }

    fn get_raw_object_by_index(&self, index: u32) -> super::Object {
        let obj = self.content.objects.get(index as usize).unwrap();
        super::Object {
            path_id: obj.path_id,
            byte_start: obj.byte_start,
            byte_size: obj.byte_size,
            class: ClassIDType::try_from(obj.get_type(&self.content.types).class_id).unwrap(),
        }
    }

    fn get_object_count(&self) -> i32 {
        self.content.object_count
    }

    fn get_version(&self) -> String {
        self.content.metadata.unity_version.clone().into_string()
    }    
    
    fn get_target_platform(&self) -> u32 {
        self.content.metadata.target_platform
    }
}

#[binrw]
#[derive(Debug, PartialEq)]
struct SerializedFileContent {
    unity_version: NullString,
    target_platform: u32,
    type_count: u32,
    #[br(args { count: type_count as usize, inner: SerializedTypeBinReadArgs { enable_type_tree:*enable_type_tree } })]
    types: Vec<SerializedType>,
}
