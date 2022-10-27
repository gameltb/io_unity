use super::MeshFilterObject;
use crate::{
    classes::{component::Component, p_ptr::PPtr},
    SerializedFileMetadata,
};
use binrw::binrw;

impl MeshFilterObject for MeshFilter {}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct MeshFilter {
    #[brw(args_raw = args.clone())]
    component: Component,
    #[brw(args_raw = args)]
    mesh: PPtr,
}
