use std::borrow::Cow;
use std::convert::TryFrom;
use std::sync::Arc;

use binrw::{binrw, NullString};

use crate::classes::ClassIDType;
use crate::type_tree::{TypeField, TypeTreeObjectBinReadClassArgs};
use crate::until::Endian;
use crate::version11::TypeTreeNode;
use crate::version4::FileIdentifier;
use crate::version6::Object;
use crate::{Serialized, SerializedFileFormatVersion};

use super::{BuildTarget, SerializedFileCommonHeader};

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

    fn get_raw_object_by_index(&self, index: u32) -> Option<super::Object> {
        let obj = self.content.objects.get(index as usize)?;
        Some(super::Object {
            path_id: obj.path_id as i64,
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
                index: 0,
                type_flags: type_tree.type_flags,
                version: type_tree.version,
                meta_flag: 0,
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
    fn get_externals(&self) -> Cow<Vec<crate::version17::FileIdentifier>> {
        let externals = self
            .content
            .externals
            .iter()
            .map(|o| crate::version17::FileIdentifier {
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
    pub type_flags: i32,
    pub version: i32,
    pub children_count: i32,
    #[br(args { count: children_count as usize, inner: TypeTreeBinReadArgs::builder().level(level + 1).finalize() })]
    pub children: Vec<TypeTree>,
}
