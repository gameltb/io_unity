use super::{BoneWeights, MeshObject, StreamBuff};
use crate::classes::named_object::{self, NamedObject, NamedObjectObject};
use crate::until::binrw_parser::AlignedString;
use crate::until::binrw_parser::*;
use crate::SerializedFileMetadata;
use binrw::{binrw, ReadOptions, VecArgs};
use binrw::{io::Cursor, BinRead};
use num_enum::TryFromPrimitive;
use std::io::{prelude::*, SeekFrom};
use supercow::Supercow;

impl named_object::DownCast for Mesh {
    fn downcast<'a>(&'a self) -> Supercow<Box<dyn NamedObjectObject + Send + 'a>> {
        Supercow::borrowed(&*self.name)
    }
}

impl MeshObject for Mesh {
    fn get_index_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<u32>> {
        let sub_mesh = self
            .sub_meshes
            .get(sub_mesh_id)
            .ok_or(anyhow!("sub_mesh"))?;

        let mut reader = Cursor::new(&self.index_buffer);
        reader.seek(SeekFrom::Start(sub_mesh.first_byte as u64))?;
        let op = ReadOptions::new(self.endian);

        if self.index_format == 0 {
            let buff = <Vec<u16>>::read_options(
                &mut reader,
                &op,
                VecArgs {
                    count: sub_mesh.index_count as usize,
                    inner: (),
                },
            )?;

            Ok(buff
                .into_iter()
                .map(|i| i as u32 - sub_mesh.first_vertex)
                .collect())
        } else {
            let buff = <Vec<u32>>::read_options(
                &mut reader,
                &op,
                VecArgs {
                    count: sub_mesh.index_count as usize,
                    inner: (),
                },
            )?;

            Ok(buff
                .into_iter()
                .map(|i| i - sub_mesh.first_vertex)
                .collect())
        }
    }

    fn get_vertex_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>> {
        let sub_mesh = self
            .sub_meshes
            .get(sub_mesh_id)
            .ok_or(anyhow!("sub_mesh"))?;

        Ok(match self.vertex_data.get_channel(
            &ChannelType::kShaderChannelVertex,
            sub_mesh,
            self.endian,
        )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        }
        .concat())
    }

    fn get_normal_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>> {
        let sub_mesh = self
            .sub_meshes
            .get(sub_mesh_id)
            .ok_or(anyhow!("sub_mesh"))?;

        Ok(match self.vertex_data.get_channel(
            &ChannelType::kShaderChannelNormal,
            sub_mesh,
            self.endian,
        )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        }
        .concat())
    }

    fn get_uv0_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>> {
        let sub_mesh = self
            .sub_meshes
            .get(sub_mesh_id)
            .ok_or(anyhow!("sub_mesh"))?;

        Ok(match self.vertex_data.get_channel(
            &ChannelType::kShaderChannelTexCoord0,
            sub_mesh,
            self.endian,
        )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        }
        .concat())
    }

    fn get_bone_weights_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<BoneWeights>> {
        let sub_mesh = self
            .sub_meshes
            .get(sub_mesh_id)
            .ok_or(anyhow!("sub_mesh"))?;

        let weight_buff = match self.vertex_data.get_channel(
            &ChannelType::kShaderChannelBlendWeight,
            sub_mesh,
            self.endian,
        )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        };
        let bone_index_buff = match self.vertex_data.get_channel(
            &ChannelType::kShaderChannelBlendIndices,
            sub_mesh,
            self.endian,
        )? {
            StreamBuff::U32(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::Float(_) => unreachable!(),
        };

        let mut buff = Vec::new();
        for (weight, bone_index) in weight_buff.into_iter().zip(bone_index_buff) {
            buff.push(BoneWeights { weight, bone_index });
        }
        Ok(buff)
    }

    fn get_sub_mesh_count(&self) -> Option<usize> {
        Some(self.sub_meshes.len())
    }

    fn get_bind_pose(&self) -> anyhow::Result<Supercow<Vec<Mat4>>> {
        Ok(Supercow::borrowed(&self.bind_pose))
    }
}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Mesh {
    #[brw(args_raw = args)]
    name: NamedObject,
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
    bones_aabb_size: i32,
    #[br(count(bones_aabb_size))]
    bones_aabb: Vec<MinMaxAABB>,
    num_variable_bone_count_weights: i32,
    #[br(count(num_variable_bone_count_weights))]
    variable_bone_count_weights: Vec<u32>,
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
    #[br(align_before(4))]
    stream_data: StreamingInfo,
    #[br(parse_with = endian_parser)]
    #[bw(ignore)]
    endian: binrw::Endian,
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
pub struct MinMaxAABB {
    min: Vec3,
    max: Vec3,
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
    num_full_weights: i32,
    #[br(count(num_full_weights))]
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

impl VertexData {
    fn get_stream_offset(&self, stream: u8) -> usize {
        let mut offset = 0;
        for s in 0..stream {
            offset += self.get_stream_stride(s) * (self.vertex_count as usize);
            if offset % 16 != 0 {
                offset += 16 - (offset % 16);
            }
        }
        offset
    }

    fn get_stream_stride(&self, stream: u8) -> usize {
        let mut stride = 0;
        for channel in &self.channels {
            if channel.stream == stream {
                stride += get_format_size(channel.format.clone()) * channel.dimension
            }
        }
        stride as usize
    }

    fn get_channel(
        &self,
        channel: &ChannelType,
        sub_mesh: &SubMesh,
        endian: binrw::Endian,
    ) -> anyhow::Result<StreamBuff> {
        let channel = &self.channels[channel.clone() as u8 as usize];
        let offset = self.get_stream_offset(channel.stream);
        let stride = self.get_stream_stride(channel.stream);
        let mut reader = Cursor::new(&self.data);
        let op = ReadOptions::new(endian);
        match &channel.format {
            VertexFormat::Float => {
                let mut buff = vec![];
                for i in sub_mesh.first_vertex..sub_mesh.vertex_count + sub_mesh.first_vertex {
                    reader.seek(SeekFrom::Start(
                        offset as u64 + i as u64 * stride as u64 + channel.offset as u64,
                    ))?;
                    let sbuff = <Vec<f32>>::read_options(
                        &mut reader,
                        &op,
                        VecArgs {
                            count: channel.dimension as usize,
                            inner: (),
                        },
                    )?;

                    buff.push(sbuff);
                }
                Ok(StreamBuff::Float(buff))
            }
            VertexFormat::Float16 => {
                let mut buff = vec![];
                for i in sub_mesh.first_vertex..sub_mesh.vertex_count + sub_mesh.first_vertex {
                    reader.seek(SeekFrom::Start(
                        offset as u64 + i as u64 * stride as u64 + channel.offset as u64,
                    ))?;
                    let sbuff = <Vec<u16>>::read_options(
                        &mut reader,
                        &op,
                        VecArgs {
                            count: channel.dimension as usize,
                            inner: (),
                        },
                    )?;

                    buff.push(
                        sbuff
                            .into_iter()
                            .map(|f| half::f16::from_bits(f).to_f32())
                            .collect(),
                    );
                }
                Ok(StreamBuff::Float(buff))
            }
            VertexFormat::UNorm8 => todo!(),
            VertexFormat::SNorm8 => todo!(),
            VertexFormat::UNorm16 => todo!(),
            VertexFormat::SNorm16 => todo!(),
            VertexFormat::UInt8 => todo!(),
            VertexFormat::SInt8 => todo!(),
            VertexFormat::UInt16 => todo!(),
            VertexFormat::SInt16 => todo!(),
            VertexFormat::UInt32 => {
                let mut buff = vec![];
                for i in sub_mesh.first_vertex..sub_mesh.vertex_count + sub_mesh.first_vertex {
                    reader.seek(SeekFrom::Start(
                        offset as u64 + i as u64 * stride as u64 + channel.offset as u64,
                    ))?;
                    let sbuff = <Vec<u32>>::read_options(
                        &mut reader,
                        &op,
                        VecArgs {
                            count: channel.dimension as usize,
                            inner: (),
                        },
                    )?;

                    buff.push(sbuff);
                }
                Ok(StreamBuff::U32(buff))
            }
            VertexFormat::SInt32 => todo!(),
        }
    }
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

#[binrw]
#[derive(Debug)]
pub struct ChannelInfo {
    stream: u8,
    offset: u8,
    format: VertexFormat,
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
pub struct StreamingInfo {
    offset: u64,
    size: u32,
    path: AlignedString,
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
