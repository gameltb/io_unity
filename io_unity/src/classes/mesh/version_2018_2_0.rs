use super::MeshObject;
use crate::until::binrw_parser::AlignedString;
use crate::until::binrw_parser::*;
use crate::SerializedFileMetadata;
use binrw::binrw;
use num_enum::TryFromPrimitive;

impl Mesh {
    fn get_name(&self) -> &AlignedString {
        &self.name
    }
}

impl MeshObject for Mesh {
    fn get_index_buff(&self, _sub_mesh_id: usize) -> Vec<u32> {
        todo!()
    }

    fn get_vertex_buff(&self, _sub_mesh_id: usize) -> Vec<f32> {
        todo!()
    }

    fn get_normal_buff(&self, _sub_mesh_id: usize) -> Vec<f32> {
        todo!()
    }

    fn get_uv0_buff(&self, _sub_mesh_id: usize) -> Vec<f32> {
        todo!()
    }

    fn get_sub_mesh_count(&self) -> usize {
        todo!()
    }

    fn get_bone_weights_buff(&self, _sub_mesh_id: usize) -> Vec<super::BoneWeights> {
        todo!()
    }

    fn get_bind_pose(&self) -> &Vec<Mat4> {
        todo!()
    }
}

#[binrw]
#[brw(import_raw(_args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Mesh {
    name: AlignedString,
    sub_meshes_size: i32,
    #[br(count(sub_meshes_size))]
    sub_meshes: Vec<SubMesh>,
    shapes: BlendShapeData,
    num_bind_pose: i32,
    #[br(count(num_bind_pose))]
    bind_pose: Vec<Mat4>,
    num_bone_name_hashes: i32,
    #[br(count(num_bone_name_hashes))]
    bone_name_hashes: Vec<u32>,
    root_bone_name_hash: u32,
    mesh_compression: u8,
    is_readable: U8Bool,
    keep_vertices: U8Bool,
    #[br(align_after(4))]
    keep_indices: U8Bool,
    index_format: u32,
    index_buffer_size: u32,
    #[br(count(index_buffer_size), align_after(4))]
    index_buffer: Vec<u8>,
    vertex_data: VertexData,
    compressed_mesh: CompressedMesh,
    local_aabb: AABB,
    mesh_usage_flags: i32,
    baked_convex_collision_mesh_size: i32,
    #[br(count(baked_convex_collision_mesh_size), align_after(4))]
    baked_convex_collision_mesh: Vec<u8>,
    baked_triangle_collision_mesh_size: i32,
    #[br(count(baked_triangle_collision_mesh_size), align_after(4))]
    baked_triangle_collision_mesh: Vec<u8>,
    mesh_metrics: [f32; 2],
}

#[binrw]
#[derive(Debug)]
pub struct SubMesh {
    first_byte: u32,
    index_count: u32,
    topology: GfxPrimitiveType,
    base_vertex: u32,
    first_vertex: u32,
    vertex_count: u32,
    local_aabb: AABB,
}

#[binrw]
#[derive(Debug)]
pub struct AABB {
    center: Vec3,
    extent: Vec3,
}

#[binrw]
#[derive(Debug)]
pub struct BlendShapeData {
    num_verts: i32,
    #[br(count(num_verts))]
    vertices: Vec<BlendShapeVertex>,
    num_shapes: i32,
    #[br(count(num_shapes))]
    shapes: Vec<MeshBlendShape>,
    num_channels: i32,
    #[br(count(num_channels))]
    channels: Vec<MeshBlendShapeChannel>,
    numfull_weights: i32,
    #[br(count(numfull_weights))]
    full_weights: Vec<f32>,
}

#[binrw]
#[derive(Debug)]
pub struct BlendShapeVertex {
    vertex: Vec3,
    normal: Vec3,
    tangent: Vec3,
    index: u32,
}

#[binrw]
#[derive(Debug)]
pub struct MeshBlendShape {
    first_vertex: u32,
    vertex_count: u32,
    has_normals: U8Bool,
    #[br(align_after(4))]
    has_tangents: U8Bool,
}

#[binrw]
#[derive(Debug)]
pub struct MeshBlendShapeChannel {
    name: AlignedString,
    name_hash: u32,
    frame_index: i32,
    frame_count: i32,
}

#[binrw]
#[derive(Debug)]
pub struct VertexData {
    vertex_count: u32,
    channels_size: i32,
    #[br(count(channels_size))]
    channels: Vec<ChannelInfo>,
    data_size: i32,
    #[br(count(data_size))]
    data: Vec<u8>,
}

#[binrw]
#[derive(Debug)]
pub struct ChannelInfo {
    stream: u8,
    offset: u8,
    format: u8,
    dimension: u8,
}

#[binrw]
#[derive(Debug)]
pub struct CompressedMesh {
    vertices: PackedFloatVector,
    uv: PackedFloatVector,
    normals: PackedFloatVector,
    tangents: PackedFloatVector,
    weights: PackedIntVector,
    normal_signs: PackedIntVector,
    tangent_signs: PackedIntVector,
    float_colors: PackedFloatVector,
    bone_indices: PackedIntVector,
    triangles: PackedIntVector,
    uvinfo: u32,
}

#[binrw]
#[derive(Debug)]
pub struct PackedFloatVector {
    num_items: u32,
    range: f32,
    start: f32,
    num_data: i32,
    #[br(count(num_data), align_after(4))]
    data: Vec<u8>,
    #[br(align_after(4))]
    bit_size: u8,
}

#[binrw]
#[derive(Debug)]
pub struct PackedIntVector {
    num_items: u32,
    num_data: i32,
    #[br(count(num_data), align_after(4))]
    data: Vec<u8>,
    #[br(align_after(4))]
    bit_size: u8,
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
