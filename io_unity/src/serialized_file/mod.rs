pub mod version17;
pub mod version22;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::io::{prelude::*, SeekFrom};

use binrw::{binrw, BinResult};
use binrw::{BinRead, ReadOptions};

use num_enum::TryFromPrimitive;

use crate::classes::animation_clip::AnimationClip;
use crate::classes::animator::Animator;
use crate::classes::asset_bundle::AssetBundle;
use crate::classes::audio_clip::AudioClip;
use crate::classes::avatar::Avatar;
use crate::classes::game_object::GameObject;
use crate::classes::material::Material;
use crate::classes::mesh::Mesh;
use crate::classes::mesh_filter::MeshFilter;
use crate::classes::mesh_renderer::MeshRenderer;
use crate::classes::mono_behaviour::MonoBehaviour;
use crate::classes::mono_script::MonoScript;
use crate::classes::skinned_mesh_renderer::SkinnedMeshRenderer;
use crate::classes::texture_2d::Texture2D;
use crate::classes::transform::Transform;
use crate::classes::{Class, ClassIDType};
use crate::type_tree::{TypeTreeObject, TypeTreeObjectBinReadArgs};
use crate::until::{Endian, UnityVersion};
use crate::UnityResource;

#[binrw]
#[brw(repr = u32)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone)]
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum SerializedFileFormatVersion {
    Unsupported = 1,
    Unknown_2 = 2,
    Unknown_3 = 3,
    /// <summary>
    /// 1.2.0 to 2.0.0
    /// </summary>
    Unknown_5 = 5,
    /// <summary>
    /// 2.1.0 to 2.6.1
    /// </summary>
    Unknown_6 = 6,
    /// <summary>
    /// 3.0.0b
    /// </summary>
    Unknown_7 = 7,
    /// <summary>
    /// 3.0.0 to 3.4.2
    /// </summary>
    Unknown_8 = 8,
    /// <summary>
    /// 3.5.0 to 4.7.2
    /// </summary>
    Unknown_9 = 9,
    /// <summary>
    /// 5.0.0aunk1
    /// </summary>
    Unknown_10 = 10,
    /// <summary>
    /// 5.0.0aunk2
    /// </summary>
    HasScriptTypeIndex = 11,
    /// <summary>
    /// 5.0.0aunk3
    /// </summary>
    Unknown_12 = 12,
    /// <summary>
    /// 5.0.0aunk4
    /// </summary>
    HasTypeTreeHashes = 13,
    /// <summary>
    /// 5.0.0unk
    /// </summary>
    Unknown_14 = 14,
    /// <summary>
    /// 5.0.1 to 5.4.0
    /// </summary>
    SupportsStrippedObject = 15,
    /// <summary>
    /// 5.5.0a
    /// </summary>
    RefactoredClassId = 16,
    /// <summary>
    /// 5.5.0unk to 2018.4
    /// </summary>
    RefactorTypeData = 17,
    /// <summary>
    /// 2019.1a
    /// </summary>
    RefactorShareableTypeTreeData = 18,
    /// <summary>
    /// 2019.1unk
    /// </summary>
    TypeTreeNodeWithTypeFlags = 19,
    /// <summary>
    /// 2019.2
    /// </summary>
    SupportsRefObject = 20,
    /// <summary>
    /// 2019.3 to 2019.4
    /// </summary>
    StoresTypeDependencies = 21,
    /// <summary>
    /// 2020.1 to x
    /// </summary>
    LargeFilesSupport = 22,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone)]
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum BuildTarget {
    NoTarget = -2,
    AnyPlayer = -1,
    ValidPlayer = 1,
    StandaloneOSX = 2,
    StandaloneOSXPPC = 3,
    StandaloneOSXIntel = 4,
    StandaloneWindows,
    WebPlayer,
    WebPlayerStreamed,
    Wii = 8,
    iOS = 9,
    PS3,
    XBOX360,
    Broadcom = 12,
    Android = 13,
    StandaloneGLESEmu = 14,
    StandaloneGLES20Emu = 15,
    NaCl = 16,
    StandaloneLinux = 17,
    FlashPlayer = 18,
    StandaloneWindows64 = 19,
    WebGL,
    WSAPlayer,
    StandaloneLinux64 = 24,
    StandaloneLinuxUniversal,
    WP8Player,
    StandaloneOSXIntel64,
    BlackBerry,
    Tizen,
    PSP2,
    PS4,
    PSM,
    XboxOne,
    SamsungTV,
    N3DS,
    WiiU,
    tvOS,
    Switch,
    Lumin,
    Stadia,
    CloudRendering,
    GameCoreXboxSeries,
    GameCoreXboxOne,
    PS5,
    EmbeddedLinux,
    QNX,
    UnknownPlatform = 9999,
}

lazy_static! {
    static ref COMMON_STRING: HashMap<u32, &'static str> = [
        (0, "AABB"),
        (5, "AnimationClip"),
        (19, "AnimationCurve"),
        (34, "AnimationState"),
        (49, "Array"),
        (55, "Base"),
        (60, "BitField"),
        (69, "bitset"),
        (76, "bool"),
        (81, "char"),
        (86, "ColorRGBA"),
        (96, "Component"),
        (106, "data"),
        (111, "deque"),
        (117, "double"),
        (124, "dynamic_array"),
        (138, "FastPropertyName"),
        (155, "first"),
        (161, "float"),
        (167, "Font"),
        (172, "GameObject"),
        (183, "Generic Mono"),
        (196, "GradientNEW"),
        (208, "GUID"),
        (213, "GUIStyle"),
        (222, "int"),
        (226, "list"),
        (231, "long long"),
        (241, "map"),
        (245, "Matrix4x4f"),
        (256, "MdFour"),
        (263, "MonoBehaviour"),
        (277, "MonoScript"),
        (288, "m_ByteSize"),
        (299, "m_Curve"),
        (307, "m_EditorClassIdentifier"),
        (331, "m_EditorHideFlags"),
        (349, "m_Enabled"),
        (359, "m_ExtensionPtr"),
        (374, "m_GameObject"),
        (387, "m_Index"),
        (395, "m_IsArray"),
        (405, "m_IsStatic"),
        (416, "m_MetaFlag"),
        (427, "m_Name"),
        (434, "m_ObjectHideFlags"),
        (452, "m_PrefabInternal"),
        (469, "m_PrefabParentObject"),
        (490, "m_Script"),
        (499, "m_StaticEditorFlags"),
        (519, "m_Type"),
        (526, "m_Version"),
        (536, "Object"),
        (543, "pair"),
        (548, "PPtr<Component>"),
        (564, "PPtr<GameObject>"),
        (581, "PPtr<Material>"),
        (596, "PPtr<MonoBehaviour>"),
        (616, "PPtr<MonoScript>"),
        (633, "PPtr<Object>"),
        (646, "PPtr<Prefab>"),
        (659, "PPtr<Sprite>"),
        (672, "PPtr<TextAsset>"),
        (688, "PPtr<Texture>"),
        (702, "PPtr<Texture2D>"),
        (718, "PPtr<Transform>"),
        (734, "Prefab"),
        (741, "Quaternionf"),
        (753, "Rectf"),
        (759, "RectInt"),
        (767, "RectOffset"),
        (778, "second"),
        (785, "set"),
        (789, "short"),
        (795, "size"),
        (800, "SInt16"),
        (807, "SInt32"),
        (814, "SInt64"),
        (821, "SInt8"),
        (827, "staticvector"),
        (840, "string"),
        (847, "TextAsset"),
        (857, "TextMesh"),
        (866, "Texture"),
        (874, "Texture2D"),
        (884, "Transform"),
        (894, "TypelessData"),
        (907, "UInt16"),
        (914, "UInt32"),
        (921, "UInt64"),
        (928, "UInt8"),
        (934, "unsigned int"),
        (947, "unsigned long long"),
        (966, "unsigned short"),
        (981, "vector"),
        (988, "Vector2f"),
        (997, "Vector3f"),
        (1006, "Vector4f"),
        (1015, "m_ScriptingClassIdentifier"),
        (1042, "Gradient"),
        (1051, "Type*"),
        (1057, "int2_storage"),
        (1070, "int3_storage"),
        (1083, "BoundsInt"),
        (1093, "m_CorrespondingSourceObject"),
        (1121, "m_PrefabInstance"),
        (1138, "m_PrefabAsset"),
        (1152, "FileSize"),
    ]
    .iter()
    .copied()
    .collect();
}

#[binrw]
#[brw(big)]
#[derive(Debug, Eq, PartialEq)]
pub struct SerializedFileCommonHeader {
    metadata_size: u32,
    file_size: u32,
    version: SerializedFileFormatVersion,
    data_offset: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SerializedFileMetadata {
    pub version: SerializedFileFormatVersion,
    pub endianess: Endian,
    pub unity_version: UnityVersion,
    pub target_platform: BuildTarget,
}

#[derive(Debug, PartialEq)]
pub struct Object {
    pub path_id: i64,
    byte_start: u64,
    byte_size: u32,
    pub class: ClassIDType,
    type_id: usize,
}

pub struct SerializedFile {
    content: Box<dyn Serialized + Send>,
    file_reader: RefCell<Box<dyn UnityResource + Send>>,
    object_map: BTreeMap<i64, Object>,
}

impl fmt::Debug for SerializedFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SerializedFile")
            .field("content", &self.content)
            .finish()
    }
}

impl SerializedFile {
    pub fn read(mut reader: Box<dyn UnityResource + Send>) -> BinResult<Self> {
        let head = SerializedFileCommonHeader::read(&mut reader)?;
        reader.seek(SeekFrom::Start(0));
        let file: Box<dyn Serialized + Send> = match head.version {
            SerializedFileFormatVersion::Unsupported => todo!(),
            SerializedFileFormatVersion::Unknown_2 => todo!(),
            SerializedFileFormatVersion::Unknown_3 => todo!(),
            SerializedFileFormatVersion::Unknown_5 => todo!(),
            SerializedFileFormatVersion::Unknown_6 => todo!(),
            SerializedFileFormatVersion::Unknown_7 => todo!(),
            SerializedFileFormatVersion::Unknown_8 => todo!(),
            SerializedFileFormatVersion::Unknown_9 => todo!(),
            SerializedFileFormatVersion::Unknown_10 => todo!(),
            SerializedFileFormatVersion::HasScriptTypeIndex => todo!(),
            SerializedFileFormatVersion::Unknown_12 => todo!(),
            SerializedFileFormatVersion::HasTypeTreeHashes => todo!(),
            SerializedFileFormatVersion::Unknown_14 => todo!(),
            SerializedFileFormatVersion::SupportsStrippedObject => todo!(),
            SerializedFileFormatVersion::RefactoredClassId => todo!(),
            SerializedFileFormatVersion::RefactorTypeData => {
                Box::new(version17::SerializedFile::read(&mut reader)?)
            }
            SerializedFileFormatVersion::RefactorShareableTypeTreeData => todo!(),
            SerializedFileFormatVersion::TypeTreeNodeWithTypeFlags => todo!(),
            SerializedFileFormatVersion::SupportsRefObject => todo!(),
            SerializedFileFormatVersion::StoresTypeDependencies => todo!(),
            SerializedFileFormatVersion::LargeFilesSupport => {
                Box::new(version22::SerializedFile::read(&mut reader)?)
            }
        };
        let mut object_map = BTreeMap::new();
        for i in 0..file.get_object_count() {
            let obj = file.get_raw_object_by_index(i as u32);
            object_map.insert(obj.path_id, obj);
        }
        Ok(SerializedFile {
            content: file,
            file_reader: RefCell::new(reader),
            object_map,
        })
    }

    pub fn get_object_count(&self) -> i32 {
        self.content.get_object_count()
    }

    pub fn get_object_map(&self) -> &BTreeMap<i64, Object> {
        &self.object_map
    }

    pub fn get_object_by_index(&self, index: u32) -> Option<Class> {
        let obj = self.content.get_raw_object_by_index(index);
        self.content
            .get_object(&mut *self.file_reader.borrow_mut(), &obj)
    }

    pub fn get_object_by_path_id(&self, path_id: i64) -> Option<Class> {
        if let Some(obj) = self.object_map.get(&path_id) {
            return self
                .content
                .get_object(&mut *self.file_reader.borrow_mut(), obj);
        }
        None
    }

    pub fn get_tt_object_by_path_id(&self, path_id: i64) -> Option<TypeTreeObject> {
        if let Some(obj) = self.object_map.get(&path_id) {
            return Some(
                self.content
                    .get_type_tree_object(&mut *self.file_reader.borrow_mut(), obj),
            );
        }
        None
    }
}

pub trait Serialized: fmt::Debug {
    fn get_serialized_file_header(&self) -> &SerializedFileCommonHeader;
    fn get_data_offset(&self) -> u64;
    fn get_endianess(&self) -> Endian;
    fn get_raw_object_by_index(&self, index: u32) -> Object;
    fn get_type_object_args_by_type_id(&self, type_id: usize) -> TypeTreeObjectBinReadArgs;
    fn get_object_count(&self) -> i32;
    fn get_version(&self) -> String;
    fn get_target_platform(&self) -> BuildTarget;

    fn get_metadata(&self) -> SerializedFileMetadata {
        SerializedFileMetadata {
            version: self.get_serialized_file_header().version.clone(),
            endianess: self.get_endianess(),
            unity_version: UnityVersion::from_str(&self.get_version()).unwrap(),
            target_platform: self.get_target_platform(),
        }
    }

    fn get_object_by_index(
        &self,
        reader: &mut Box<dyn UnityResource + Send>,
        index: u32,
    ) -> Option<Class> {
        let obj = self.get_raw_object_by_index(index);
        self.get_object(reader, &obj)
    }

    fn get_type_tree_object(
        &self,
        reader: &mut Box<dyn UnityResource + Send>,
        obj: &Object,
    ) -> TypeTreeObject {
        let args = self.get_type_object_args_by_type_id(obj.type_id);
        reader.seek(SeekFrom::Start(self.get_data_offset() + obj.byte_start));

        let options = ReadOptions::new(match self.get_endianess() {
            Endian::Little => binrw::Endian::Little,
            Endian::Big => binrw::Endian::Big,
        });

        let type_tree_object = TypeTreeObject::read_options(reader, &options, args).unwrap();
        let apos = reader.seek(SeekFrom::Current(0)).unwrap();
        assert_eq!(
            apos - (self.get_data_offset() + obj.byte_start),
            obj.byte_size as u64
        );
        type_tree_object
    }

    fn get_object(
        &self,
        reader: &mut Box<dyn UnityResource + Send>,
        obj: &Object,
    ) -> Option<Class> {
        reader.seek(SeekFrom::Start(self.get_data_offset() + obj.byte_start));

        let op = ReadOptions::new(match self.get_endianess() {
            Endian::Little => binrw::Endian::Little,
            Endian::Big => binrw::Endian::Big,
        });

        #[macro_export]
        macro_rules! cov_class {
            ($($x:ident($y:path)),+) => {
                match obj.class {
                    $(ClassIDType::$x => {
                            if let Ok(o) = $x::read_options(reader, &op, self.get_metadata()) {
                                Some(Class::$x(o))
                            } else {
                                None
                            }
                        },)+
                    _ => {
                        println!("{:?}", &obj.class);
                        None
                    }
                }
            };
        }

        cov_class!(
            AssetBundle(asset_bundle::AssetBundle),
            AudioClip(audio_clip::AudioClip),
            Texture2D(texture_2d::Texture2D),
            Mesh(mesh::Mesh),
            Transform(transform::Transform),
            GameObject(game_object::GameObject),
            AnimationClip(animation_clip::AnimationClip),
            SkinnedMeshRenderer(skinned_mesh_renderer::SkinnedMeshRenderer),
            MeshRenderer(mesh_renderer::MeshRenderer),
            Material(material::Material),
            MeshFilter(mesh_filter::MeshFilter),
            MonoBehaviour(mono_behaviour::MonoBehaviour),
            MonoScript(mono_script::MonoScript),
            Animator(animator::Animator),
            Avatar(avatar::Avatar)
        )
    }

    fn get_asset_bundle(&self, reader: &mut Box<dyn UnityResource + Send>) -> Option<Class> {
        for i in 0..self.get_object_count() {
            let obj = self.get_raw_object_by_index(i as u32);
            if obj.class == ClassIDType::AssetBundle {
                return self.get_object(reader, &obj);
            }
        }
        None
    }
}
