pub mod transform;
pub mod type_tree;

use super::{component, p_ptr::PPtr};
use crate::{def_unity_class, unity_asset_view::UnityAssetViewer, SerializedFileMetadata};
use binrw::{BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};
use crc::{Crc, CRC_32_ISO_HDLC};
use glam::Mat4;
use std::{
    collections::BTreeMap,
    fmt,
    io::{Read, Seek, Write},
};
use supercow::Supercow;

pub const CRC_ISO_HDLC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

def_unity_class!(Transform, TransformObject);

pub trait TransformObject: fmt::Debug + component::DownCast {
    fn get_father(&self) -> Option<Supercow<PPtr>>;
    fn get_local_mat(&self) -> Option<Mat4>;
    fn get_children(&self) -> Option<Supercow<Vec<PPtr>>>;
}

impl BinRead for Transform {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        return Ok(Transform(Box::new(transform::Transform::read_options(
            reader, options, args,
        )?)));
    }
}

impl BinWrite for Transform {
    type Args = SerializedFileMetadata;

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Ok(())
    }
}

pub fn get_transform_path(
    viewer: &UnityAssetViewer,
    transform: &Transform,
) -> anyhow::Result<String> {
    let game_object = transform
        .downcast()
        .get_game_object()
        .unwrap()
        .get_type_tree_object_in_view(viewer)?
        .unwrap();
    if let Some(father) = transform.get_father() {
        if let Some(father) = father.get_type_tree_object_in_view(viewer)? {
            return Ok(get_transform_path(viewer, &Transform::new(father))?
                + "/"
                + &game_object.get_string_by_path("/Base/m_Name").unwrap());
        }
    } else {
        return Ok(game_object.get_string_by_path("/Base/m_Name").unwrap());
    }
    Ok(String::default())
}

pub fn get_bone_path_hash_map(
    viewer: &UnityAssetViewer,
    transform: &Transform,
) -> anyhow::Result<BTreeMap<u32, String>> {
    let mut map = BTreeMap::new();
    let mut path = get_transform_path(viewer, transform)?;

    let clc_crc = |path: &str| {
        let mut crc = CRC_ISO_HDLC.digest();
        crc.update(path.as_bytes());
        crc.finalize()
    };

    map.insert(clc_crc(&path), path.clone());

    while let Some((_, rpath)) = path.split_once("/") {
        path = rpath.to_string();
        map.insert(clc_crc(&path), path.clone());
    }
    if let Some(chilrens) = transform.get_children() {
        for chilren in &*chilrens {
            let chilren = chilren.get_type_tree_object_in_view(viewer)?.unwrap();
            let chilren = Transform::new(chilren);
            map.extend(get_bone_path_hash_map(viewer, &chilren)?);
        }
    }
    Ok(map)
}
