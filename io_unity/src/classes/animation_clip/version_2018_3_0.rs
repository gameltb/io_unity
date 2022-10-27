use super::AnimationClipObject;
use crate::classes::p_ptr::PPtr;
use crate::until::binrw_parser::*;
use crate::SerializedFileMetadata;
use binrw::binrw;

impl AnimationClipObject for AnimationClip {
    fn get_name(&self) -> String {
        self.name.to_string()
    }
}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct AnimationClip {
    name: AlignedString,
    legacy: U8Bool,
    compressed: U8Bool,
    use_high_quality_curve: U8Bool,
    #[br(align_before(4))]
    num_rcurves: i32,
    #[br(count(num_rcurves))]
    rotation_curves: Vec<QuaternionCurve>,
    num_crcurves: i32,
    #[br(count(num_crcurves))]
    compressed_rotation_curves: Vec<CompressedAnimationCurve>,
    num_euler_curves: i32,
    #[br(count(num_euler_curves))]
    euler_curves: Vec<Vector3Curve>,
    num_pcurves: i32,
    #[br(count(num_pcurves))]
    position_curves: Vec<Vector3Curve>,
    num_scurves: i32,
    #[br(count(num_scurves))]
    scale_curves: Vec<Vector3Curve>,
    num_fcurves: i32,
    #[br(count(num_fcurves),args { inner: FloatCurveBinReadArgs { args : args.clone() } })]
    #[bw(args { args : args.clone() })]
    float_curves: Vec<FloatCurve>,
    num_ptr_curves: i32,
    #[br(count(num_ptr_curves),args { inner: PPtrCurveBinReadArgs { args : args.clone() } })]
    #[bw(args { args : args.clone()})]
    pptr_curves: Vec<PPtrCurve>,
    sample_rate: f32,
    wrap_mode: i32,
    bounds: AABB,
    muscle_clip_size: u32,
    muscle_clip: ClipMuscleConstant,
    #[brw(args { args : args })]
    clip_binding_constant: AnimationClipBindingConstant,
}

#[binrw]
#[derive(Debug)]
pub struct QuaternionCurve {
    curve: AnimationCurve<Quat>,
    path: AlignedString,
}

#[binrw]
#[derive(Debug)]
pub struct Vector3Curve {
    curve: AnimationCurve<Vec3>,
    path: AlignedString,
}

#[binrw]
#[brw(import { args: SerializedFileMetadata})]
#[derive(Debug)]
pub struct FloatCurve {
    curve: AnimationCurve<f32>,
    attribute: AlignedString,
    path: AlignedString,
    class_id: i32,
    #[brw(args_raw = args)]
    script: PPtr,
}

#[binrw]
#[brw(import { args: SerializedFileMetadata})]
#[derive(Debug)]
pub struct PPtrCurve {
    num_curves: i32,
    #[br(count(num_curves),args { inner: PPtrKeyframeBinReadArgs { args : args.clone() } })]
    #[bw(args { args : args.clone() })]
    curve: Vec<PPtrKeyframe>,
    attribute: AlignedString,
    path: AlignedString,
    class_id: i32,
    #[brw(args_raw = args)]
    script: PPtr,
}

#[binrw]
#[brw(import { args: SerializedFileMetadata})]
#[derive(Debug)]
pub struct PPtrKeyframe {
    time: f32,
    #[brw(args_raw = args)]
    value: PPtr,
}

#[binrw]
#[derive(Debug)]
pub struct AnimationCurve<T: binrw::BinRead + binrw::BinWrite + 'static>
where
    T: binrw::BinRead<Args = ()>,
    T: binrw::BinWrite<Args = ()>,
{
    num_curves: i32,
    #[br(count(num_curves))]
    curves: Vec<Keyframe<T>>,
    pre_infinity: i32,
    post_infinity: i32,
    rotation_order: i32,
}

#[binrw]
#[derive(Debug)]
pub struct Keyframe<T: binrw::BinRead + binrw::BinWrite + 'static>
where
    T: binrw::BinRead<Args = ()>,
    T: binrw::BinWrite<Args = ()>,
{
    time: f32,
    value: T,
    in_slope: T,
    out_slope: T,
    weighted_mode: i32,
    in_weight: T,
    out_weight: T,
}

#[binrw]
#[derive(Debug)]
pub struct CompressedAnimationCurve {
    path: AlignedString,
    times: PackedIntVector,
    values: PackedQuatVector,
    slopes: PackedFloatVector,
    pre_infinity: i32,
    post_infinity: i32,
}

#[binrw]
#[derive(Debug)]
pub struct ClipMuscleConstant {
    delta_pose: HumanPose,
    start_x: Xform,
    stop_x: Xform,
    left_foot_start_x: Xform,
    right_foot_start_x: Xform,
    average_speed: Vec3,
    clip: Clip,
    start_time: f32,
    stop_time: f32,
    orientation_offset_y: f32,
    level: f32,
    cycle_offset: f32,
    average_angular_speed: f32,
    num_index_array: i32,
    #[br(count(num_index_array))]
    index_array: Vec<i32>,
    num_deltas: i32,
    #[br(count(num_deltas))]
    value_array_delta: Vec<ValueDelta>,
    num_value_array_reference_pose: i32,
    #[br(count(num_value_array_reference_pose))]
    value_array_reference_pose: Vec<f32>,
    mirror: U8Bool,
    loop_time: U8Bool,
    loop_blend: U8Bool,
    loop_blend_orientation: U8Bool,
    loop_blend_position_y: U8Bool,
    loop_blend_position_xz: U8Bool,
    start_at_origin: U8Bool,
    keep_original_orientation: U8Bool,
    keep_original_position_y: U8Bool,
    keep_original_position_xz: U8Bool,
    #[br(align_after(4))]
    height_from_feet: U8Bool,
}

#[binrw]
#[derive(Debug)]
pub struct HumanPose {
    root_x: Xform,
    look_at_position: Vec3,
    look_at_weight: Vec4,
    num_goals: i32,
    #[br(count(num_goals))]
    goal_array: Vec<HumanGoal>,
    left_hand_pose: HandPose,
    right_hand_pose: HandPose,
    num_do_farray: i32,
    #[br(count(num_do_farray))]
    do_farray: Vec<f32>,
    num_tdof: i32,
    #[br(count(num_tdof))]
    tdo_farray: Vec<Vec3>,
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
pub struct HumanGoal {
    x: Xform,
    weight_t: f32,
    weight_r: f32,
    hint_t: Vec3,
    hint_weight_t: f32,
}

#[binrw]
#[derive(Debug)]
pub struct HandPose {
    grab_x: Xform,
    num_do_farray: i32,
    #[br(count(num_do_farray))]
    do_farray: Vec<f32>,
    r#override: f32,
    close_open: f32,
    in_out: f32,
    grab: f32,
}

#[binrw]
#[derive(Debug)]
pub struct Clip {
    streamed_clip: StreamedClip,
    dense_clip: DenseClip,
    constant_clip: ConstantClip,
}

#[binrw]
#[derive(Debug)]
pub struct StreamedClip {
    num_data: i32,
    #[br(count(num_data))]
    data: Vec<u32>,
    curve_count: u32,
}

#[binrw]
#[derive(Debug)]
pub struct DenseClip {
    frame_count: i32,
    curve_count: u32,
    sample_rate: f32,
    begin_time: f32,
    num_sample_array: i32,
    #[br(count(num_sample_array))]
    sample_array: Vec<f32>,
}

#[binrw]
#[derive(Debug)]
pub struct ConstantClip {
    num_data: i32,
    #[br(count(num_data))]
    data: Vec<f32>,
}

#[binrw]
#[derive(Debug)]
pub struct ValueDelta {
    start: f32,
    stop: f32,
}

#[binrw]
#[brw(import { args: SerializedFileMetadata})]
#[derive(Debug)]
pub struct AnimationClipBindingConstant {
    num_bindings: i32,
    #[br(count(num_bindings),args { inner: GenericBindingBinReadArgs { args : args.clone() } })]
    #[bw(args { args : args.clone() })]
    generic_bindings: Vec<GenericBinding>,
    num_mappings: i32,
    #[br(count = num_mappings,args { inner: args })]
    #[bw(args_raw = args)]
    pptr_curve_mapping: Vec<PPtr>,
}

#[binrw]
#[brw(import { args: SerializedFileMetadata})]
#[derive(Debug)]
pub struct GenericBinding {
    path: u32,
    attribute: u32,
    #[brw(args_raw = args)]
    script: PPtr,
    type_id: u32,
    custom_type: u8,
    #[br(align_after(4))]
    is_pptr_curve: u8,
}
