use binrw::binrw;

use crate::{BuildTarget, SerializedFileMetadata};

use super::p_ptr::PPtr;

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct EditorExtension {
    #[brw(if(args.target_platform == BuildTarget::NoTarget),args_raw = args.clone())]
    prefab_parent_object: Option<PPtr>,
    #[brw(if(args.target_platform == BuildTarget::NoTarget),args_raw = args.clone())]
    prefab_internal: Option<PPtr>,
}
