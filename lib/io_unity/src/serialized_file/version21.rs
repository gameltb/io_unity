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
    enable_type_tree: U8Bool,
    type_count: u32,
    #[br(args { count: type_count as usize, inner: SerializedTypeBinReadArgs { enable_type_tree:*enable_type_tree } })]
    types: Vec<SerializedType>,
    object_count: i32,
    #[br(count = object_count)]
    objects: Vec<Object>,
    script_count: i32,
    #[br(count = script_count)]
    script_types: Vec<ScriptType>,
    externals_count: i32,
    #[br(count = externals_count)]
    externals: Vec<FileIdentifier>,
    user_information: NullString,
}


#[binrw]
#[br(import { enable_type_tree: bool})]
#[derive(Debug, PartialEq)]
pub struct SerializedType {
    class_id: i32,
    is_stripped_type: U8Bool,
    script_type_index: i16,
    #[br(if(class_id == 114))]
    script_id: Option<[u8; 16]>,
    old_type_hash: [u8; 16],
    #[br(if(enable_type_tree))]
    type_tree: Option<TypeTree>,
}

#[binrw]
#[derive(PartialEq)]
struct TypeTree {
    number_of_nodes: i32,
    string_buffer_size: i32,
    #[br(count = number_of_nodes)]
    type_tree_node_blobs: Vec<TypeTreeNodeBlob>,
    #[br(count = string_buffer_size)]
    string_buffer: Vec<u8>,
}

impl fmt::Debug for TypeTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut StringReader = Cursor::new(self.string_buffer.clone());

        write!(f, "TypeTree [")?;
        if f.alternate() {
            write!(f, "\n")?;
        }
        for node in &self.type_tree_node_blobs {
            write!(
                f,
                "{:?} -> {{ type: {}, name: {} }},",
                node,
                node.get_type_str(&mut StringReader),
                node.get_name_str(&mut StringReader)
            )?;
            if f.alternate() {
                write!(f, "\n")?;
            }
        }
        write!(f, "]")
    }
}

#[binrw]
#[derive(Debug, PartialEq)]
struct TypeTreeNodeBlob {
    version: u16,
    level: u8,
    type_flags: u8,
    type_str_offset: u32,
    name_str_offset: u32,
    byte_size: i32,
    index: i32,
    meta_flag: i32,
}

impl TypeTreeNodeBlob {
    fn get_type_str<R: Read + Seek>(&self, reader: &mut R) -> String {
        read_type_tree_string(self.type_str_offset, reader)
    }

    fn get_name_str<R: Read + Seek>(&self, reader: &mut R) -> String {
        read_type_tree_string(self.name_str_offset, reader)
    }
}

fn read_type_tree_string<R: Read + Seek>(value: u32, reader: &mut R) -> String {
    let is_offset = (value & 0x80000000) == 0;
    if is_offset {
        reader.seek(SeekFrom::Start(value.into()));
        return NullString::read(reader).unwrap().to_string();
    }
    let offset = value & 0x7FFFFFFF;
    COMMON_STRING.get(&offset).unwrap().to_string()
}

#[binrw]
#[derive(Debug, PartialEq)]
struct Object {
    #[br(align_before(4))]
    path_id: i64,
    byte_start: u32,
    byte_size: u32,
    type_id: i32,
}

impl Object {
    fn get_type<'a>(&self, types: &'a Vec<SerializedType>) -> &'a SerializedType {
        types.get(self.type_id as usize).unwrap()
    }
}

#[binrw]
#[derive(Debug, PartialEq)]
struct ScriptType {
    local_serialized_file_index: i32,
    #[br(align_before(4))]
    local_identifier_in_file: i64,
}

#[binrw]
#[derive(Debug, PartialEq)]
struct FileIdentifier {
    temp_empty: NullString,
    guid: [u8; 16],
    r#type: i32,
    path: NullString,
}
