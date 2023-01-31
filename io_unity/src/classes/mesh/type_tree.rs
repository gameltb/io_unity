use std::io::{Cursor, Seek, SeekFrom};

use super::{
    get_format_size, BoneWeights, ChannelType, Mesh, MeshObject, StreamBuff, VertexFormat,
};

use crate::classes::CastRef;
use crate::def_unity_class;
use crate::type_tree::convert::TryCastFrom;
use crate::type_tree::convert::TryCastRefFrom;
use crate::type_tree::TypeTreeObjectRef;

use crate::type_tree::TypeTreeObject;
use binrw::{BinRead, ReadOptions, VecArgs};

impl MeshObject for Mesh<'_> {
    fn get_index_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<u32>> {
        let binding = self.get_sub_meshes().ok_or(anyhow!("sub_meshs"))?;
        let sub_mesh: SubMesh = binding
            .get(sub_mesh_id)
            .ok_or(anyhow!("sub_mesh"))?
            .cast_as();

        let buff = self.get_index_buffer().ok_or(anyhow!("get_index_buffer"))?;
        let mut reader = Cursor::new(buff);
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
        let sub_mesh: SubMesh = binding
            .get(sub_mesh_id)
            .ok_or(anyhow!("sub_mesh"))?
            .cast_as();
        let vertex_data_obj = self.get_vertex_data().ok_or(anyhow!("get_vertex_data"))?;
        let vertex_data: VertexData = (&vertex_data_obj).cast_as();

        Ok(match vertex_data.get_channel(
            &ChannelType::kShaderChannelVertex,
            &sub_mesh,
            self.inner.get_endian(),
        )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        }
        .concat())
    }

    fn get_normal_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>> {
        let binding = self.get_sub_meshes().ok_or(anyhow!("sub_meshs"))?;
        let sub_mesh: SubMesh = binding
            .get(sub_mesh_id)
            .ok_or(anyhow!("sub_mesh"))?
            .cast_as();
        let vertex_data_obj = self.get_vertex_data().ok_or(anyhow!("get_vertex_data"))?;
        let vertex_data: VertexData = (&vertex_data_obj).cast_as();

        Ok(match vertex_data.get_channel(
            &ChannelType::kShaderChannelNormal,
            &sub_mesh,
            self.inner.get_endian(),
        )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        }
        .concat())
    }

    fn get_uv0_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>> {
        let binding = self.get_sub_meshes().ok_or(anyhow!("sub_meshs"))?;
        let sub_mesh: SubMesh = binding
            .get(sub_mesh_id)
            .ok_or(anyhow!("sub_mesh"))?
            .cast_as();

        let vertex_data_obj = self.get_vertex_data().ok_or(anyhow!("get_vertex_data"))?;
        let vertex_data: VertexData = (&vertex_data_obj).cast_as();

        Ok(match vertex_data.get_channel(
            &ChannelType::kShaderChannelTexCoord0,
            &sub_mesh,
            self.inner.get_endian(),
        )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        }
        .concat())
    }

    fn get_bone_weights_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<BoneWeights>> {
        let binding = self.get_sub_meshes().ok_or(anyhow!("get_sub_meshes"))?;
        let sub_mesh: SubMesh = binding
            .get(sub_mesh_id)
            .ok_or(anyhow!("sub_mesh"))?
            .cast_as();
        let vertex_data_obj = self.get_vertex_data().ok_or(anyhow!("get_vertex_data"))?;
        let vertex_data: VertexData = (&vertex_data_obj).cast_as();

        let weight_buff = match vertex_data.get_channel(
            &ChannelType::kShaderChannelBlendWeight,
            &sub_mesh,
            self.inner.get_endian(),
        )? {
            StreamBuff::Float(buff) => buff,
            StreamBuff::I32(_) | StreamBuff::U32(_) => unreachable!(),
        };
        let bone_index_buff = match vertex_data.get_channel(
            &ChannelType::kShaderChannelBlendIndices,
            &sub_mesh,
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

    fn get_bind_pose(&self) -> anyhow::Result<Vec<crate::until::binrw_parser::Mat4>> {
        todo!()
    }
}

impl Mesh<'_> {
    pub fn get_sub_meshes(&self) -> Option<Vec<TypeTreeObjectRef>> {
        <Vec<TypeTreeObjectRef>>::try_cast_from(self.inner, "/Base/m_SubMeshes/Array").ok()
    }

    pub fn get_index_format(&self) -> Option<i64> {
        i64::try_cast_from(self.inner, "/Base/m_IndexFormat").ok()
    }

    pub fn get_index_buffer(&self) -> Option<Vec<u8>> {
        Some(<Vec<u8>>::try_cast_from(self.inner, "/Base/m_IndexBuffer/Array").ok()?)
    }

    pub fn get_vertex_data(&self) -> Option<TypeTreeObjectRef> {
        TypeTreeObjectRef::try_cast_from(self.inner, "/Base/m_VertexData").ok()
    }
}

def_unity_class!(SubMesh);

impl SubMesh<'_> {
    pub fn get_first_byte(&self) -> Option<u64> {
        u64::try_cast_from(self.inner, "/Base/firstByte").ok()
    }
    pub fn get_index_count(&self) -> Option<u64> {
        u64::try_cast_from(self.inner, "/Base/indexCount").ok()
    }
    pub fn get_first_vertex(&self) -> Option<u64> {
        u64::try_cast_from(self.inner, "/Base/firstVertex").ok()
    }
    pub fn get_vertex_count(&self) -> Option<u64> {
        u64::try_cast_from(self.inner, "/Base/vertexCount").ok()
    }
}

def_unity_class!(VertexData);
def_unity_class!(Channel);

impl Channel<'_> {
    pub fn get_stream(&self) -> Option<u64> {
        u64::try_cast_from(self.inner, "/Base/stream").ok()
    }
    pub fn get_offset(&self) -> Option<u64> {
        u64::try_cast_from(self.inner, "/Base/offset").ok()
    }
    pub fn get_format(&self) -> Option<VertexFormat> {
        u8::try_cast_from(self.inner, "/Base/format")
            .ok()
            .and_then(|t| VertexFormat::try_from(t).ok())
    }
    pub fn get_dimension(&self) -> Option<u64> {
        u64::try_cast_from(self.inner, "/Base/dimension").ok()
    }
}

impl VertexData<'_> {
    pub fn get_channels(&self) -> Option<Vec<TypeTreeObjectRef>> {
        <Vec<TypeTreeObjectRef>>::try_cast_from(self.inner, "/Base/m_Channels/Array").ok()
    }

    pub fn get_vertex_count(&self) -> Option<u64> {
        u64::try_cast_from(self.inner, "/Base/m_VertexCount").ok()
    }

    pub fn get_data(&self) -> Option<Vec<u8>> {
        Some(<Vec<u8>>::try_cast_from(self.inner, "/Base/m_DataSize").ok()?)
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
            let channel: Channel = channel.cast_as();
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
        let channel: Channel = channel.cast_as();

        let offset =
            self.get_stream_offset(channel.get_stream().ok_or(anyhow!("get_stream"))? as u8)?;
        let stride =
            self.get_stream_stride(channel.get_stream().ok_or(anyhow!("get_stream"))? as u8)?;
        let buff = self.get_data().ok_or(anyhow!("get_data"))?;
        let mut reader = Cursor::new(buff);
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
