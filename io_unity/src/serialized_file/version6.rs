use super::version11::{SerializedType, TypeTree, TypeTreeNode};
use super::version17::FileIdentifier;
use super::{BuildTarget, SerializedFileCommonHeader};
use super::{Serialized, SerializedFileFormatVersion};
use crate::type_tree::{reader::TypeTreeObjectBinReadClassArgs, TypeField};
use crate::until::Endian;
use binrw::{binrw, NullString};
use std::borrow::Cow;
use std::sync::Arc;

#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq)]
pub struct SerializedFile {
    header: SerializedFileCommonHeader,
    #[br(offset = (header.file_size - header.metadata_size) as u64)]
    endianess: Endian,
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
                path_id: obj.path_id as i64,
                byte_start: obj.byte_start as u64,
                byte_size: obj.byte_size,
                class: obj.class_id as i32,
                type_id: obj.type_id as usize,
            })
            .collect()
    }

    fn get_unity_version(&self) -> String {
        "".to_string()
    }

    fn get_target_platform(&self) -> &BuildTarget {
        &BuildTarget::UnknownPlatform
    }

    fn get_enable_type_tree(&self) -> bool {
        true
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
        let type_tree = &stypetree.type_tree;
        let mut type_fields = Vec::new();

        fn build_type_fields(
            type_fields: &mut Vec<Arc<Box<dyn TypeField + Send + Sync>>>,
            type_tree: &TypeTree,
        ) {
            type_fields.push(Arc::new(Box::new(TypeTreeNode {
                level: type_tree.level,
                type_name: type_tree.type_name.to_string(),
                name: type_tree.name.to_string(),
                byte_size: type_tree.byte_size,
                index: type_tree.index,
                type_flags: type_tree.type_flags,
                version: type_tree.version,
                meta_flag: type_tree.meta_flag,
            }) as Box<dyn TypeField + Send + Sync>));

            for tp in &type_tree.children {
                build_type_fields(type_fields, tp);
            }
        }
        build_type_fields(&mut type_fields, type_tree);
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
    type_count: u32,
    #[br(count = type_count)]
    types: Vec<SerializedType>,
    object_count: i32,
    #[br(count = object_count)]
    objects: Vec<Object>,
    externals_count: i32,
    #[br(count = externals_count)]
    externals: Vec<FileIdentifier>,
    user_information: NullString,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct Object {
    pub path_id: i32,
    pub byte_start: u32,
    pub byte_size: u32,
    pub type_id: i32,
    pub class_id: u16,
    pub is_destroyed: u16,
}
