use std::borrow::Cow;
use std::convert::TryFrom;
use std::sync::Arc;

use binrw::{binrw, NullString};

use crate::classes::ClassIDType;
use crate::type_tree::{TypeField, TypeTreeObjectBinReadClassArgs};
use crate::until::Endian;
use crate::version13::{Object, ObjectBinReadArgs, ScriptType};
use crate::version17::FileIdentifier;
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
    unity_version: NullString,
    target_platform: BuildTarget,
    type_count: u32,
    #[br(count = type_count)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct TypeTreeNode {
    pub level: i32,
    pub type_name: String,
    pub name: String,
    pub byte_size: i32,
    pub index: i32,
    pub type_flags: i32,
    pub version: i32,
    pub meta_flag: i32,
}

impl TypeField for TypeTreeNode {
    fn get_version(&self) -> u16 {
        self.version as u16
    }

    fn get_level(&self) -> u8 {
        self.level as u8
    }
    //0x01 : IsArray
    //0x02 : IsRef
    //0x04 : IsRegistry
    //0x08 : IsArrayOfRefs
    fn is_array(&self) -> bool {
        self.type_flags & 1 > 0
    }

    fn get_byte_size(&self) -> i32 {
        self.byte_size
    }

    fn get_index(&self) -> i32 {
        self.index
    }

    //0x0001 : is invisible(?), set for m_FileID and m_PathID; ignored if no parent field exists or the type is neither ColorRGBA, PPtr nor string
    //0x0100 : ? is bool
    //0x1000 : ?
    //0x4000 : align bytes
    //0x8000 : any child has the align bytes flag
    //=> if flags & 0xC000 and size != 0xFFFFFFFF, the size field matches the total length of this field plus its children.
    //0x400000 : ?
    //0x800000 : ? is non-primitive type
    //0x02000000 : ? is UInt16 (called char)
    //0x08000000 : has fixed buffer size? related to Array (i.e. this field or its only child or its father is an array), should be set for vector, Array and the size and data fields.
    fn get_meta_flag(&self) -> i32 {
        self.meta_flag
    }

    fn is_align(&self) -> bool {
        self.meta_flag & 0x4000 > 0
    }

    fn get_ref_type_hash(&self) -> Option<u64> {
        None
    }

    fn get_type(&self) -> &String {
        &self.type_name
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}
