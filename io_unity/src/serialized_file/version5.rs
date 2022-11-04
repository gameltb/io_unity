use std::convert::TryFrom;
use std::sync::Arc;

use binrw::{binrw, NullString};

use crate::classes::ClassIDType;
use crate::type_tree::{TypeField, TypeTreeObjectBinReadArgs};
use crate::until::Endian;
use crate::version11::{SerializedType, TypeTree, TypeTreeNode};
use crate::version6::Object;
use crate::{Serialized, SerializedFileFormatVersion};

use super::{BuildTarget, SerializedFileCommonHeader};

#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq)]
pub struct SerializedFile {
    header: SerializedFileCommonHeader,
    #[br(offset =(header.file_size - header.metadata_size) as u64)]
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

    fn get_raw_object_by_index(&self, index: u32) -> super::Object {
        let obj = self.content.objects.get(index as usize).unwrap();
        super::Object {
            path_id: obj.path_id as i64,
            byte_start: obj.byte_start as u64,
            byte_size: obj.byte_size,
            class: ClassIDType::try_from(obj.class_id as i32).unwrap(),
            type_id: obj.type_id as usize,
        }
    }

    fn get_object_count(&self) -> i32 {
        self.content.object_count
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

    fn get_type_object_args_by_type_id(&self, type_id: usize) -> TypeTreeObjectBinReadArgs {
        let stypetree = self
            .content
            .types
            .iter()
            .find(|tp| tp.class_id == type_id as i32)
            .unwrap();
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
        TypeTreeObjectBinReadArgs::new(stypetree.class_id, type_fields)
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
pub struct FileIdentifier {
    guid: [u8; 16],
    r#type: i32,
    path: NullString,
}
