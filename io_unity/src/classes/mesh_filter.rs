use binrw::binrw;

use crate::SerializedFileMetadata;

use super::{component::Component, p_ptr::PPtr};

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct MeshFilter {
    #[brw(args_raw = args.clone())]
    component: Component,
    #[brw(args_raw = args)]
    mesh: PPtr,
}
