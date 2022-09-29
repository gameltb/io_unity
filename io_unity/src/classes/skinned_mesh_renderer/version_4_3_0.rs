use binrw::binrw;

use crate::classes::p_ptr::PPtr;
use crate::classes::renderer::Renderer;
use crate::until::binrw_parser::*;
use crate::SerializedFileMetadata;

use super::SkinnedMeshRendererObject;

impl SkinnedMeshRendererObject for SkinnedMeshRenderer {
    fn get_bones(&self) -> &Vec<PPtr> {
        &self.bones
    }

    fn get_mesh(&self) -> &PPtr {
        &self.mesh
    }

    fn get_materials(&self) -> &Vec<PPtr> {
        &self.renderer.get_materials()
    }
}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct SkinnedMeshRenderer {
    #[brw(args_raw = args.clone())]
    renderer: Renderer,
    quality: i32,
    update_when_offscreen: U8Bool,
    #[br(align_after(4))]
    skin_normals: U8Bool,
    #[brw(args_raw = args.clone())]
    mesh: PPtr,
    bones_size: i32,
    #[br(count = bones_size,args { inner: args })]
    #[bw(args_raw = args)]
    bones: Vec<PPtr>,
    num_blend_shape_weights: i32,
    #[br(count(num_blend_shape_weights))]
    blend_shape_weights: Vec<f32>,
}
