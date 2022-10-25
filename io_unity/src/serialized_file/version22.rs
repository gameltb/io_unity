use std::convert::TryFrom;
use std::fmt;

use std::io::prelude::*;
use std::sync::Arc;

use binrw::io::Cursor;
use binrw::{binrw, NullString};

use crate::classes::ClassIDType;
use crate::type_tree::{TypeField, TypeTreeObjectBinReadArgs};
use crate::until::{binrw_parser::*, Endian};
use crate::version17::read_type_tree_string;
use crate::{Serialized, SerializedFileFormatVersion};

use super::{BuildTarget, SerializedFileCommonHeader}; // reading/writing utilities

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

    fn get_raw_object_by_index(&self, index: u32) -> super::Object {
        let obj = self.content.objects.get(index as usize).unwrap();
        super::Object {
            path_id: obj.path_id,
            byte_start: obj.byte_start,
            byte_size: obj.byte_size,
            class: ClassIDType::try_from(obj.get_type(&self.content.types).class_id).unwrap(),
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

    fn get_type_object_args_by_type_id(&self, type_id: usize) -> TypeTreeObjectBinReadArgs {
        let stypetree = &self.content.types.get(type_id).unwrap();
        let type_tree = stypetree.type_tree.as_ref().unwrap();
        let mut type_fields = Vec::new();
        let mut string_reader = Cursor::new(&type_tree.string_buffer);

        for tp in &type_tree.type_tree_node_blobs {
            type_fields.push(Arc::new(Box::new(TypeTreeNode {
                name: tp.get_name_str(&mut string_reader),
                type_name: tp.get_type_str(&mut string_reader),
                node: tp.clone(),
            }) as Box<dyn TypeField + Send>))
        }

        TypeTreeObjectBinReadArgs::new(stypetree.class_id, type_fields)
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
    #[br(args { count: ref_type_count as usize, inner: SerializedTypeBinReadArgs { enable_type_tree:*enable_type_tree } })]
    ref_types: Vec<SerializedType>,
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
    type_dependencies_count: i32,
    #[br(count = type_dependencies_count)]
    type_dependencies: Vec<u32>,
}

#[binrw]
#[derive(Clone, PartialEq)]
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
        let mut string_reader = Cursor::new(&self.string_buffer);

        write!(f, "TypeTree [")?;
        if f.alternate() {
            write!(f, "\n")?;
        }
        for node in &self.type_tree_node_blobs {
            write!(
                f,
                "{:?} -> {{ type: {}, name: {} }},",
                node,
                node.get_type_str(&mut string_reader),
                node.get_name_str(&mut string_reader)
            )?;
            if f.alternate() {
                write!(f, "\n")?;
            }
        }
        write!(f, "]")
    }
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
struct TypeTreeNodeBlob {
    version: u16,
    level: u8,
    type_flags: u8,
    type_str_offset: u32,
    name_str_offset: u32,
    byte_size: i32,
    index: i32,
    meta_flag: i32,
    ref_type_hash: u64,
}

impl TypeTreeNodeBlob {
    fn get_type_str<R: Read + Seek>(&self, reader: &mut R) -> String {
        read_type_tree_string(self.type_str_offset, reader)
    }

    fn get_name_str<R: Read + Seek>(&self, reader: &mut R) -> String {
        read_type_tree_string(self.name_str_offset, reader)
    }
}

#[binrw]
#[derive(Debug, PartialEq)]
struct Object {
    #[br(align_before(4))]
    path_id: i64,
    byte_start: u64,
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

#[derive(Debug, PartialEq)]
pub struct TypeTreeNode {
    name: String,
    type_name: String,
    node: TypeTreeNodeBlob,
}

impl TypeField for TypeTreeNode {
    fn get_version(&self) -> u16 {
        self.node.version
    }

    fn get_level(&self) -> u8 {
        self.node.level
    }
    //0x01 : IsArray
    //0x02 : IsRef
    //0x04 : IsRegistry
    //0x08 : IsArrayOfRefs
    fn is_array(&self) -> bool {
        self.node.type_flags & 1 > 0
    }

    fn get_byte_size(&self) -> i32 {
        self.node.byte_size
    }

    fn get_index(&self) -> i32 {
        self.node.index
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
        self.node.meta_flag
    }

    fn is_align(&self) -> bool {
        self.node.meta_flag & 0x4000 > 0
    }

    fn get_ref_type_hash(&self) -> Option<u64> {
        Some(self.node.ref_type_hash)
    }

    fn get_type(&self) -> &String {
        &self.type_name
    }

    fn get_name(&self) -> &String {
        &self.name
    }
}
