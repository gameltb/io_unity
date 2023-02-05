pub mod type_tree;

use crate::{
    def_unity_class,
    type_tree::{convert::TryCastFrom, TypeTreeObjectRef},
    unity_asset_view::UnityAssetViewer,
};

use crc::{Crc, CRC_32_ISO_HDLC};
use glam::Mat4;
use std::collections::BTreeMap;

use super::p_ptr::{PPtr, PPtrObject};

pub const CRC_ISO_HDLC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

def_unity_class!(Transform);

pub trait TransformObject {
    fn get_father(&self) -> anyhow::Result<TypeTreeObjectRef>;
    fn get_local_mat(&self) -> anyhow::Result<Mat4>;
    fn get_children(&self) -> anyhow::Result<Vec<TypeTreeObjectRef>>;
}

pub fn get_transform_path(
    viewer: &UnityAssetViewer,
    transform: &Transform,
) -> anyhow::Result<String> {
    let game_object_pptr = TypeTreeObjectRef::try_cast_from(transform.inner, "/Base/m_GameObject")?;
    let game_object = PPtr::new(&game_object_pptr).get_type_tree_object_in_view(viewer)?;
    if let Some(game_object) = game_object {
        if let Ok(father) = transform.get_father() {
            if let Some(father) = PPtr::new(&father).get_type_tree_object_in_view(viewer)? {
                return Ok(get_transform_path(viewer, &Transform::new(&father.into()))?
                    + "/"
                    + &String::try_cast_from(&game_object, "/Base/m_Name").unwrap());
            }
        } else {
            return Ok(String::try_cast_from(&game_object, "/Base/m_Name").unwrap());
        }
    }
    Ok(String::default())
}

pub fn get_bone_children_path_hash_map(
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

    while let Some((_, rpath)) = path.split_once('/') {
        path = rpath.to_string();
        map.insert(clc_crc(&path), path.clone());
    }
    if let Ok(chilrens) = transform.get_children() {
        for chilren in &*chilrens {
            let chilren = PPtr::new(chilren)
                .get_type_tree_object_in_view(viewer)?
                .unwrap();
            let chilren = chilren.into();
            let chilren = Transform::new(&chilren);
            map.extend(get_bone_children_path_hash_map(viewer, &chilren)?);
        }
    }
    Ok(map)
}

pub fn get_bone_path_hash_map(
    viewer: &UnityAssetViewer,
    transform: &Transform,
) -> anyhow::Result<BTreeMap<u32, String>> {
    let root_transform = get_root_bone(viewer, transform)?;
    let root_transform = Transform::new(&root_transform);
    get_bone_children_path_hash_map(viewer, &root_transform)
}

pub fn get_root_bone(
    viewer: &UnityAssetViewer,
    transform: &Transform,
) -> anyhow::Result<TypeTreeObjectRef> {
    let mut root_transform = transform.inner().clone();
    loop {
        if let Some(father) = PPtr::new(&Transform::new(&root_transform).get_father()?)
            .get_type_tree_object_in_view(viewer)?
        {
            root_transform = father.into();
        } else {
            break;
        }
    }
    Ok(root_transform)
}
