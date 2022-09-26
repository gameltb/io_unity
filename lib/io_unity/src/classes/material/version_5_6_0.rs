use binrw::binrw;

use crate::classes::p_ptr::PPtr;
use crate::until::binrw_parser::*;
use crate::SerializedFileMetadata;

use super::MaterialObject;

impl MaterialObject for Material {}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Material {
    name: AlignedString,
    #[brw(args_raw = args.clone())]
    shader: PPtr,
    shader_keywords: AlignedString,
    lightmap_flags: u32,
    enable_instancing_variants: U8Bool,
    #[br(align_before(4))]
    custom_render_queue: i32,
    string_tag_map_size: i32,
    #[br(count(string_tag_map_size))]
    string_tag_map: Vec<StringMap<AlignedString>>,
    num_disabled_shader_passes: i32,
    #[br(count(num_disabled_shader_passes))]
    disabled_shader_passes: Vec<AlignedString>,
    #[brw(args { args })]
    saved_properties: UnityPropertySheet,
}

#[binrw]
#[derive(Debug)]
pub struct StringMap<T: binrw::BinRead + binrw::BinWrite + 'static>
where
    T: binrw::BinRead<Args = ()>,
    T: binrw::BinWrite<Args = ()>,
{
    first: AlignedString,
    second: T,
}

#[binrw]
#[brw(import {args: SerializedFileMetadata})]
#[derive(Debug)]
pub struct StringMapUnityTexEnv {
    first: AlignedString,
    #[brw(args{args})]
    second: UnityTexEnv,
}

#[binrw]
#[brw(import {args: SerializedFileMetadata})]
#[derive(Debug)]
pub struct UnityTexEnv {
    #[brw(args_raw = args)]
    texture: PPtr,
    scale: Vec2,
    offset: Vec2,
}

#[binrw]
#[brw(import {args: SerializedFileMetadata})]
#[derive(Debug)]
pub struct UnityPropertySheet {
    tex_envs_size: i32,
    #[br(count(tex_envs_size),args { inner: StringMapUnityTexEnvBinReadArgs { args : args } })]
    #[bw(args { args : args })]
    tex_envs: Vec<StringMapUnityTexEnv>,
    floats_size: i32,
    #[br(count(floats_size))]
    floats: Vec<StringMap<f32>>,
    colors_size: i32,
    #[br(count(colors_size))]
    colors: Vec<StringMap<Vec4>>,
}
