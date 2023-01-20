use std::borrow::Cow;
use std::convert::TryFrom;
use std::io::prelude::*;
use std::sync::Arc;

use binrw::{binrw, BinResult, NullString, ReadOptions};
use binrw::{io::Cursor, BinRead};

use crate::classes::ClassIDType;
use crate::type_tree::{TypeField, TypeTreeObjectBinReadClassArgs};
use crate::until::{binrw_parser::*, Endian};
use crate::version15::{SerializedType, SerializedTypeBinReadArgs};
use crate::version17::{FileIdentifier, TypeTreeNode};
use crate::{Serialized, SerializedFileFormatVersion};

use super::{BuildTarget, SerializedFileCommonHeader};

#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq)]
pub struct SerializedFile {
    header: SerializedFileCommonHeader,
    endianess: Endian,
    reserved: [u8; 3],
    #[br(is_little = endianess == Endian::Little)]
    content: SerializedFileContent,
}

impl Serialized for SerializedFile {
    fn get_serialized_file_version(&self) -> &SerializedFileFormatVersion {
        &self.header.version
    }

    fn get_data_offset(&self) -> u64 {
        self.header.data_offset as u64
    }

    fn get_endianess(&self) -> &Endian {
        &self.endianess
    }

    fn get_raw_object_by_index(&self, index: u32) -> Option<super::Object> {
        let obj = self.content.objects.get(index as usize)?;
        Some(super::Object {
            path_id: obj.path_id,
            byte_start: obj.byte_start as u64,
            byte_size: obj.byte_size,
            class: ClassIDType::try_from(obj.class_id as i32).unwrap_or(ClassIDType::Object),
            type_id: obj.type_id as usize,
        })
    }

    fn get_object_count(&self) -> i32 {
        self.content.object_count
    }

    fn get_unity_version(&self) -> String {
        self.content.unity_version.to_string()
    }

    fn get_target_platform(&self) -> &BuildTarget {
        &self.content.target_platform
    }

    fn get_enable_type_tree(&self) -> bool {
        *self.content.enable_type_tree
    }

    fn get_type_object_args_by_type_id(
        &self,
        type_id: usize,
    ) -> Option<TypeTreeObjectBinReadClassArgs> {
        let stypetree = self
            .content
            .types
            .iter()
            .find(|tp| tp.class_id == type_id as i32)?;
        let type_tree = stypetree.type_tree.as_ref()?;
        let mut type_fields = Vec::new();
        let mut string_reader = Cursor::new(&type_tree.string_buffer);

        for tp in &type_tree.type_tree_node_blobs {
            type_fields.push(Arc::new(Box::new(TypeTreeNode {
                name: tp.get_name_str(&mut string_reader),
                type_name: tp.get_type_str(&mut string_reader),
                node: tp.clone(),
            }) as Box<dyn TypeField + Send + Sync>))
        }

        Some(TypeTreeObjectBinReadClassArgs::new(
            stypetree.class_id,
            type_fields,
        ))
    }
    fn get_externals(&self) -> Cow<Vec<FileIdentifier>> {
        return Cow::Borrowed(&self.content.externals);
    }
}

#[binrw]
#[derive(Debug, PartialEq)]
struct SerializedFileContent {
    unity_version: NullString,
    target_platform: BuildTarget,
    enable_type_tree: U8Bool,
    type_count: u32,
    #[br(args { count: type_count as usize, inner: SerializedTypeBinReadArgs::builder().enable_type_tree(*enable_type_tree).finalize() })]
    types: Vec<SerializedType>,
    big_id_enabled: i32,
    object_count: i32,
    #[br(args { count: object_count as usize, inner: ObjectBinReadArgs::builder().big_id_enabled(big_id_enabled != 0).finalize() })]
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
#[br(import { big_id_enabled: bool})]
#[derive(Debug, PartialEq)]
pub struct Object {
    #[br(parse_with = path_id_parser, args (big_id_enabled))]
    pub path_id: i64,
    pub byte_start: u32,
    pub byte_size: u32,
    pub type_id: i32,
    pub class_id: u16,
    pub script_type_index: i16,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct ScriptType {
    local_serialized_file_index: i32,
    local_identifier_in_file: i32,
}

pub fn path_id_parser<R: Read + Seek>(
    reader: &mut R,
    ro: &ReadOptions,
    flags: (bool,),
) -> BinResult<i64> {
    let (big_id_enabled,) = flags;

    if !big_id_enabled {
        return Ok(<i32>::read_options(reader, ro, ())? as i64);
    }
    <i64>::read_options(reader, ro, ())
}
