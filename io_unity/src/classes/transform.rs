use binrw::binrw;
use glam::Mat4;

use crate::{
    until::binrw_parser::{Quat, Vec3},
    SerializedFileMetadata,
};

use super::{component::Component, p_ptr::PPtr};

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Transform {
    #[brw(args_raw = args.clone())]
    component: Component,
    local_rotation: Quat,
    local_position: Vec3,
    local_scale: Vec3,
    children_count: i32,
    #[br(count = children_count,args { inner: args.clone() })]
    #[bw(args_raw = args.clone())]
    children: Vec<PPtr>,
    #[brw(args_raw = args)]
    father: PPtr,
}

impl Transform {
    pub fn get_component(&self) -> &Component {
        &self.component
    }

    pub fn get_father(&self) -> &PPtr {
        &self.father
    }

    pub fn get_local_mat(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            *self.local_scale,
            *self.local_rotation,
            *self.local_position,
        )
    }
}
