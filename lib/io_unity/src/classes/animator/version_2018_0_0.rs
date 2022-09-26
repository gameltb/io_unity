use binrw::binrw;

use crate::classes::behaviour::Behaviour;
use crate::classes::p_ptr::PPtr;
use crate::until::binrw_parser::*;
use crate::SerializedFileMetadata;

use super::AnimatorObject;

impl AnimatorObject for Animator {}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Animator {
    #[brw(args_raw = args.clone())]
    behaviour: Behaviour,
    #[brw(args_raw = args.clone())]
    avatar: PPtr,
    #[brw(args_raw = args)]
    controller: PPtr,
    culling_mode: i32,
    update_mode: i32,
    apply_root_motion: U8Bool,
    #[br(align_after(4))]
    linear_velocity_blending: U8Bool,
    has_transform_hierarchy: U8Bool,
    allow_constant_clip_sampling_optimization: U8Bool,
    keep_animator_controller_state_on_disable: U8Bool,
}
