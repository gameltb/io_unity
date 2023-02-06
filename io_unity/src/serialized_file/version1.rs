use super::version11::TypeTreeNode;
use super::version4::FileIdentifier;
use super::version6::Object;
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
    #[br(seek_before = std::io::SeekFrom::Start((header.file_size - header.metadata_size) as u64))]
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
    fn get_externals(&self) -> Cow<Vec<super::version17::FileIdentifier>> {
        let externals = self
            .content
            .externals
            .iter()
            .map(|o| super::version17::FileIdentifier {
                temp_empty: NullString::default(),
                guid: [0u8; 16],
                r#type: 0,
                path: o.path.clone(),
            })
            .collect();
        return Cow::Owned(externals);
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
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct SerializedType {
    pub class_id: i32,
    pub type_tree: TypeTree,
}

#[binrw]
#[br(import { level: i32 = 0})]
#[derive(Debug, Clone, PartialEq)]
pub struct TypeTree {
    #[br(calc = level)]
    pub level: i32,
    pub type_name: NullString,
    pub name: NullString,
    pub byte_size: i32,
    pub index: i32,
    pub type_flags: i32,
    pub version: i32,
    pub meta_flag: i32,
    pub children_count: i32,
    #[br(args { count: children_count as usize, inner: TypeTreeBinReadArgs::builder().level(level + 1).finalize() })]
    pub children: Vec<TypeTree>,
}
