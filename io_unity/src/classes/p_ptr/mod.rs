pub mod type_tree;
pub mod version13;
pub mod version14;

use crate::type_tree::TypeTreeObject;
use crate::unity_asset_view::UnityAssetViewer;
use crate::{def_unity_class, SerializedFile, SerializedFileMetadata};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use std::path::PathBuf;
use std::{
    fmt,
    io::{Read, Seek, Write},
};

def_unity_class!(PPtr, PPtrObject);

pub trait PPtrObject: fmt::Debug {
    fn get_path_id(&self) -> Option<i64>;
    fn get_file_id(&self) -> Option<i64>;
    fn get_serialized_file_id(&self) -> i64;

    fn get_serialized_file<'a>(
        &self,
        self_serialized_file: &'a SerializedFile,
        viewer: Option<&'a UnityAssetViewer>,
    ) -> Option<&'a SerializedFile> {
        if let Some(file_id) = self.get_file_id() {
            if file_id == 0 {
                return Some(self_serialized_file);
            }

            if let Some(viewer) = viewer {
                let externals = self_serialized_file.get_externals();

                if file_id > 0 {
                    if let Some(external) = externals.get(file_id as usize - 1) {
                        if let Some(file_name) = PathBuf::from(&external.path.to_string())
                            .file_name()
                            .map(|f| f.to_string_lossy().into_owned())
                        {
                            return viewer.get_serialized_file_by_path(&file_name);
                        }
                    }
                }
            }
        }
        None
    }

    fn get_type_tree_object(
        &self,
        self_serialized_file: &SerializedFile,
        viewer: Option<&UnityAssetViewer>,
    ) -> anyhow::Result<Option<TypeTreeObject>> {
        if let Some(path_id) = self.get_path_id() {
            if let Some(serialized_file) = self.get_serialized_file(self_serialized_file, viewer) {
                return serialized_file.get_tt_object_by_path_id(path_id);
            }
        }
        Ok(None)
    }

    fn get_type_tree_object_in_view(
        &self,
        viewer: &UnityAssetViewer,
    ) -> anyhow::Result<Option<TypeTreeObject>> {
        if let Some(self_serialized_file) = viewer
            .serialized_file_map
            .get(&self.get_serialized_file_id())
        {
            if let Some(path_id) = self.get_path_id() {
                if let Some(serialized_file) =
                    self.get_serialized_file(self_serialized_file, Some(viewer))
                {
                    return serialized_file.get_tt_object_by_path_id(path_id);
                }
            }
        }
        Ok(None)
    }
}

impl BinRead for PPtr {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        if args.version.clone() as i32 >= 14 {
            return Ok(PPtr(Box::new(version14::PPtr::read_options(
                reader, options, args,
            )?)));
        }
        Ok(PPtr(Box::new(version13::PPtr::read_options(
            reader, options, args,
        )?)))
    }
}

impl BinWrite for PPtr {
    type Args = SerializedFileMetadata;

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        todo!()
    }
}
