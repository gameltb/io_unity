use std::convert::TryFrom;
use std::sync::Arc;

use binrw::io::Cursor;
use binrw::{binrw, NullString};

use crate::classes::ClassIDType;
use crate::type_tree::{TypeField, TypeTreeObjectBinReadArgs};
use crate::until::Endian;
use crate::version13::{Object, ObjectBinReadArgs};
use crate::version17::{FileIdentifier, TypeTree, TypeTreeNode};
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

    fn get_raw_object_by_index(&self, index: u32) -> super::Object {
        let obj = self.content.objects.get(index as usize).unwrap();
        super::Object {
            path_id: obj.path_id,
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
        self.content.unity_version.to_string()
    }

    fn get_target_platform(&self) -> &BuildTarget {
        &self.content.target_platform
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
        let mut string_reader = Cursor::new(&type_tree.string_buffer);

        for tp in &type_tree.type_tree_node_blobs {
            type_fields.push(Arc::new(Box::new(TypeTreeNode {
                name: tp.get_name_str(&mut string_reader),
                type_name: tp.get_type_str(&mut string_reader),
                node: tp.clone(),
            }) as Box<dyn TypeField + Send + Sync>))
        }

        TypeTreeObjectBinReadArgs::new(stypetree.class_id, type_fields)
    }
}

#[binrw]
#[derive(Debug, PartialEq)]
struct SerializedFileContent {
    unity_version: NullString,
    target_platform: BuildTarget,
    type_count: u32,
    #[br(count = type_count)]
    types: Vec<SerializedType>,
    big_id_enabled: i32,
    object_count: i32,
    #[br(args { count: object_count as usize, inner: ObjectBinReadArgs::builder().big_id_enabled(big_id_enabled != 0).finalize() })]
    objects: Vec<Object>,
    externals_count: i32,
    #[br(count = externals_count)]
    externals: Vec<FileIdentifier>,
    user_information: NullString,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct SerializedType {
    pub class_id: i32,
    pub type_tree: TypeTree,
}
