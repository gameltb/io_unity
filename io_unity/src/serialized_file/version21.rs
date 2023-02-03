use std::borrow::Cow;

use std::sync::Arc;

use binrw::io::Cursor;
use binrw::{binrw, NullString};

use crate::type_tree::{reader::TypeTreeObjectBinReadClassArgs, TypeField};
use crate::until::{binrw_parser::*, Endian};
use crate::version17::{FileIdentifier, Object, ScriptType};
use crate::version19::{TypeTree, TypeTreeNode};
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

    fn get_objects_metadata(&self) -> Vec<super::Object> {
        self.content
            .objects
            .iter()
            .map(|obj| super::Object {
                path_id: obj.path_id,
                byte_start: obj.byte_start as u64,
                byte_size: obj.byte_size,
                class: self
                    .content
                    .types
                    .get(obj.type_id as usize)
                    .map(|t| t.class_id)
                    .unwrap_or(0),
                type_id: obj.type_id as usize,
            })
            .collect()
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
        let stypetree = &self.content.types.get(type_id)?;
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
    ref_type_count: i32,
    #[br(args { count: ref_type_count as usize, inner: SerializedRefTypeBinReadArgs { enable_type_tree:*enable_type_tree } })]
    ref_types: Vec<SerializedRefType>,
    user_information: NullString,
}

#[binrw]
#[br(import { enable_type_tree: bool})]
#[derive(Debug, PartialEq)]
pub struct SerializedType {
    pub class_id: i32,
    pub is_stripped_type: U8Bool,
    script_type_index: i16,
    #[br(if(class_id == 114))]
    script_id: Option<[u8; 16]>,
    old_type_hash: [u8; 16],
    #[br(if(enable_type_tree))]
    pub type_tree: Option<TypeTree>,
    type_dependencies_count: i32,
    #[br(count = type_dependencies_count)]
    type_dependencies: Vec<u32>,
}

#[binrw]
#[br(import { enable_type_tree: bool})]
#[derive(Debug, PartialEq)]
pub struct SerializedRefType {
    class_id: i32,
    is_stripped_type: U8Bool,
    script_type_index: i16,
    #[br(if(class_id == 114))]
    script_id: Option<[u8; 16]>,
    old_type_hash: [u8; 16],
    #[br(if(enable_type_tree))]
    type_tree: Option<TypeTree>,
    class_name: NullString,
    name_space: NullString,
    assembly_name: NullString,
}
