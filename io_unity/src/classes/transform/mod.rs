pub mod type_tree;

use crate::{def_unity_class, unity_asset_view::UnityAssetViewer};

use crc::{Crc, CRC_32_ISO_HDLC};
use glam::Mat4;
use std::{collections::BTreeMap, fmt};

use super::p_ptr::{PPtr, PPtrObject};

pub const CRC_ISO_HDLC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

def_unity_class!(Transform);

pub trait TransformObject: fmt::Debug {
    fn get_father(&self) -> Option<PPtr>;
    fn get_local_mat(&self) -> Option<Mat4>;
    fn get_children(&self) -> Option<Vec<PPtr>>;
}

pub fn get_transform_path(
    _viewer: &UnityAssetViewer,
    _transform: &Transform,
) -> anyhow::Result<String> {
    //        TypeTreeObject::try_cast_from(&self.inner, "/Base/m_GameObject")
    // .ok()
    // .and_then(|f| Some(PPtr::new(f)))
    // let game_object = transform
    //     .downcast()
    //     .get_game_object()
    //     .unwrap()
    //     .get_type_tree_object_in_view(viewer)?
    //     .unwrap();
    // if let Some(father) = transform.get_father() {
    //     if let Some(father) = father.get_type_tree_object_in_view(viewer)? {
    //         return Ok(get_transform_path(viewer, &Transform::new(father))?
    //             + "/"
    //             + &String::try_cast_from(&game_object, "/Base/m_Name").unwrap());
    //     }
    // } else {
    //     return Ok(String::try_cast_from(&game_object, "/Base/m_Name").unwrap());
    // }
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
