use std::borrow::Cow;
use std::io::{Cursor, Seek, SeekFrom};

use super::version_2020_0_0::{get_format_size, ChannelType, VertexFormat};
use super::{BoneWeights, MeshObject, StreamBuff};
use crate::classes::named_object::{self, NamedObjectObject};
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;
use binrw::{BinRead, ReadOptions, VecArgs};
use supercow::Supercow;

def_type_tree_class!(Mesh);

impl named_object::DownCast for Mesh<'_> {
    fn downcast<'a>(&'a self) -> Supercow<Box<dyn NamedObjectObject + Send + 'a>> {
        Supercow::owned(Box::new(named_object::type_tree::NamedObject::new(
            &*self.inner,
        )))
    }
}

impl MeshObject for Mesh<'_> {
    fn get_index_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<u32>> {
        let binding = self.get_sub_meshes().ok_or(anyhow!("sub_meshs"))?;
        let sub_mesh = binding.get(sub_mesh_id).ok_or(anyhow!("sub_mesh"))?;

        let buff = self.get_index_buffer().ok_or(anyhow!("get_index_buffer"))?;
        let mut reader = Cursor::new(buff.as_ref());
        reader.seek(SeekFrom::Start(
            sub_mesh.get_first_byte().ok_or(anyhow!("get_first_byte"))?,
        ))?;

        let op = ReadOptions::new(self.inner.get_endian());
        let first_vertex = sub_mesh
            .get_first_vertex()
            .ok_or(anyhow!("get_first_vertex"))?;

        if self.get_index_format().ok_or(anyhow!("get_index_format"))? == 0 {
            let buff = <Vec<u16>>::read_options(
                &mut reader,
                &op,
                VecArgs {
                    count: sub_mesh
                        .get_index_count()
                        .ok_or(anyhow!("get_index_count"))? as usize,
                    inner: (),
                },
            )?;
            Ok(buff
                .into_iter()
                .map(|i| i as u32 - first_vertex as u32)
                .collect())
        } else {
            let buff = <Vec<u32>>::read_options(
                &mut reader,
                &op,
                VecArgs {
                    count: sub_mesh
                        .get_index_count()
                        .ok_or(anyhow!("get_index_count"))? as usize,
                    inner: (),
                },
            )?;
            Ok(buff.into_iter().map(|i| i - first_vertex as u32).collect())
        }
    }

    fn get_vertex_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>> {
        let binding = self.get_sub_meshes().ok_or(anyhow!("sub_meshs"))?;
        let sub_mesh = binding.get(sub_mesh_id).ok_or(anyhow!("sub_mesh"))?;

        Ok(match self
            .get_vertex_data()
            .ok_or(anyhow!("get_vertex_data"))?
            .get_channel(
                &ChannelType::kShaderChannelVertex,
                sub_mesh,
                self.inner.get_endian(),
            )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        }
        .concat())
    }

    fn get_normal_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>> {
        let binding = self.get_sub_meshes().ok_or(anyhow!("sub_meshs"))?;
        let sub_mesh = binding.get(sub_mesh_id).ok_or(anyhow!("sub_mesh"))?;

        Ok(match self
            .get_vertex_data()
            .ok_or(anyhow!("get_vertex_data"))?
            .get_channel(
                &ChannelType::kShaderChannelNormal,
                sub_mesh,
                self.inner.get_endian(),
            )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        }
        .concat())
    }

    fn get_uv0_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>> {
        let binding = self.get_sub_meshes().ok_or(anyhow!("sub_meshs"))?;
        let sub_mesh = binding.get(sub_mesh_id).ok_or(anyhow!("sub_mesh"))?;

        Ok(match self
            .get_vertex_data()
            .ok_or(anyhow!("get_vertex_data"))?
            .get_channel(
                &ChannelType::kShaderChannelTexCoord0,
                sub_mesh,
                self.inner.get_endian(),
            )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        }
        .concat())
    }

    fn get_bone_weights_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<BoneWeights>> {
        let binding = self.get_sub_meshes().ok_or(anyhow!("get_sub_meshes"))?;
        let sub_mesh = binding.get(sub_mesh_id).ok_or(anyhow!("sub_mesh"))?;

        let weight_buff = match self
            .get_vertex_data()
            .ok_or(anyhow!("get_vertex_data"))?
            .get_channel(
                &ChannelType::kShaderChannelBlendWeight,
                sub_mesh,
                self.inner.get_endian(),
            )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        };
        let bone_index_buff = match self
            .get_vertex_data()
            .ok_or(anyhow!("get_vertex_data"))?
            .get_channel(
                &ChannelType::kShaderChannelBlendIndices,
                sub_mesh,
                self.inner.get_endian(),
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
        Some(self.get_sub_meshes()?.len())
    }

    fn get_bind_pose(&self) -> anyhow::Result<Supercow<Vec<crate::until::binrw_parser::Mat4>>> {
        todo!()
    }
}

impl Mesh<'_> {
    pub fn get_sub_meshes(&self) -> Option<Vec<SubMesh>> {
        self.inner
            .get_array_object_by_path("/Base/m_SubMeshes/Array")
            .and_then(|f| Some(f.into_iter().map(|i| SubMesh::new(i)).collect()))
    }

    pub fn get_index_format(&self) -> Option<i64> {
        self.inner.get_int_by_path("/Base/m_IndexFormat")
    }

    pub fn get_index_buffer(&self) -> Option<Cow<Vec<u8>>> {
        self.inner
            .get_byte_array_by_path("/Base/m_IndexBuffer/Array")
    }

    pub fn get_vertex_data(&self) -> Option<VertexData> {
        self.inner
            .get_object_by_path("/Base/m_VertexData")
            .and_then(|f| Some(VertexData::new(f)))
    }
}

def_type_tree_class!(SubMesh);

impl SubMesh<'_> {
    pub fn get_first_byte(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/firstByte")
    }
    pub fn get_index_count(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/indexCount")
    }
    pub fn get_first_vertex(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/firstVertex")
    }
    pub fn get_vertex_count(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/vertexCount")
    }
}

def_type_tree_class!(VertexData);
def_type_tree_class!(Channel);

impl Channel<'_> {
    pub fn get_stream(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/stream")
    }
    pub fn get_offset(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/offset")
    }
    pub fn get_format(&self) -> Option<VertexFormat> {
        self.inner
            .get_uint_by_path("/Base/format")
            .and_then(|t| VertexFormat::try_from(t as u8).ok())
    }
    pub fn get_dimension(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/dimension")
    }
}

impl VertexData<'_> {
    pub fn get_channels(&self) -> Option<Vec<Channel>> {
        self.inner
            .get_array_object_by_path("/Base/m_Channels/Array")
            .and_then(|f| Some(f.into_iter().map(|i| Channel::new(i)).collect()))
    }

    pub fn get_vertex_count(&self) -> Option<u64> {
        self.inner.get_uint_by_path("/Base/m_VertexCount")
    }

    pub fn get_data(&self) -> Option<Cow<Vec<u8>>> {
        self.inner.get_byte_array_by_path("/Base/m_DataSize")
    }
}

impl VertexData<'_> {
    fn get_stream_offset(&self, stream: u8) -> anyhow::Result<usize> {
        let mut offset = 0;
        for s in 0..stream {
            offset += self.get_stream_stride(s)?
                * (self.get_vertex_count().ok_or(anyhow!("get_vertex_count"))? as usize);
            if offset % 16 != 0 {
                offset += 16 - (offset % 16);
            }
        }
        Ok(offset)
    }

    fn get_stream_stride(&self, stream: u8) -> anyhow::Result<usize> {
        let mut stride = 0u64;
        for channel in &self.get_channels().ok_or(anyhow!("get_channels"))? {
            if channel.get_stream().ok_or(anyhow!("get_stream"))? == stream as u64 {
                stride += get_format_size(channel.get_format().ok_or(anyhow!("get_format"))?) as u64
                    * channel.get_dimension().ok_or(anyhow!("get_dimension"))?
            }
        }
        Ok(stride as usize)
    }

    fn get_channel(
        &self,
        channel: &ChannelType,
        sub_mesh: &SubMesh,
        endian: binrw::Endian,
    ) -> anyhow::Result<StreamBuff> {
        let channel =
            &self.get_channels().ok_or(anyhow!("get_channels"))?[channel.clone() as u8 as usize];
        let offset =
            self.get_stream_offset(channel.get_stream().ok_or(anyhow!("get_stream"))? as u8)?;
        let stride =
            self.get_stream_stride(channel.get_stream().ok_or(anyhow!("get_stream"))? as u8)?;
        let buff = self.get_data().ok_or(anyhow!("get_data"))?;
        let mut reader = Cursor::new(buff.as_ref());
        let op = ReadOptions::new(endian);
        match &channel.get_format().ok_or(anyhow!("get_format"))? {
            VertexFormat::Float => {
                let mut buff = vec![];
                for i in sub_mesh
                    .get_first_vertex()
                    .ok_or(anyhow!("get_first_vertex"))?
                    ..sub_mesh
                        .get_vertex_count()
                        .ok_or(anyhow!("get_vertex_count"))?
                        + sub_mesh
                            .get_first_vertex()
                            .ok_or(anyhow!("get_first_vertex"))?
                {
                    reader.seek(SeekFrom::Start(
                        offset as u64
                            + i as u64 * stride as u64
                            + channel.get_offset().ok_or(anyhow!("get_offset"))?,
                    ))?;
                    let sbuff = <Vec<f32>>::read_options(
                        &mut reader,
                        &op,
                        VecArgs {
                            count: channel.get_dimension().ok_or(anyhow!("get_dimension"))?
                                as usize,
                            inner: (),
                        },
                    )?;
                    buff.push(sbuff);
                }
                Ok(StreamBuff::Float(buff))
            }
            VertexFormat::Float16 => {
                let mut buff = vec![];
                for i in sub_mesh
                    .get_first_vertex()
                    .ok_or(anyhow!("get_first_vertex"))?
                    ..sub_mesh
                        .get_vertex_count()
                        .ok_or(anyhow!("get_vertex_count"))?
                        + sub_mesh
                            .get_first_vertex()
                            .ok_or(anyhow!("get_first_vertex"))?
                {
                    reader.seek(SeekFrom::Start(
                        offset as u64
                            + i as u64 * stride as u64
                            + channel.get_offset().ok_or(anyhow!("get_offset"))? as u64,
                    ))?;
                    let sbuff = <Vec<u16>>::read_options(
                        &mut reader,
                        &op,
                        VecArgs {
                            count: channel.get_dimension().ok_or(anyhow!("get_dimension"))?
                                as usize,
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
                for i in sub_mesh
                    .get_first_vertex()
                    .ok_or(anyhow!("get_first_vertex"))?
                    ..sub_mesh
                        .get_vertex_count()
                        .ok_or(anyhow!("get_vertex_count"))?
                        + sub_mesh
                            .get_first_vertex()
                            .ok_or(anyhow!("get_first_vertex"))?
                {
                    reader.seek(SeekFrom::Start(
                        offset as u64
                            + i as u64 * stride as u64
                            + channel.get_offset().ok_or(anyhow!("get_offset"))? as u64,
                    ))?;
                    let sbuff = <Vec<u32>>::read_options(
                        &mut reader,
                        &op,
                        VecArgs {
                            count: channel.get_dimension().ok_or(anyhow!("get_dimension"))?
                                as usize,
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
