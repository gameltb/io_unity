extern crate io_unity;

use std::{
    collections::{BTreeMap, HashMap},
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
};

use clap::{arg, Parser, Subcommand};
use io_unity::{
    classes::{
        animation_clip::{
            animation_clip_binding_constant_find_binding, streamed_clip_read_u32_buff,
        },
        p_ptr::PPtr,
        texture2d::Texture2D,
        transform::{get_bone_path_hash_map, Transform},
        ClassIDType,
    },
    type_tree::{type_tree_json::set_info_json_tar_path, TypeTreeObject},
    unity_asset_view::UnityAssetViewer,
};

mod CubismModel3Json {
    #![allow(non_snake_case)]

    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct CubismModel3Json {
        pub Version: i32,

        pub FileReferences: SerializableFileReferences,

        pub Groups: Vec<SerializableGroup>,

        pub HitAreas: Vec<SerializableHitArea>,
    }

    impl CubismModel3Json {
        pub fn new() -> Self {
            let mut new = Self::default();
            new.Version = 3;
            new
        }
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableFileReferences {
        pub Moc: String,

        pub Textures: Vec<String>,

        pub Pose: String,

        pub Expressions: Vec<SerializableExpression>,

        pub Motions: HashMap<String, Vec<SerializableMotion>>,

        pub Physics: String,

        pub UserData: String,

        pub DisplayInfo: String,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableGroup {
        pub Target: String,

        pub Name: String,

        pub Ids: Vec<String>,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableExpression {
        pub Name: String,

        pub File: String,

        pub FadeInTime: f32,

        pub FadeOutTime: f32,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableMotions {
        pub GroupNames: Vec<String>,

        pub Motions: Vec<Vec<SerializableMotion>>,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableMotion {
        pub File: String,

        pub Sound: String,

        pub FadeInTime: f32,

        pub FadeOutTime: f32,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableHitArea {
        pub Name: String,

        pub Id: String,
    }
}

mod CubismPhysics3Json {
    #![allow(non_snake_case)]

    use serde::{Deserialize, Serialize};

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct CubismPhysics3Json {
        pub Version: i32,

        pub Meta: SerializableMeta,

        pub PhysicsSettings: Vec<SerializablePhysicsSettings>,
    }

    impl CubismPhysics3Json {
        pub fn new() -> Self {
            let mut new = Self::default();
            new.Version = 3;
            new
        }
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableVector2 {
        pub X: f32,

        pub Y: f32,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableNormalizationValue {
        pub Minimum: f32,

        pub Default: f32,

        pub Maximum: f32,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableParameter {
        pub Target: String,

        pub Id: String,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableInput {
        pub Source: SerializableParameter,

        pub Weight: f32,

        pub Type: String,

        pub Reflect: bool,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableOutput {
        pub Destination: SerializableParameter,

        pub VertexIndex: i32,

        pub Scale: f32,

        pub Weight: f32,

        pub Type: String,

        pub Reflect: bool,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableVertex {
        pub Position: SerializableVector2,

        pub Mobility: f32,

        pub Delay: f32,

        pub Acceleration: f32,

        pub Radius: f32,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableNormalization {
        pub Position: SerializableNormalizationValue,

        pub Angle: SerializableNormalizationValue,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializablePhysicsSettings {
        pub Id: String,

        pub Input: Vec<SerializableInput>,

        pub Output: Vec<SerializableOutput>,

        pub Vertices: Vec<SerializableVertex>,

        pub Normalization: SerializableNormalization,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableMeta {
        pub PhysicsSettingCount: i32,

        pub TotalInputCount: i32,

        pub TotalOutputCount: i32,

        pub TotalVertexCount: i32,

        pub EffectiveForces: SerializableEffectiveForces,

        pub Fps: f32,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableEffectiveForces {
        pub Gravity: SerializableVector2,

        pub Wind: SerializableVector2,
    }
}

mod CubismMotion3Json {
    #![allow(non_snake_case)]

    use serde::{Deserialize, Serialize};

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    // ReSharper disable once ClassCannotBeInstantiated
    pub struct CubismMotion3Json {
        pub Version: i32,

        pub Meta: SerializableMeta,

        pub Curves: Vec<SerializableCurve>,

        pub UserData: Vec<SerializableUserData>,
    }

    impl CubismMotion3Json {
        pub fn new() -> Self {
            let mut new = Self::default();
            new.Version = 3;
            new
        }
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableMeta {
        pub Duration: f32,

        pub Fps: f32,

        pub Loop: bool,

        pub CurveCount: i32,

        pub TotalSegmentCount: i32,

        pub TotalPointCount: i32,

        pub AreBeziersRestricted: bool,

        pub UserDataCount: i32,

        pub TotalUserDataSize: i32,

        pub FadeInTime: f32,

        pub FadeOutTime: f32,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableCurve {
        pub Target: String,

        pub Id: String,

        pub Segments: Vec<f32>,

        pub FadeInTime: f32,

        pub FadeOutTime: f32,
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableUserData {
        pub Time: f32,

        pub Value: String,
    }
}

mod CubismExp3Json {
    #![allow(non_snake_case)]

    use serde::{Deserialize, Serialize};

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct CubismExp3Json {
        pub Type: String,

        pub FadeInTime: f32,

        pub FadeOutTime: f32,

        pub Parameters: Vec<SerializableExpressionParameter>,
    }

    impl CubismExp3Json {
        pub fn new() -> Self {
            let mut new = Self::default();
            new.Type = "Live2D Expression".to_owned();
            new.FadeInTime = 1.0;
            new.FadeOutTime = 1.0;
            new
        }
    }

    #[derive(Default, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SerializableExpressionParameter {
        pub Id: String,

        pub Value: f32,

        pub Blend: String,
    }
}

/// live2d extractor
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// The dir contain AssetBundle files.
    #[arg(short, long)]
    bundle_dir: String,
    /// The tar zstd compressed file contain type tree info json files
    /// for read file without typetree info.
    /// see https://github.com/DaZombieKiller/TypeTreeDumper
    /// aslo https://github.com/AssetRipper/TypeTreeDumps.
    /// File create by "tar -caf InfoJson.tar.zst InfoJson"
    /// or "tar -c InfoJson | zstd --ultra -22 -o InfoJson.tar.zst"  
    /// whitch can be less then 5MiB.
    /// contain file path like /InfoJson/x.x.x.json.
    #[arg(short, long)]
    info_json_tar_path: Option<String>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List container path
    List {
        /// filter path
        #[arg(value_parser)]
        filter_path: Option<String>,
    },
    /// Extract one model, Assets under the filter path will be treated as one model.
    Extract {
        /// filter path
        #[arg(value_parser)]
        filter_path: String,
    },
}

#[derive(Default)]
struct KeyframeCurve {
    target: String,
    id: String,
    keyframes: Vec<Keyframe>,
}

#[derive(Default)]
struct Keyframe {
    time: f32,
    value: f32,
    in_slope: f32,
    out_slope: f32,
    coeff: [f32; 4],
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(path) = args.info_json_tar_path {
        set_info_json_tar_path(path);
    }

    let time = std::time::Instant::now();

    let mut unity_asset_viewer = UnityAssetViewer::new();
    unity_asset_viewer.read_bundle_dir(args.bundle_dir)?;

    println!("Read use {:?}", time.elapsed());

    match &args.command {
        Commands::List { filter_path } => {
            for (container_path, _) in &unity_asset_viewer.container_maps {
                if let Some(filter_path) = filter_path {
                    if container_path.starts_with(filter_path) {
                        println!("{}", container_path);
                    }
                } else {
                    println!("{}", container_path);
                }
            }
        }
        Commands::Extract { filter_path } => {
            let mut container_paths = Vec::new();
            let mut path_hash_map = BTreeMap::new();

            let mut cubism_model3_json = CubismModel3Json::CubismModel3Json::new();
            let mut cubism_model3_dir_path_and_name = None;
            let mut cubism_motion3_json_map = HashMap::new();
            let mut cubism_physics3_json = None;
            let mut cubism_exp3_json_map = HashMap::new();

            for (container_path, _) in &unity_asset_viewer.container_maps {
                if container_path.starts_with(filter_path) {
                    println!("Get {}", container_path);
                    let obj = unity_asset_viewer
                        .get_type_tree_object_by_container_name(container_path)?
                        .unwrap();
                    container_paths.push((container_path, obj));
                }
            }

            for (container_path, obj) in &container_paths {
                if obj.class_id == ClassIDType::GameObject as i32 {
                    println!("Parse GameObject : {}", container_path);
                    println!("{:?}", obj.get_string_by_path("/Base/m_Name"));
                    for component in obj
                        .get_array_object_by_path("/Base/m_Component/Array")
                        .unwrap()
                    {
                        let pptr = component.get_object_by_path("/Base/component").unwrap();
                        let pptr = PPtr::new(pptr);
                        let component_obj = pptr
                            .get_type_tree_object_in_view(&unity_asset_viewer)
                            .unwrap()
                            .unwrap();
                        if component_obj.class_id == ClassIDType::Transform as i32 {
                            let transform = Transform::new(component_obj);
                            path_hash_map.extend(
                                get_bone_path_hash_map(&unity_asset_viewer, &transform).unwrap(),
                            );
                            // println!("{:?}", &path_hash_map);
                        } else {
                            if component_obj.class_id == ClassIDType::MonoBehaviour as i32 {
                                if let Some(pptr_o) =
                                    component_obj.get_object_by_path("/Base/m_Script")
                                {
                                    let script_pptr = PPtr::new(pptr_o);
                                    let script = script_pptr
                                        .get_type_tree_object_in_view(&unity_asset_viewer)?
                                        .unwrap();
                                    if let Some(class_name) =
                                        script.get_string_by_path("/Base/m_ClassName")
                                    {
                                        parse_cubism_class(
                                            class_name,
                                            &component_obj,
                                            &mut cubism_physics3_json,
                                            &mut cubism_model3_json,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            for (container_path, obj) in container_paths {
                println!("Parse : {}", container_path);
                if obj.class_id == ClassIDType::MonoBehaviour as i32 {
                    if let Some(pptr_o) = obj.get_object_by_path("/Base/m_Script") {
                        let script_pptr = PPtr::new(pptr_o);
                        let script = script_pptr
                            .get_type_tree_object_in_view(&unity_asset_viewer)?
                            .unwrap();
                        if let Some(class_name) = script.get_string_by_path("/Base/m_ClassName") {
                            if class_name == "CubismMoc" {
                                let container_path = PathBuf::from(container_path);
                                let moc3_name = container_path
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string();

                                // moc3
                                let moc3_path = container_path
                                    .parent()
                                    .unwrap()
                                    .join(moc3_name.clone() + ".moc3");
                                println!("writing {:?}", &moc3_path);
                                create_dir_all(container_path.parent().unwrap());
                                let mut moc3_file = File::create(&moc3_path)?;

                                moc3_file.write_all(
                                    &obj.get_byte_array_by_path("/Base/_bytes/Array").unwrap(),
                                );

                                cubism_model3_json.FileReferences.Moc = moc3_name.clone() + ".moc3";
                                cubism_model3_dir_path_and_name =
                                    Some((container_path.parent().unwrap().to_owned(), moc3_name));
                            } else {
                                parse_cubism_class(
                                    class_name,
                                    &obj,
                                    &mut cubism_physics3_json,
                                    &mut cubism_model3_json,
                                );
                            }
                        }
                    }
                } else if obj.class_id == ClassIDType::Texture2D as i32
                    || obj.class_id == ClassIDType::Sprite as i32
                {
                    // texture
                    println!("Parse texture : {}", container_path);

                    let container_path = PathBuf::from(container_path);
                    let texture_name = container_path
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    let texture2d = if obj.class_id == ClassIDType::Sprite as i32 {
                        let tex_pptr = obj.get_object_by_path("/Base/m_RD/texture").unwrap();
                        let tex_pptr = PPtr::new(tex_pptr);
                        tex_pptr
                            .get_type_tree_object_in_view(&unity_asset_viewer)?
                            .unwrap()
                    } else {
                        obj
                    };

                    let container_path = PathBuf::from(container_path);

                    let tex = Texture2D::new(texture2d);
                    create_dir_all(container_path.parent().unwrap());
                    let tex_path = container_path.parent().unwrap().join(texture_name + ".png");
                    if let Ok(img) = tex.get_image(&unity_asset_viewer) {
                        img.flipv().save(&tex_path);
                    }

                    cubism_model3_json
                        .FileReferences
                        .Textures
                        .push(tex_path.to_string_lossy().to_string())
                } else if obj.class_id == ClassIDType::AnimationClip as i32 {
                    let name = obj.get_string_by_path("/Base/m_Name").unwrap();
                    let name = PathBuf::from(name);
                    let name = name.file_stem().unwrap().to_string_lossy().to_string();
                    println!("AnimationClip : {}", &name);

                    let m_Clip = obj.get_object_by_path("/Base/m_MuscleClip/m_Clip").unwrap();
                    let m_ClipBindingConstant = obj
                        .get_object_by_path("/Base/m_ClipBindingConstant")
                        .unwrap();
                    let stream_count = m_Clip
                        .get_uint_by_path("/Base/data/m_StreamedClip/curveCount")
                        .unwrap();
                    let m_DenseClip = m_Clip.get_object_by_path("/Base/data/m_DenseClip").unwrap();
                    let m_DenseClip_m_CurveCount =
                        m_DenseClip.get_uint_by_path("/Base/m_CurveCount").unwrap();

                    if stream_count == 0 && m_DenseClip_m_CurveCount == 0 {
                        let mut cubism_exp3_json = CubismExp3Json::CubismExp3Json::new();
                        let m_ConstantClip = m_Clip
                            .get_object_by_path("/Base/data/m_ConstantClip")
                            .unwrap();
                        let m_ConstantClip_data = m_ConstantClip
                            .get_array_float_by_path("/Base/data/Array")
                            .unwrap_or(Vec::new());
                        // println!("{:?}", &m_ConstantClip_data);
                        for curve_index in 0..m_ConstantClip_data.len() {
                            let index = curve_index;
                            let binding = animation_clip_binding_constant_find_binding(
                                &m_ClipBindingConstant,
                                index,
                            )
                            .unwrap();
                            let (_live2d_target, live2d_id) =
                                get_live2d_path(&unity_asset_viewer, &path_hash_map, &binding);
                            let mut serializable_expression_parameter =
                                CubismExp3Json::SerializableExpressionParameter::default();
                            serializable_expression_parameter.Id = live2d_id;
                            serializable_expression_parameter.Value =
                                m_ConstantClip_data[curve_index];
                            cubism_exp3_json
                                .Parameters
                                .push(serializable_expression_parameter);
                        }
                        cubism_exp3_json_map.insert(name, cubism_exp3_json);
                        continue;
                    }

                    let mut cubism_motion3_json = CubismMotion3Json::CubismMotion3Json::new();
                    let mut keyframe_curves = Vec::new();
                    cubism_motion3_json.Meta.Duration = obj
                        .get_float_by_path("/Base/m_MuscleClip/m_StopTime")
                        .unwrap();
                    cubism_motion3_json.Meta.Fps =
                        obj.get_float_by_path("/Base/m_SampleRate").unwrap();
                    cubism_motion3_json.Meta.Loop = true;
                    cubism_motion3_json.Meta.AreBeziersRestricted = true;
                    cubism_motion3_json.Meta.CurveCount = 0;
                    cubism_motion3_json.Meta.FadeInTime = 1.0;
                    cubism_motion3_json.Meta.FadeOutTime = 1.0;

                    let streamed_frames = m_Clip
                        .get_object_by_path("/Base/data/m_StreamedClip")
                        .unwrap();
                    let streamed_clip_buff = streamed_frames
                        .get_array_uint32_by_path("/Base/data/Array")
                        .unwrap_or(Vec::new());
                    let streamed_frames = streamed_clip_read_u32_buff(&streamed_clip_buff).unwrap();

                    for curve_index in 0..stream_count {
                        let binding = animation_clip_binding_constant_find_binding(
                            &m_ClipBindingConstant,
                            curve_index as usize,
                        )
                        .unwrap();
                        let (live2d_target, live2d_id) =
                            get_live2d_path(&unity_asset_viewer, &path_hash_map, &binding);
                        let mut keyframe_curve = KeyframeCurve::default();
                        keyframe_curve.target = live2d_target;
                        keyframe_curve.id = live2d_id;
                        keyframe_curves.push(keyframe_curve);
                    }

                    for (frame_index, streamed_frame) in streamed_frames.iter().enumerate() {
                        if frame_index == 0 || frame_index == streamed_frames.len() - 1 {
                            continue;
                        }
                        for key in &streamed_frame.key_list {
                            let mut keyframe = Keyframe::default();
                            keyframe.time = streamed_frame.time;
                            keyframe.value = *key.get_value();
                            keyframe.in_slope = key.in_slope;
                            keyframe.out_slope = *key.get_out_slope();
                            keyframe.coeff = key.coeff.clone();
                            keyframe_curves[key.index as usize].keyframes.push(keyframe);
                        }
                    }

                    let m_DenseClip_m_BeginTime =
                        m_DenseClip.get_float_by_path("/Base/m_BeginTime").unwrap();
                    let m_DenseClip_m_SampleRate =
                        m_DenseClip.get_float_by_path("/Base/m_SampleRate").unwrap();
                    let m_DenseClip_m_FrameCount =
                        m_DenseClip.get_int_by_path("/Base/m_FrameCount").unwrap();
                    let m_DenseClip_m_SampleArray = m_DenseClip
                        .get_array_float_by_path("/Base/m_SampleArray/Array")
                        .unwrap_or(Vec::new());
                    for curve_index in 0..m_DenseClip_m_CurveCount {
                        let index = stream_count + curve_index;

                        let binding = animation_clip_binding_constant_find_binding(
                            &m_ClipBindingConstant,
                            index as usize,
                        )
                        .unwrap();
                        let (live2d_target, live2d_id) =
                            get_live2d_path(&unity_asset_viewer, &path_hash_map, &binding);
                        let mut keyframe_curve = KeyframeCurve::default();
                        keyframe_curve.target = live2d_target;
                        keyframe_curve.id = live2d_id;

                        for frame_index in 0..m_DenseClip_m_FrameCount {
                            let time = m_DenseClip_m_BeginTime
                                + frame_index as f32 / m_DenseClip_m_SampleRate;
                            // let _frameOffset = frame_index * m_DenseClip_m_CurveCount as i64;

                            let mut keyframe = Keyframe::default();
                            keyframe.time = time;
                            keyframe.value = m_DenseClip_m_SampleArray[curve_index as usize];
                            keyframe_curve.keyframes.push(keyframe);
                        }
                        keyframe_curves.push(keyframe_curve);
                    }

                    let m_ConstantClip = m_Clip
                        .get_object_by_path("/Base/data/m_ConstantClip")
                        .unwrap();
                    let denseCount = m_DenseClip_m_CurveCount;
                    let time_end = obj
                        .get_float_by_path("/Base/m_MuscleClip/m_StopTime")
                        .unwrap();
                    let m_ConstantClip_data = m_ConstantClip
                        .get_array_float_by_path("/Base/data/Array")
                        .unwrap_or(Vec::new());
                    // println!("{:?}", &m_ConstantClip_data);
                    for curve_index in 0..m_ConstantClip_data.len() {
                        let index = stream_count + denseCount + curve_index as u64;
                        let binding = animation_clip_binding_constant_find_binding(
                            &m_ClipBindingConstant,
                            index as usize,
                        )
                        .unwrap();
                        let (live2d_target, live2d_id) =
                            get_live2d_path(&unity_asset_viewer, &path_hash_map, &binding);

                        let mut keyframe_curve = KeyframeCurve::default();
                        keyframe_curve.target = live2d_target;
                        keyframe_curve.id = live2d_id;
                        let mut keyframe = Keyframe::default();
                        keyframe.time = 0.0;
                        keyframe.value = m_ConstantClip_data[curve_index];
                        keyframe_curve.keyframes.push(keyframe);
                        let mut keyframe = Keyframe::default();
                        keyframe.time = time_end;
                        keyframe.value = m_ConstantClip_data[curve_index];
                        keyframe_curve.keyframes.push(keyframe);
                        keyframe_curves.push(keyframe_curve);
                    }

                    cubism_motion3_json.Meta.CurveCount = keyframe_curves.len() as i32;

                    let mut total_segment_count = 1;
                    let mut total_point_count = 1;
                    for keyframe_curve in keyframe_curves {
                        let mut curve = CubismMotion3Json::SerializableCurve::default();
                        curve.Id = keyframe_curve.id;
                        curve.Target = keyframe_curve.target;
                        curve.Segments = vec![0.0, keyframe_curve.keyframes[0].value];

                        let mut j = 1;
                        while j < keyframe_curve.keyframes.len() {
                            let keyframe = &keyframe_curve.keyframes[j];
                            let pre_keyframe = &keyframe_curve.keyframes[j - 1];
                            if (keyframe.time - pre_keyframe.time - 0.01).abs() < 0.0001
                            //InverseSteppedSegment
                            {
                                let next_keyframe = &keyframe_curve.keyframes[j + 1];
                                if next_keyframe.value == keyframe.value {
                                    curve.Segments.push(3.0);
                                    curve.Segments.push(next_keyframe.time);
                                    curve.Segments.push(next_keyframe.value);
                                    j += 2;
                                    total_point_count += 1;
                                    total_segment_count += 1;
                                    continue;
                                }
                            }
                            if keyframe.in_slope.is_infinite()
                                && keyframe.in_slope.is_sign_positive()
                            //SteppedSegment
                            {
                                curve.Segments.push(2.0);
                                curve.Segments.push(keyframe.time);
                                curve.Segments.push(keyframe.value);
                                total_point_count += 1;
                            } else if pre_keyframe.out_slope == 0.0
                                && keyframe.in_slope.abs() < 0.0001
                            //LinearSegment
                            {
                                curve.Segments.push(0.0);
                                curve.Segments.push(keyframe.time);
                                curve.Segments.push(keyframe.value);
                                total_point_count += 1;
                            } else
                            //BezierSegment
                            {
                                let tangent_length = (keyframe.time - pre_keyframe.time) / 3.0;
                                curve.Segments.push(1.0);
                                curve.Segments.push(pre_keyframe.time + tangent_length);
                                curve.Segments.push(
                                    pre_keyframe.out_slope * tangent_length + pre_keyframe.value,
                                );
                                curve.Segments.push(keyframe.time - tangent_length);
                                curve
                                    .Segments
                                    .push(keyframe.value - keyframe.in_slope * tangent_length);
                                curve.Segments.push(keyframe.time);
                                curve.Segments.push(keyframe.value);
                                total_point_count += 3;
                            }
                            j += 1;
                            total_segment_count += 1;
                        }

                        cubism_motion3_json.Curves.push(curve);
                    }
                    cubism_motion3_json.Meta.TotalSegmentCount = total_segment_count;
                    cubism_motion3_json.Meta.TotalPointCount = total_point_count;

                    let mut total_user_data_size = 0;
                    for m_Event in obj
                        .get_array_object_by_path("/Base/m_Events/Array")
                        .unwrap()
                    {
                        let mut event = CubismMotion3Json::SerializableUserData::default();
                        event.Time = m_Event.get_float_by_path("/Base/time").unwrap();
                        event.Value = m_Event.get_string_by_path("/Base/data").unwrap();
                        total_user_data_size += event.Value.len();
                        cubism_motion3_json.UserData.push(event);
                    }
                    cubism_motion3_json.Meta.UserDataCount =
                        cubism_motion3_json.UserData.len() as i32;
                    cubism_motion3_json.Meta.TotalUserDataSize = total_user_data_size as i32;

                    // let serialized = serde_json::to_string(&cubism_motion3_json).unwrap();
                    // println!("serialized = {}", serialized);
                    cubism_motion3_json_map.insert(name, cubism_motion3_json);
                }
            }

            if let Some((cubism_model3_dir_path, name)) = cubism_model3_dir_path_and_name {
                if let Some(cubism_physics3_json) = cubism_physics3_json {
                    let cubism_physics3_json_path =
                        cubism_model3_dir_path.join(name.clone() + ".physics3.json");
                    println!("writing {:?}", &cubism_physics3_json_path);
                    let physics_json_file = File::create(cubism_physics3_json_path)?;
                    serde_json::to_writer(physics_json_file, &cubism_physics3_json)?;
                    cubism_model3_json.FileReferences.Physics = name.clone() + ".physics3.json";
                }

                let mut relative_textures = Vec::new();
                for texture in &cubism_model3_json.FileReferences.Textures {
                    let abs_path = PathBuf::from(texture.clone());
                    if let Ok(relative_path) = abs_path.strip_prefix(&cubism_model3_dir_path) {
                        relative_textures.push(relative_path.to_string_lossy().to_string());
                    }
                }
                cubism_model3_json.FileReferences.Textures = relative_textures;

                create_dir_all(cubism_model3_dir_path.join("motions"));
                for (motion_name, cubism_motion3_json) in cubism_motion3_json_map {
                    let cubism_motion3_json_path = cubism_model3_dir_path
                        .join("motions")
                        .join(motion_name.clone() + ".motion3.json");
                    println!("writing {:?}", &cubism_motion3_json_path);
                    let motion_json_file = File::create(cubism_motion3_json_path)?;
                    serde_json::to_writer(motion_json_file, &cubism_motion3_json)?;
                    let mut serializable_motion = CubismModel3Json::SerializableMotion::default();
                    serializable_motion.File =
                        "motions/".to_string() + &motion_name + ".motion3.json";
                    cubism_model3_json
                        .FileReferences
                        .Motions
                        .insert(motion_name, vec![serializable_motion]);
                }

                create_dir_all(cubism_model3_dir_path.join("expressions"));
                for (exp_name, cubism_exp3_json) in cubism_exp3_json_map {
                    let cubism_exp3_json_path = cubism_model3_dir_path
                        .join("expressions")
                        .join(exp_name.clone() + ".exp3.json");
                    println!("writing {:?}", &cubism_exp3_json_path);
                    let exp_json_file = File::create(cubism_exp3_json_path)?;
                    serde_json::to_writer(exp_json_file, &cubism_exp3_json)?;
                    let mut serializable_exp = CubismModel3Json::SerializableExpression::default();
                    serializable_exp.File = "expressions/".to_string() + &exp_name + ".exp3.json";

                    cubism_model3_json
                        .FileReferences
                        .Expressions
                        .push(serializable_exp);
                }

                let cubism_model3_json_path = cubism_model3_dir_path.join(name + ".model3.json");
                println!("writing {:?}", &cubism_model3_json_path);
                let model_json_file = File::create(cubism_model3_json_path)?;
                serde_json::to_writer(model_json_file, &cubism_model3_json)?;
            }
        }
    }

    Ok(())
}

fn parse_cubism_class(
    class_name: String,
    obj: &TypeTreeObject,
    cubism_physics3_json: &mut Option<CubismPhysics3Json::CubismPhysics3Json>,
    cubism_model3_json: &mut CubismModel3Json::CubismModel3Json,
) {
    // parse_cubism_class
    if class_name == "CubismPhysicsController" {
        let mut cubism_physics3_json_m = CubismPhysics3Json::CubismPhysics3Json::new();
        let cubism_physics_rig = obj.get_object_by_path("/Base/_rig").unwrap();
        for (i, sub_rig) in cubism_physics_rig
            .get_array_object_by_path("/Base/SubRigs/Array")
            .unwrap()
            .iter()
            .enumerate()
        {
            let mut physics_setting = CubismPhysics3Json::SerializablePhysicsSettings::default();
            physics_setting.Id = format!("PhysicsSetting{}", i + 1);
            physics_setting.Normalization.Position.Default = sub_rig
                .get_float_by_path("/Base/Normalization/Position/Default")
                .unwrap();
            physics_setting.Normalization.Position.Minimum = sub_rig
                .get_float_by_path("/Base/Normalization/Position/Minimum")
                .unwrap();
            physics_setting.Normalization.Position.Maximum = sub_rig
                .get_float_by_path("/Base/Normalization/Position/Maximum")
                .unwrap();
            physics_setting.Normalization.Angle.Default = sub_rig
                .get_float_by_path("/Base/Normalization/Angle/Default")
                .unwrap();
            physics_setting.Normalization.Angle.Minimum = sub_rig
                .get_float_by_path("/Base/Normalization/Angle/Minimum")
                .unwrap();
            physics_setting.Normalization.Angle.Maximum = sub_rig
                .get_float_by_path("/Base/Normalization/Angle/Maximum")
                .unwrap();

            for input in sub_rig
                .get_array_object_by_path("/Base/Input/Array")
                .unwrap()
            {
                let mut serializable_input = CubismPhysics3Json::SerializableInput::default();
                serializable_input.Source.Target = "Parameter".to_owned();
                serializable_input.Source.Id = input.get_string_by_path("/Base/SourceId").unwrap();
                serializable_input.Weight = input.get_float_by_path("/Base/Weight").unwrap();
                serializable_input.Type =
                    match input.get_int_by_path("/Base/SourceComponent").unwrap() {
                        0 => "X".to_owned(),
                        1 => "Y".to_owned(),
                        2 => "Angle".to_owned(),
                        _ => "".to_owned(),
                    };
                serializable_input.Reflect =
                    input.get_uint_by_path("/Base/IsInverted").unwrap() == 1;
                physics_setting.Input.push(serializable_input);
            }
            for output in sub_rig
                .get_array_object_by_path("/Base/Output/Array")
                .unwrap()
            {
                let mut serializable_output = CubismPhysics3Json::SerializableOutput::default();
                serializable_output.Destination.Target = "Parameter".to_owned();
                serializable_output.Destination.Id =
                    output.get_string_by_path("/Base/DestinationId").unwrap();
                serializable_output.Scale = output.get_float_by_path("/Base/AngleScale").unwrap();
                serializable_output.VertexIndex =
                    output.get_int_by_path("/Base/ParticleIndex").unwrap() as i32;
                serializable_output.Weight = output.get_float_by_path("/Base/Weight").unwrap();
                serializable_output.Type =
                    match output.get_int_by_path("/Base/SourceComponent").unwrap() {
                        0 => "X".to_owned(),
                        1 => "Y".to_owned(),
                        2 => "Angle".to_owned(),
                        _ => "".to_owned(),
                    };
                serializable_output.Reflect =
                    output.get_uint_by_path("/Base/IsInverted").unwrap() == 1;
                physics_setting.Output.push(serializable_output);
            }
            for particl in sub_rig
                .get_array_object_by_path("/Base/Particles/Array")
                .unwrap()
            {
                let mut serializable_vertex = CubismPhysics3Json::SerializableVertex::default();
                let initial_position = particl.get_object_by_path("/Base/InitialPosition").unwrap();
                serializable_vertex.Position.X =
                    initial_position.get_float_by_path("/Base/x").unwrap();
                serializable_vertex.Position.Y =
                    initial_position.get_float_by_path("/Base/y").unwrap();
                serializable_vertex.Mobility = particl.get_float_by_path("/Base/Mobility").unwrap();
                serializable_vertex.Delay = particl.get_float_by_path("/Base/Delay").unwrap();
                serializable_vertex.Acceleration =
                    particl.get_float_by_path("/Base/Acceleration").unwrap();
                serializable_vertex.Radius = particl.get_float_by_path("/Base/Radius").unwrap();
                physics_setting.Vertices.push(serializable_vertex);
            }
            cubism_physics3_json_m.PhysicsSettings.push(physics_setting);
        }
        cubism_physics3_json_m.Meta.PhysicsSettingCount =
            cubism_physics3_json_m.PhysicsSettings.len() as i32;
        cubism_physics3_json_m.Meta.TotalInputCount = cubism_physics3_json_m
            .PhysicsSettings
            .iter()
            .map(|o| o.Input.len())
            .sum::<usize>() as i32;
        cubism_physics3_json_m.Meta.TotalOutputCount = cubism_physics3_json_m
            .PhysicsSettings
            .iter()
            .map(|o| o.Output.len())
            .sum::<usize>() as i32;
        cubism_physics3_json_m.Meta.TotalVertexCount = cubism_physics3_json_m
            .PhysicsSettings
            .iter()
            .map(|o| o.Vertices.len())
            .sum::<usize>() as i32;
        cubism_physics3_json_m.Meta.EffectiveForces.Gravity.X = 0.0;
        cubism_physics3_json_m.Meta.EffectiveForces.Gravity.Y = -1.0;
        cubism_physics3_json_m.Meta.EffectiveForces.Wind.X = 0.0;
        cubism_physics3_json_m.Meta.EffectiveForces.Wind.X = 0.0;

        *cubism_physics3_json = Some(cubism_physics3_json_m);
    } else if class_name == "CubismEyeBlinkParameter" {
        let mut eye_blink = CubismModel3Json::SerializableGroup::default();
        eye_blink.Target = "Parameter".to_owned();
        eye_blink.Name = "EyeBlink".to_owned();
        cubism_model3_json.Groups.push(eye_blink);
    } else if class_name == "CubismMouthParameter" {
        let mut lip_sync = CubismModel3Json::SerializableGroup::default();
        lip_sync.Target = "Parameter".to_owned();
        lip_sync.Name = "LipSync".to_owned();
        cubism_model3_json.Groups.push(lip_sync);
    }
}

// (target,id)
fn get_live2d_path(
    viewer: &UnityAssetViewer,
    path_hash_map: &BTreeMap<u32, String>,
    binding: &TypeTreeObject,
) -> (String, String) {
    let path = binding.get_uint_by_path("/Base/path").unwrap() as u32;
    let string_path = path_hash_map.get(&path);
    if path != 0 && string_path.is_some() {
        if let Some((target, id)) = string_path.unwrap().rsplit_once("/") {
            if target == "Parameters" {
                return ("Parameter".to_owned(), id.to_owned());
            } else if target == "Parts" {
                return ("PartOpacity".to_owned(), id.to_owned());
            }
            return (target.to_owned(), id.to_owned());
        }
    } else if let Some(pptr) = binding.get_object_by_path("/Base/script") {
        let pptr = PPtr::new(pptr);
        let script = pptr.get_type_tree_object_in_view(viewer).unwrap().unwrap();
        match script
            .get_string_by_path("/Base/m_ClassName")
            .unwrap()
            .as_str()
        {
            "CubismRenderController" => {
                return ("Model".to_owned(), "Opacity".to_owned());
            }
            "CubismEyeBlinkController" => {
                return ("Model".to_owned(), "EyeBlink".to_owned());
            }
            "CubismMouthController" => {
                return ("Model".to_owned(), "LipSync".to_owned());
            }
            &_ => (),
        }
    }
    return (String::default(), String::default());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let modeljson = CubismModel3Json::CubismModel3Json::new();
        let serialized = serde_json::to_string(&modeljson).unwrap();
        println!("serialized = {}", serialized);
    }
}
