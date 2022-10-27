use super::AvatarObject;
use crate::until::binrw_parser::*;
use crate::SerializedFileMetadata;
use binrw::binrw;

impl AvatarObject for Avatar {}

#[binrw]
#[brw(import_raw(_args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Avatar {
    name: AlignedString,
    avatar_size: u32,
    avatar: AvatarConstant,
}

#[binrw]
#[derive(Debug)]
pub struct AvatarConstant {
    avatar_skeleton: Skeleton,
    avatar_skeleton_pose: SkeletonPose,
    default_pose: SkeletonPose,
    num_skeleton_name_id_array: i32,
    #[br(count(num_skeleton_name_id_array))]
    skeleton_name_idarray: Vec<u32>,
    human: Human,
    num_human_skeleton_index_array: i32,
    #[br(count(num_human_skeleton_index_array))]
    human_skeleton_index_array: Vec<i32>,
    // num_human_skeleton_reverse_index_array: i32,
    // #[br(count(num_human_skeleton_reverse_index_array))]
    // human_skeleton_reverse_index_array: Vec<i32>,
    root_motion_bone_index: i32,
    root_motion_bone_x: Xform,
    root_motion_skeleton: Skeleton,
    root_motion_skeleton_pose: SkeletonPose,
    num_root_motion_skeleton_index_array: i32,
    #[br(count(num_root_motion_skeleton_index_array))]
    root_motion_skeleton_index_array: Vec<i32>,
}

#[binrw]
#[derive(Debug)]
pub struct Skeleton {
    num_nodes: i32,
    #[br(count(num_nodes))]
    node: Vec<Node>,
    num_id: i32,
    #[br(count(num_id))]
    id: Vec<u32>,
    num_axes: i32,
    #[br(count(num_axes))]
    axes_array: Vec<Axes>,
}

#[binrw]
#[derive(Debug)]
pub struct Node {
    parent_id: i32,
    axes_id: i32,
}

#[binrw]
#[derive(Debug)]
pub struct Axes {
    pre_q: Vec4,
    post_q: Vec4,
    sgn: Vec3,
    limit: Limit,
    length: f32,
    r#type: u32,
}

#[binrw]
#[derive(Debug)]
pub struct Limit {
    min: Vec3,
    max: Vec3,
}

#[binrw]
#[derive(Debug)]
pub struct SkeletonPose {
    num_xforms: i32,
    #[br(count(num_xforms))]
    x: Vec<Xform>,
}

#[binrw]
#[derive(Debug)]
pub struct Xform {
    t: Vec3,
    q: Quat,
    s: Vec3,
}

#[binrw]
#[derive(Debug)]
pub struct Human {
    root_x: Xform,
    skeleton: Skeleton,
    skeleton_pose: SkeletonPose,
    left_hand: Hand,
    right_hand: Hand,
    num_human_bone_index: i32,
    #[br(count(num_human_bone_index))]
    human_bone_index: Vec<i32>,
    num_human_bone_mass: i32,
    #[br(count(num_human_bone_mass))]
    human_bone_mass: Vec<f32>,
    scale: f32,
    arm_twist: f32,
    fore_arm_twist: f32,
    upper_leg_twist: f32,
    leg_twist: f32,
    arm_stretch: f32,
    leg_stretch: f32,
    feet_spacing: f32,
    has_left_hand: f32,
    has_right_hand: U8Bool,
    #[br(align_after(4))]
    has_tdo_f: U8Bool,
}

#[binrw]
#[derive(Debug)]
pub struct Hand {
    num_hand_bone_index: i32,
    #[br(count(num_hand_bone_index))]
    hand_bone_index: Vec<i32>,
}

#[binrw]
#[derive(Debug)]
pub struct Handle {
    x: Xform,
    parent_human_index: u32,
    id: u32,
}
