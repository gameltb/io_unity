use std::{
    collections::{BTreeMap, HashMap},
    fs::OpenOptions,
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::type_tree::convert::TryCastFrom;
use crate::{
    classes::{p_ptr::PPtr, ClassIDType},
    type_tree::TypeTreeObject,
    SerializedFile, UnityFS, UnityResource,
};
use crate::{
    classes::{p_ptr::PPtrObject, SerializedFileRef},
    type_tree::TypeTreeObjectRef,
};

#[derive(Default)]
pub struct UnityAssetViewer {
    pub cab_maps: HashMap<String, i64>,
    pub serialized_file_map: BTreeMap<i64, SerializedFile>,
    serialized_file_count: i64,
    unity_fs_map: BTreeMap<i64, UnityFS>,
    unity_fs_count: i64,
    serialized_file_to_unity_fs_map: BTreeMap<i64, i64>,
    pub container_maps: HashMap<String, Vec<(i64, TypeTreeObjectRef)>>,
    container_name_maps: HashMap<i64, HashMap<i64, String>>,
}

impl UnityAssetViewer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_bundle_dir<P: AsRef<Path>>(&mut self, dir_path: P) -> anyhow::Result<()> {
        for entry in WalkDir::new(dir_path).into_iter().flatten() {
            if entry.file_type().is_file() {
                let file = OpenOptions::new().read(true).open(entry.path())?;
                let file = Box::new(BufReader::new(file));
                let _unity_fs_id = self
                    .add_bundle_file(
                        file,
                        Some(entry.path().parent().unwrap().to_string_lossy().to_string()),
                    )
                    .unwrap_or_default();
            }
        }
        Ok(())
    }

    pub fn add_bundle_file(
        &mut self,
        bundle_file_reader: Box<dyn UnityResource + Send + Sync>,
        resource_search_path: Option<String>,
    ) -> anyhow::Result<i64> {
        let unity_fs = UnityFS::read(bundle_file_reader, resource_search_path)?;
        let unity_fs_id = self.unity_fs_count;
        self.unity_fs_count += 1;
        for cab_path in unity_fs.get_cab_path() {
            let cab_buff = unity_fs.get_file_data_by_path(&cab_path)?;
            let cab_buff_reader = Box::new(Cursor::new(cab_buff));
            // let cab_buff_reader = Box::new(BufReader::new(
            //     unity_fs
            //         .get_file_reader_by_path(&cab_path)
            //         .ok_or(anyhow!("can not get cab reader"))?,
            // ));

            let serialized_file_id = self.add_serialized_file(cab_buff_reader, None)?;
            self.serialized_file_to_unity_fs_map
                .insert(serialized_file_id, unity_fs_id);
            self.cab_maps.insert(cab_path, serialized_file_id);
        }
        self.unity_fs_map.insert(unity_fs_id, unity_fs);
        Ok(unity_fs_id)
    }

    pub fn add_serialized_file(
        &mut self,
        serialized_file_reader: Box<dyn UnityResource + Send + Sync>,
        resource_search_path: Option<String>,
    ) -> anyhow::Result<i64> {
        let serialized_file_id = self.serialized_file_count;
        self.serialized_file_count += 1;

        let serialized_file = SerializedFile::read(
            serialized_file_reader,
            serialized_file_id,
            resource_search_path,
        )?;
        if let Ok(Some(asset_bundle)) = serialized_file.get_tt_object_by_path_id(1) {
            if let Ok(containers) = <HashMap<String, TypeTreeObjectRef>>::try_cast_from(
                &asset_bundle.into(),
                "/Base/m_Container/Array",
            ) {
                let mut name_map = HashMap::new();
                for (name, asset_info) in containers {
                    if let Ok(pptr) = TypeTreeObjectRef::try_cast_from(&asset_info, "/Base/asset") {
                        if let Ok(path_id) = PPtr::new(&pptr).get_path_id() {
                            name_map.insert(path_id, name.clone());
                        }

                        if let Some(objs) = self.container_maps.get_mut(&name) {
                            objs.push((serialized_file_id, pptr));
                        } else {
                            self.container_maps
                                .insert(name, vec![(serialized_file_id, pptr)]);
                        }
                    }
                }
                self.container_name_maps
                    .insert(serialized_file_id, name_map);
            }
        }

        for (path_id, obj) in serialized_file.get_object_map() {
            if obj.class == ClassIDType::ResourceManager as i32 {
                if let Ok(Some(resource_manager)) =
                    serialized_file.get_tt_object_by_path_id(*path_id)
                {
                    if let Ok(containers) = <HashMap<String, TypeTreeObjectRef>>::try_cast_from(
                        &resource_manager.into(),
                        "/Base/m_Container/Array",
                    ) {
                        let mut name_map = HashMap::new();
                        for (name, pptr) in containers {
                            if let Ok(path_id) = PPtr::new(&pptr).get_path_id() {
                                name_map.insert(path_id, name.clone());
                            }

                            if let Some(objs) = self.container_maps.get_mut(&name) {
                                objs.push((serialized_file_id, pptr));
                            } else {
                                self.container_maps
                                    .insert(name, vec![(serialized_file_id, pptr)]);
                            }
                        }
                        self.container_name_maps
                            .insert(serialized_file_id, name_map);
                    }
                }
            }
        }

        self.serialized_file_map
            .insert(serialized_file_id, serialized_file);
        Ok(serialized_file_id)
    }

    pub fn read_data_dir<P: AsRef<Path>>(&mut self, data_dir_path: P) -> anyhow::Result<()> {
        for i in 0..u8::MAX {
            let file_name = format!("level{i}");
            if let Ok(file) = OpenOptions::new()
                .read(true)
                .open(data_dir_path.as_ref().join(&file_name))
            {
                let serialized_file_id = self.add_serialized_file(
                    Box::new(BufReader::new(file)),
                    Some(data_dir_path.as_ref().to_string_lossy().to_string()),
                )?;
                self.cab_maps.insert(file_name, serialized_file_id);
            } else {
                break;
            }
        }
        for i in 0..u8::MAX {
            let file_name = format!("sharedassets{i}.assets");
            if let Ok(file) = OpenOptions::new()
                .read(true)
                .open(data_dir_path.as_ref().join(&file_name))
            {
                let serialized_file_id = self.add_serialized_file(
                    Box::new(BufReader::new(file)),
                    Some(data_dir_path.as_ref().to_string_lossy().to_string()),
                )?;
                self.cab_maps.insert(file_name, serialized_file_id);
            } else {
                break;
            }
        }

        let file_names = [
            "resources.assets",
            "globalgamemanagers.assets",
            "globalgamemanagers",
        ];
        for file_name in file_names {
            if let Ok(file) = OpenOptions::new()
                .read(true)
                .open(data_dir_path.as_ref().join(file_name))
            {
                let serialized_file_id = self.add_serialized_file(
                    Box::new(BufReader::new(file)),
                    Some(data_dir_path.as_ref().to_string_lossy().to_string()),
                )?;
                self.cab_maps
                    .insert(file_name.to_owned(), serialized_file_id);
            }
        }
        Ok(())
    }

    pub fn get_serialized_file_by_path(&self, path: &String) -> Option<&SerializedFile> {
        if let Some(serialized_file_id) = self.cab_maps.get(path) {
            if let Some(serialized_file) = self.serialized_file_map.get(serialized_file_id) {
                return Some(serialized_file);
            }
        }
        None
    }

    pub fn get_unity_fs_by_cab_path(&self, path: &String) -> Option<&UnityFS> {
        if let Some(serialized_file_id) = self.cab_maps.get(path) {
            if let Some(unity_fs_id) = self.serialized_file_to_unity_fs_map.get(serialized_file_id)
            {
                if let Some(unity_fs) = self.unity_fs_map.get(unity_fs_id) {
                    return Some(unity_fs);
                }
            }
        }
        None
    }

    pub fn get_unity_fs_by_serialized_file(
        &self,
        serialized_file: &SerializedFile,
    ) -> Option<&UnityFS> {
        if let Some(unity_fs_id) = self
            .serialized_file_to_unity_fs_map
            .get(&serialized_file.get_serialized_file_id())
        {
            if let Some(unity_fs) = self.unity_fs_map.get(unity_fs_id) {
                return Some(unity_fs);
            }
        }
        None
    }

    pub fn get_unity_fs_by_pptr(&self, pptr: &PPtr) -> Option<&UnityFS> {
        let serialized_file_id = pptr.get_serialized_file_id();
        if let Some(unity_fs_id) = self
            .serialized_file_to_unity_fs_map
            .get(&serialized_file_id)
        {
            if let Some(unity_fs) = self.unity_fs_map.get(unity_fs_id) {
                return Some(unity_fs);
            }
        }

        None
    }

    pub fn get_unity_fs_by_type_tree_object(
        &self,
        type_tree_object: &TypeTreeObject,
    ) -> Option<&UnityFS> {
        if let Some(unity_fs_id) = self
            .serialized_file_to_unity_fs_map
            .get(&type_tree_object.serialized_file_id)
        {
            if let Some(unity_fs) = self.unity_fs_map.get(unity_fs_id) {
                return Some(unity_fs);
            }
        }
        None
    }

    pub fn get_container_name_by_path_id(
        &self,
        cab_name: &String,
        path_id: i64,
    ) -> Option<&String> {
        if let Some(serialized_file_id) = self.cab_maps.get(cab_name) {
            if let Some(name_map) = self.container_name_maps.get(serialized_file_id) {
                return name_map.get(&path_id);
            }
        }
        None
    }

    pub fn get_container_name_by_serialized_file_id_and_path_id(
        &self,
        serialized_file_id: i64,
        path_id: i64,
    ) -> Option<&String> {
        if let Some(name_map) = self.container_name_maps.get(&serialized_file_id) {
            return name_map.get(&path_id);
        }
        None
    }

    pub fn get_container_name_by_pptr(&self, pptr: &PPtr) -> Option<&String> {
        let serialized_file_id = pptr.get_serialized_file_id();
        if let Some(name_map) = self.container_name_maps.get(&serialized_file_id) {
            if let Ok(path_id) = pptr.get_path_id() {
                return name_map.get(&path_id);
            }
        }
        None
    }

    pub fn get_type_tree_object_by_container_name(
        &self,
        container_name: &String,
    ) -> anyhow::Result<Option<TypeTreeObject>> {
        if let Some(serialized_file_id) = self.container_maps.get(container_name) {
            if let Some((serialized_file_id, pptr)) = serialized_file_id.get(0) {
                if let Some(serialized_file) = self.serialized_file_map.get(serialized_file_id) {
                    return PPtr::new(pptr).get_type_tree_object(serialized_file, Some(self));
                }
            }
        }
        Ok(None)
    }

    pub fn get_serialized_file_by_container_name(
        &self,
        container_name: &String,
    ) -> Option<&SerializedFile> {
        if let Some(serialized_file_id) = self.container_maps.get(container_name) {
            if let Some((serialized_file_id, _pptr)) = serialized_file_id.get(0) {
                return self.serialized_file_map.get(serialized_file_id);
            }
        }
        None
    }

    pub fn get_resource_file_by_serialized_file_id_and_path(
        &self,
        serialized_file_id: i64,
        path: &String,
    ) -> Option<Box<dyn UnityResource>> {
        get_resource_file_by_path(
            path,
            self.serialized_file_map.get(&serialized_file_id),
            self.serialized_file_to_unity_fs_map
                .get(&serialized_file_id)
                .and_then(|fs_id| self.unity_fs_map.get(fs_id)),
            None,
        )
    }
}

pub fn get_resource_file_by_path(
    path: &String,
    serialized_file: Option<&SerializedFile>,
    unityfs: Option<&UnityFS>,
    search_path: Option<&String>,
) -> Option<Box<dyn UnityResource>> {
    if let Some(file_name) = PathBuf::from(&path)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
    {
        if path.starts_with("archive:/") {
            if let Some(unityfs) = unityfs {
                if let Some(file_reader) = unityfs.get_file_reader_by_path(&file_name) {
                    return Some(Box::new(file_reader));
                }
            }
        } else {
            if let Some(search_path) = search_path {
                let path = PathBuf::from(search_path).join(&file_name);
                if let Ok(file) = OpenOptions::new().read(true).open(path) {
                    return Some(Box::new(BufReader::new(file)));
                }
            }
            if let Some(serialized_file) = serialized_file {
                if let Some(search_path) = &serialized_file.resource_search_path {
                    let path = PathBuf::from(search_path).join(&file_name);
                    if let Ok(file) = OpenOptions::new().read(true).open(path) {
                        return Some(Box::new(BufReader::new(file)));
                    }
                }
            }
            if let Some(unityfs) = unityfs {
                if let Some(search_path) = &unityfs.resource_search_path {
                    let path = PathBuf::from(search_path).join(&file_name);
                    if let Ok(file) = OpenOptions::new().read(true).open(path) {
                        return Some(Box::new(BufReader::new(file)));
                    }
                }
            }
            let path = PathBuf::from(".").join(&file_name);
            if let Ok(file) = OpenOptions::new().read(true).open(path) {
                return Some(Box::new(BufReader::new(file)));
            }
        }
    }

    None
}
