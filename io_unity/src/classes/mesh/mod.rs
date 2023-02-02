pub mod type_tree;

use crate::{def_unity_class, until::binrw_parser::Mat4};
use binrw::binrw;
use num_enum::TryFromPrimitive;

def_unity_class!(Mesh);

pub trait MeshObject {
    fn get_index_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<u32>>;
    fn get_vertex_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>>;
    fn get_normal_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>>;
    fn get_uv0_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>>;
    fn get_sub_mesh_count(&self) -> Option<usize>;
    fn get_bone_weights_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<BoneWeights>>;
    fn get_bind_pose(&self) -> anyhow::Result<Vec<Mat4>>;
}

pub fn get_format_size(format: VertexFormat) -> u8 {
    match format {
        VertexFormat::UNorm8 | VertexFormat::SNorm8 | VertexFormat::UInt8 | VertexFormat::SInt8 => {
            1
        }
        VertexFormat::UInt16
        | VertexFormat::SInt16
        | VertexFormat::UNorm16
        | VertexFormat::SNorm16
        | VertexFormat::Float16 => 2,
        VertexFormat::Float | VertexFormat::UInt32 | VertexFormat::SInt32 => 4,
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StreamBuff {
    Float(Vec<Vec<f32>>),
    I32(Vec<Vec<i32>>),
    U32(Vec<Vec<u32>>),
}

#[derive(Debug)]
pub struct BoneWeights {
    pub weight: Vec<f32>,
    pub bone_index: Vec<u32>,
}

#[binrw]
#[derive(Debug)]
struct BoneWeights4 {
    weight: [f32; 4],
    bone_index: [u32; 4],
}

#[binrw]
#[brw(repr = u32)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone)]
#[repr(u32)]
pub enum GfxPrimitiveType {
    Triangles = 0,
    TriangleStrip = 1,
    Quads = 2,
    Lines = 3,
    LineStrip = 4,
    Points = 5,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone)]
#[repr(u8)]
pub enum VertexChannelFormat {
    Float,
    Float16,
    Color,
    Byte,
    UInt32,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone)]
#[repr(u8)]
pub enum VertexFormat2017 {
    Float,
    Float16,
    Color,
    UNorm8,
    SNorm8,
    UNorm16,
    SNorm16,
    UInt8,
    SInt8,
    UInt16,
    SInt16,
    UInt32,
    SInt32,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone)]
#[repr(u8)]
pub enum VertexFormat {
    Float,
    Float16,
    UNorm8,
    SNorm8,
    UNorm16,
    SNorm16,
    UInt8,
    SInt8,
    UInt16,
    SInt16,
    UInt32,
    SInt32,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum ChannelType {
    kShaderChannelVertex,
    kShaderChannelNormal,
    kShaderChannelTangent,
    kShaderChannelColor,
    kShaderChannelTexCoord0,
    kShaderChannelTexCoord1,
    kShaderChannelTexCoord2,
    kShaderChannelTexCoord3,
    kShaderChannelTexCoord4,
    kShaderChannelTexCoord5,
    kShaderChannelTexCoord6,
    kShaderChannelTexCoord7,
    kShaderChannelBlendWeight,
    kShaderChannelBlendIndices,
}
