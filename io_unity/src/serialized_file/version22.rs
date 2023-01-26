use std::borrow::Cow;
use std::convert::TryFrom;
use std::sync::Arc;

use binrw::io::Cursor;
use binrw::{binrw, NullString};

use crate::classes::ClassIDType;
use crate::type_tree::{reader::TypeTreeObjectBinReadClassArgs, TypeField};
use crate::until::{binrw_parser::*, Endian};
use crate::version17::{FileIdentifier, ScriptType};
use crate::version19::TypeTreeNode;
use crate::version21::{
    SerializedRefType, SerializedRefTypeBinReadArgs, SerializedType, SerializedTypeBinReadArgs,
};
use crate::{Serialized, SerializedFileFormatVersion};

use super::{BuildTarget, SerializedFileCommonHeader};

#[binrw]
#[br(big)]
#[derive(Debug, Eq, PartialEq)]
struct SerializedFileHeader {
    metadata_size: u32,
    file_size: u64,
    data_offset: u64,
    unknown: u64,
}

#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq)]
pub struct SerializedFile {
    header: SerializedFileCommonHeader,
    endianess: Endian,
    reserved: [u8; 3],
    header2: SerializedFileHeader,
    #[br(is_little = endianess == Endian::Little)]
    content: SerializedFileContent,
}

impl Serialized for SerializedFile {
    fn get_serialized_file_version(&self) -> &SerializedFileFormatVersion {
        &self.header.version
    }

    fn get_data_offset(&self) -> u64 {
        self.header2.data_offset
    }

    fn get_endianess(&self) -> &Endian {
        &self.endianess
    }

    fn get_raw_object_by_index(&self, index: u32) -> Option<super::Object> {
        let obj = self.content.objects.get(index as usize)?;
        Some(super::Object {
            path_id: obj.path_id,
            byte_start: obj.byte_start,
            byte_size: obj.byte_size,
            class: ClassIDType::try_from(
                self.content
                    .types
                    .get(obj.type_id as usize)
                    .and_then(|t| Some(t.class_id))
                    .unwrap_or(0),
            )
            .unwrap_or(ClassIDType::Object),
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
pub struct SerializedFileContent {
    pub unity_version: NullString,
    pub target_platform: BuildTarget,
    pub enable_type_tree: U8Bool,
    type_count: u32,
    #[br(args { count: type_count as usize, inner:  SerializedTypeBinReadArgs::builder().enable_type_tree(*enable_type_tree).finalize() })]
    pub types: Vec<SerializedType>,
    pub object_count: i32,
    #[br(count = object_count)]
    pub objects: Vec<Object>,
    script_count: i32,
    #[br(count = script_count)]
    script_types: Vec<ScriptType>,
    externals_count: i32,
    #[br(count = externals_count)]
    externals: Vec<FileIdentifier>,
    ref_type_count: i32,
    #[br(args { count: ref_type_count as usize, inner: SerializedRefTypeBinReadArgs::builder().enable_type_tree(*enable_type_tree).finalize() })]
    ref_types: Vec<SerializedRefType>,
    user_information: NullString,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct Object {
    #[br(align_before(4))]
    pub path_id: i64,
    pub byte_start: u64,
    pub byte_size: u32,
    pub type_id: i32,
}
