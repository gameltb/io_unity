use super::EditorExtensionObject;
use crate::{classes::p_ptr::PPtr, BuildTarget, SerializedFileMetadata};
use binrw::binrw;

impl EditorExtensionObject for EditorExtension {}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct EditorExtension {
    #[brw(if(args.target_platform == BuildTarget::NoTarget),args_raw = args.clone())]
    prefab_parent_object: Option<PPtr>,
    #[brw(if(args.target_platform == BuildTarget::NoTarget),args_raw = args.clone())]
    prefab_internal: Option<PPtr>,
}
