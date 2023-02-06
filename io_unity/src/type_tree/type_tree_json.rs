use super::{reader::TypeTreeObjectBinReadClassArgs, TypeField};
use crate::unityfs::UnityResource;
use once_cell::sync::Lazy;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use tar::Archive;

mod InfoJson {
    #![allow(non_snake_case)]

    use serde::{Deserialize, Serialize};

    #[derive(Default, Serialize, Deserialize)]
    pub struct InfoJson {
        pub Version: String,
        pub Strings: Vec<StringInfo>,
        pub Classes: Vec<Class>,
    }

    #[derive(Default, Serialize, Deserialize)]
    pub struct StringInfo {
        pub Index: i32,
        pub String: String,
    }

    #[derive(Default, Serialize, Deserialize)]
    pub struct Class {
        pub Name: String,
        pub FullName: String,
        pub TypeID: i32,
        pub Base: String,
        pub Derived: Vec<String>,
        pub DescendantCount: i32,
        pub Size: i32,
        pub TypeIndex: i32,
        pub IsAbstract: bool,
        pub IsSealed: bool,
        pub IsEditorOnly: bool,
        pub IsStripped: bool,
        pub ReleaseRootNode: Option<Node>,
        pub EditorRootNode: Option<Node>,
    }

    #[derive(Default, Serialize, Deserialize)]
    pub struct Node {
        pub TypeName: String,
        pub Name: String,
        pub Level: u8,
        pub ByteSize: i32,
        pub Index: i32,
        pub Version: u16,
        pub TypeFlags: u8,
        pub MetaFlag: i32,
        pub SubNodes: Vec<Node>,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeTreeNode {
    version: u16,
    level: u8,
    type_flags: u8,
    type_name: String,
    name: String,
    byte_size: i32,
    index: i32,
    meta_flag: i32,
}

impl TypeField for TypeTreeNode {
    fn get_version(&self) -> u16 {
        self.version
    }

    fn get_level(&self) -> u8 {
        self.level
    }

    fn is_array(&self) -> bool {
        self.type_flags & 1 > 0
    }

    fn get_byte_size(&self) -> i32 {
        self.byte_size
    }

    fn get_index(&self) -> i32 {
        self.index
    }

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

static INFO_JSON_TAR_READER: Lazy<Mutex<Option<Box<dyn UnityResource + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(None));
static INFO_JSON_CACHE_MAP: Lazy<Mutex<HashMap<String, InfoJson::InfoJson>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static TYPE_TREE_OBJECT_BIN_READ_CLASS_ARGS_CACHE_MAP: Lazy<
    Mutex<HashMap<String, BTreeMap<i32, TypeTreeObjectBinReadClassArgs>>>,
> = Lazy::new(|| Mutex::new(HashMap::new()));

/// The tar zstd compressed file contain type tree info json files
/// for read file without typetree info.
/// see https://github.com/DaZombieKiller/TypeTreeDumper
/// aslo https://github.com/AssetRipper/TypeTreeDumps.
/// File create by "tar -caf InfoJson.tar.zst InfoJson"
/// or "tar -c InfoJson | zstd --ultra -22 -o InfoJson.tar.zst"  
/// whitch can be less then 5MiB.
/// contain file path like /InfoJson/x.x.x.json.
pub fn set_info_json_tar_reader(reader: Box<dyn UnityResource + Send + Sync>) {
    if let Ok(mut info_json_tar_reader) = INFO_JSON_TAR_READER.lock() {
        *info_json_tar_reader = Some(reader)
    }
}

fn read_info_json_by_version(version: &String) -> anyhow::Result<InfoJson::InfoJson> {
    if let Ok(mut info_json_tar_reader) = INFO_JSON_TAR_READER.lock() {
        if let Some(ref mut info_json_tar_reader) = &mut *info_json_tar_reader {
            info_json_tar_reader.seek(std::io::SeekFrom::Start(0))?;
            let tar_reader = zstd::stream::read::Decoder::new(&mut *info_json_tar_reader)?;
            let mut tar = Archive::new(tar_reader);

            let json_path = format!("InfoJson/{version}.json");

            for file in tar.entries()? {
                let file = file?;

                if let Some(path) = file.header().path()?.to_str() {
                    if path == json_path {
                        // files implement the Read trait
                        let info_json: InfoJson::InfoJson = serde_json::from_reader(file)?;
                        return Ok(info_json);
                    }
                }
            }
        }
    }
    Err(anyhow!("cannot find json file for version {:?}", version))
}

pub fn get_type_object_args_by_version_class_id(
    version: &String,
    class_id: i32,
) -> Option<TypeTreeObjectBinReadClassArgs> {
    if let Ok(type_tree_object_bin_read_class_args_cache_map) =
        TYPE_TREE_OBJECT_BIN_READ_CLASS_ARGS_CACHE_MAP.lock()
    {
        if let Some(class_map) = type_tree_object_bin_read_class_args_cache_map.get(version) {
            if let Some(read_args) = class_map.get(&class_id) {
                return Some(read_args.clone());
            }
        }
    }

    if let Ok(mut info_json_cache_map) = INFO_JSON_CACHE_MAP.lock() {
        let info_json = if let Some(info_json) = info_json_cache_map.get(version) {
            info_json
        } else {
            let info_json = read_info_json_by_version(version).ok()?;
            info_json_cache_map.insert(version.clone(), info_json);
            info_json_cache_map.get(version).unwrap()
        };

        if let Some(class) = info_json
            .Classes
            .iter()
            .find(|class| class.TypeID == class_id)
        {
            if let Some(node) = &class.ReleaseRootNode {
                fn get_nodes(type_tree_nodes: &mut Vec<TypeTreeNode>, node: &InfoJson::Node) {
                    let type_tree_node = TypeTreeNode {
                        version: node.Version,
                        level: node.Level,
                        type_flags: node.TypeFlags,
                        type_name: node.TypeName.clone(),
                        name: node.Name.clone(),
                        byte_size: node.ByteSize,
                        index: node.Index,
                        meta_flag: node.MetaFlag,
                    };
                    type_tree_nodes.push(type_tree_node);
                    for node in &node.SubNodes {
                        get_nodes(type_tree_nodes, node);
                    }
                }
                let mut type_tree_nodes = Vec::new();
                get_nodes(&mut type_tree_nodes, node);
                type_tree_nodes.sort_by(|ttna, ttnb| ttna.index.cmp(&ttnb.index));

                let type_fields = type_tree_nodes
                    .into_iter()
                    .map(|ttn| Arc::new(Box::new(ttn) as Box<dyn TypeField + Send + Sync>))
                    .collect();
                let read_args = TypeTreeObjectBinReadClassArgs::new(class_id, type_fields);

                if let Ok(mut type_tree_object_bin_read_class_args_cache_map) =
                    TYPE_TREE_OBJECT_BIN_READ_CLASS_ARGS_CACHE_MAP.lock()
                {
                    if let Some(class_map) =
                        type_tree_object_bin_read_class_args_cache_map.get_mut(version)
                    {
                        class_map.insert(class_id, read_args.clone());
                    } else {
                        let mut class_map = BTreeMap::new();
                        class_map.insert(class_id, read_args.clone());
                        type_tree_object_bin_read_class_args_cache_map
                            .insert(version.clone(), class_map);
                    }
                }

                return Some(read_args);
            }
        }
    }
    None
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_json() {
        let time = std::time::Instant::now();

        let args = get_type_object_args_by_version_class_id(&"3.4.0".to_string(), 1);
        println!("{:?}", args);
        println!("Read use {:?}", time.elapsed());

        let args = get_type_object_args_by_version_class_id(&"3.4.0".to_string(), 2);
        println!("{:?}", args);
        println!("Read use {:?}", time.elapsed());
    }
}
