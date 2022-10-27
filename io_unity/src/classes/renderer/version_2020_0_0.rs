use super::RendererObject;
use crate::classes::component::Component;
use crate::classes::p_ptr::PPtr;
use crate::until::binrw_parser::*;
use crate::SerializedFileMetadata;
use binrw::binrw;

impl RendererObject for Renderer {
    fn get_materials(&self) -> &Vec<PPtr> {
        &self.materials
    }
}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Renderer {
    #[brw(args_raw = args.clone())]
    component: Component,
    enabled: U8Bool,
    cast_shadows: u8,
    receive_shadows: u8,
    dynamic_occludee: u8,
    motion_vectors: u8,
    light_probe_usage: u8,
    reflection_probe_usage: u8,
    ray_tracing_mode: u8,
    ray_trace_procedural: u8,
    #[br(align_before(4))]
    rendering_layer_mask: u32,
    renderer_priority: i32,
    lightmap_index: u16,
    lightmap_index_dynamic: u16,
    lightmap_tiling_offset: Vec4,
    lightmap_tiling_offset_dynamic: Vec4,
    materials_size: i32,
    #[br(count = materials_size,args { inner: args.clone() })]
    #[bw(args_raw = args.clone())]
    materials: Vec<PPtr>,
    static_batch_info: StaticBatchInfo,
    #[brw(args_raw = args.clone())]
    static_batch_root: PPtr,
    #[brw(args_raw = args.clone())]
    probe_anchor: PPtr,
    #[brw(args_raw = args)]
    light_probe_volume_override: PPtr,
    sorting_layer_id: u32,
    #[br(align_after(4))]
    sorting_order: i16,
}

#[binrw]
#[derive(Debug)]
pub struct StaticBatchInfo {
    first_sub_mesh: u16,
    sub_mesh_count: u16,
}
