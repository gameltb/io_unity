use super::TransformObject;
use crate::{
    classes::{
        component::{self, Component, ComponentObject},
        p_ptr::PPtr,
    },
    until::binrw_parser::{Quat, Vec3},
    SerializedFileMetadata,
};
use binrw::binrw;
use glam::Mat4;
use supercow::Supercow;

impl component::DownCast for Transform {
    fn downcast<'a>(&'a self) -> Supercow<Box<dyn ComponentObject + Send + 'a>> {
        Supercow::borrowed(&*self.component)
    }
}

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

impl TransformObject for Transform {
    fn get_father(&self) -> Option<Supercow<PPtr>> {
        Some(Supercow::borrowed(&self.father))
    }

    fn get_local_mat(&self) -> Option<Mat4> {
        Some(Mat4::from_scale_rotation_translation(
            *self.local_scale,
            *self.local_rotation,
            *self.local_position,
        ))
    }

    fn get_children(&self) -> Option<Supercow<Vec<PPtr>>> {
        Some(Supercow::borrowed(&self.children))
    }
}
