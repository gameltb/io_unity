use std::io::{Cursor, Seek, SeekFrom};

use super::{
    get_format_size, BoneWeights, ChannelType, Mesh, MeshObject, StreamBuff, VertexFormat,
};

use crate::classes::CastRef;
use crate::def_unity_class;
use crate::type_tree::convert::TryCastFrom;
use crate::type_tree::TypeTreeObjectRef;

use binrw::{BinRead, VecArgs};

impl MeshObject for Mesh<'_> {
    fn get_index_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<u32>> {
        let binding = self.get_sub_meshes()?;
        let sub_mesh: SubMesh = binding
            .get(sub_mesh_id)
            .ok_or(anyhow!(format!("cannot get sub mesh at {sub_mesh_id}")))?
            .cast_as();

        let buff = self.get_index_buffer()?;
        let mut reader = Cursor::new(buff);
        reader.seek(SeekFrom::Start(sub_mesh.get_first_byte()?))?;

        let first_vertex = sub_mesh.get_first_vertex()?;

        if self.get_index_format()? == 0 {
            let buff = <Vec<u16>>::read_options(
                &mut reader,
                self.inner.get_endian(),
                VecArgs {
                    count: sub_mesh.get_index_count()? as usize,
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
                self.inner.get_endian(),
                VecArgs {
                    count: sub_mesh.get_index_count()? as usize,
                    inner: (),
                },
            )?;
            Ok(buff.into_iter().map(|i| i - first_vertex as u32).collect())
        }
    }

    fn get_vertex_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>> {
        let binding = self.get_sub_meshes()?;
        let sub_mesh: SubMesh = binding
            .get(sub_mesh_id)
            .ok_or(anyhow!(format!("cannot get sub mesh at {sub_mesh_id}")))?
            .cast_as();
        let vertex_data_obj = self.get_vertex_data()?;
        let vertex_data: VertexData = (&vertex_data_obj).cast_as();

        Ok(match vertex_data.get_channel_stream_buff(
            &ChannelType::kShaderChannelVertex,
            &sub_mesh,
            self.inner.get_endian(),
        )? {
            StreamBuff::Float(buff) => buff,
            _ => unreachable!(),
        }
        .concat())
    }

    fn get_normal_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>> {
        let binding = self.get_sub_meshes()?;
        let sub_mesh: SubMesh = binding
            .get(sub_mesh_id)
            .ok_or(anyhow!(format!("cannot get sub mesh at {sub_mesh_id}")))?
            .cast_as();
        let vertex_data_obj = self.get_vertex_data()?;
        let vertex_data: VertexData = (&vertex_data_obj).cast_as();

        Ok(match vertex_data.get_channel_stream_buff(
            &ChannelType::kShaderChannelNormal,
            &sub_mesh,
            self.inner.get_endian(),
        )? {
            StreamBuff::Float(buff) => buff,
            _ => unreachable!(),
        }
        .concat())
    }

    fn get_uv0_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<f32>> {
        let binding = self.get_sub_meshes()?;
        let sub_mesh: SubMesh = binding
            .get(sub_mesh_id)
            .ok_or(anyhow!(format!("cannot get sub mesh at {sub_mesh_id}")))?
            .cast_as();

        let vertex_data_obj = self.get_vertex_data()?;
        let vertex_data: VertexData = (&vertex_data_obj).cast_as();

        Ok(match vertex_data.get_channel_stream_buff(
            &ChannelType::kShaderChannelTexCoord0,
            &sub_mesh,
            self.inner.get_endian(),
        )? {
            StreamBuff::Float(buff) => buff,
            _ => unreachable!(),
        }
        .concat())
    }

    fn get_bone_weights_buff(&self, sub_mesh_id: usize) -> anyhow::Result<Vec<BoneWeights>> {
        let binding = self.get_sub_meshes()?;
        let sub_mesh: SubMesh = binding
            .get(sub_mesh_id)
            .ok_or(anyhow!(format!("cannot get sub mesh at {sub_mesh_id}")))?
            .cast_as();
        let vertex_data_obj = self.get_vertex_data()?;
        let vertex_data: VertexData = (&vertex_data_obj).cast_as();

        let weight_buff = match vertex_data.get_channel_stream_buff(
            &ChannelType::kShaderChannelBlendWeight,
            &sub_mesh,
            self.inner.get_endian(),
        )? {
            StreamBuff::Float(buff) => buff,
            _ => unreachable!(),
        };
        let bone_index_buff = match vertex_data.get_channel_stream_buff(
            &ChannelType::kShaderChannelBlendIndices,
            &sub_mesh,
            self.inner.get_endian(),
        )? {
            StreamBuff::I64(buff) => buff,
            _ => unreachable!(),
        };

        let mut buff = Vec::new();
        for (weight, bone_index) in weight_buff.into_iter().zip(bone_index_buff) {
            buff.push(BoneWeights { weight, bone_index });
        }
        Ok(buff)
    }

    fn get_sub_mesh_count(&self) -> anyhow::Result<usize> {
        Ok(self.get_sub_meshes()?.len())
    }
}

impl Mesh<'_> {
    pub fn get_sub_meshes(&self) -> anyhow::Result<Vec<TypeTreeObjectRef>> {
        <Vec<TypeTreeObjectRef>>::try_cast_from(self.inner, "/Base/m_SubMeshes/Array")
    }

    pub fn get_index_format(&self) -> anyhow::Result<i64> {
        i64::try_cast_from(self.inner, "/Base/m_IndexFormat")
    }

    pub fn get_index_buffer(&self) -> anyhow::Result<Vec<u8>> {
        <Vec<u8>>::try_cast_from(self.inner, "/Base/m_IndexBuffer/Array")
    }

    pub fn get_vertex_data(&self) -> anyhow::Result<TypeTreeObjectRef> {
        TypeTreeObjectRef::try_cast_from(self.inner, "/Base/m_VertexData")
    }
}

def_unity_class!(SubMesh);

impl SubMesh<'_> {
    pub fn get_first_byte(&self) -> anyhow::Result<u64> {
        u64::try_cast_from(self.inner, "/Base/firstByte")
    }
    pub fn get_index_count(&self) -> anyhow::Result<u64> {
        u64::try_cast_from(self.inner, "/Base/indexCount")
    }
    pub fn get_first_vertex(&self) -> anyhow::Result<u64> {
        u64::try_cast_from(self.inner, "/Base/firstVertex")
    }
    pub fn get_vertex_count(&self) -> anyhow::Result<u64> {
        u64::try_cast_from(self.inner, "/Base/vertexCount")
    }
}

def_unity_class!(VertexData);
def_unity_class!(Channel);

impl Channel<'_> {
    pub fn get_stream(&self) -> anyhow::Result<u64> {
        u64::try_cast_from(self.inner, "/Base/stream")
    }
    pub fn get_offset(&self) -> anyhow::Result<u64> {
        u64::try_cast_from(self.inner, "/Base/offset")
    }
    pub fn get_format(&self) -> anyhow::Result<VertexFormat> {
        Ok(u8::try_cast_from(self.inner, "/Base/format").map(VertexFormat::try_from)??)
    }
    pub fn get_dimension(&self) -> anyhow::Result<u64> {
        u64::try_cast_from(self.inner, "/Base/dimension")
    }
}

impl VertexData<'_> {
    pub fn get_channels(&self) -> anyhow::Result<Vec<TypeTreeObjectRef>> {
        <Vec<TypeTreeObjectRef>>::try_cast_from(self.inner, "/Base/m_Channels/Array")
    }

    pub fn get_vertex_count(&self) -> anyhow::Result<u64> {
        u64::try_cast_from(self.inner, "/Base/m_VertexCount")
    }

    pub fn get_data(&self) -> anyhow::Result<Vec<u8>> {
        <Vec<u8>>::try_cast_from(self.inner, "/Base/m_DataSize")
    }
}

impl VertexData<'_> {
    fn get_stream_offset(&self, stream: u8) -> anyhow::Result<usize> {
        let mut offset = 0;
        for s in 0..stream {
            offset += self.get_stream_stride(s)? * (self.get_vertex_count()? as usize);
            if offset % 16 != 0 {
                offset += 16 - (offset % 16);
            }
        }
        Ok(offset)
    }

    fn get_stream_stride(&self, stream: u8) -> anyhow::Result<usize> {
        let mut stride = 0u64;
        for channel in &self.get_channels()? {
            let channel: Channel = channel.cast_as();
            if channel.get_stream()? == stream as u64 {
                stride += get_format_size(channel.get_format()?) as u64 * channel.get_dimension()?
            }
        }
        Ok(stride as usize)
    }

    fn get_channel_stream_buff(
        &self,
        channel: &ChannelType,
        sub_mesh: &SubMesh,
        endian: binrw::Endian,
    ) -> anyhow::Result<StreamBuff> {
        let channel = &self.get_channels()?[channel.clone() as u8 as usize];
        let channel: Channel = channel.cast_as();

        match &channel.get_format()? {
            VertexFormat::Float => {
                let buff = self.get_channel(&channel, sub_mesh, endian)?;
                Ok(StreamBuff::Float(buff))
            }
            VertexFormat::Float16 => {
                let buff = self
                    .get_channel(&channel, sub_mesh, endian)?
                    .into_iter()
                    .map(|f| {
                        f.into_iter()
                            .map(|f| half::f16::from_bits(f).to_f32())
                            .collect()
                    })
                    .collect();
                Ok(StreamBuff::Float(buff))
            }
            VertexFormat::UNorm8 => {
                let buff = self
                    .get_channel::<u8>(&channel, sub_mesh, endian)?
                    .into_iter()
                    .map(|f| f.into_iter().map(|f| f as f32 / 255.0).collect())
                    .collect();
                Ok(StreamBuff::Float(buff))
            }
            VertexFormat::SNorm8 => {
                let buff = self
                    .get_channel::<i8>(&channel, sub_mesh, endian)?
                    .into_iter()
                    .map(|f| f.into_iter().map(|f| (f as f32 / 127.0).max(1.0)).collect())
                    .collect();
                Ok(StreamBuff::Float(buff))
            }
            VertexFormat::UNorm16 => {
                let buff = self
                    .get_channel::<u16>(&channel, sub_mesh, endian)?
                    .into_iter()
                    .map(|f| f.into_iter().map(|f| f as f32 / 65535.0).collect())
                    .collect();
                Ok(StreamBuff::Float(buff))
            }
            VertexFormat::SNorm16 => {
                let buff = self
                    .get_channel::<i16>(&channel, sub_mesh, endian)?
                    .into_iter()
                    .map(|f| {
                        f.into_iter()
                            .map(|f| (f as f32 / 32767.0).max(1.0))
                            .collect()
                    })
                    .collect();
                Ok(StreamBuff::Float(buff))
            }
            VertexFormat::UInt8 => {
                let buff = self
                    .get_channel::<u8>(&channel, sub_mesh, endian)?
                    .into_iter()
                    .map(|f| f.into_iter().map(|f| f as i64).collect())
                    .collect();
                Ok(StreamBuff::I64(buff))
            }
            VertexFormat::SInt8 => {
                let buff = self
                    .get_channel::<i8>(&channel, sub_mesh, endian)?
                    .into_iter()
                    .map(|f| f.into_iter().map(|f| f as i64).collect())
                    .collect();
                Ok(StreamBuff::I64(buff))
            }
            VertexFormat::UInt16 => {
                let buff = self
                    .get_channel::<u16>(&channel, sub_mesh, endian)?
                    .into_iter()
                    .map(|f| f.into_iter().map(|f| f as i64).collect())
                    .collect();
                Ok(StreamBuff::I64(buff))
            }
            VertexFormat::SInt16 => {
                let buff = self
                    .get_channel::<i16>(&channel, sub_mesh, endian)?
                    .into_iter()
                    .map(|f| f.into_iter().map(|f| f as i64).collect())
                    .collect();
                Ok(StreamBuff::I64(buff))
            }
            VertexFormat::UInt32 => {
                let buff = self
                    .get_channel::<u32>(&channel, sub_mesh, endian)?
                    .into_iter()
                    .map(|f| f.into_iter().map(|f| f as i64).collect())
                    .collect();
                Ok(StreamBuff::I64(buff))
            }
            VertexFormat::SInt32 => {
                let buff = self
                    .get_channel::<i32>(&channel, sub_mesh, endian)?
                    .into_iter()
                    .map(|f| f.into_iter().map(|f| f as i64).collect())
                    .collect();
                Ok(StreamBuff::I64(buff))
            }
        }
    }

    fn get_channel<T: for<'a> BinRead<Args<'a> = ()> + 'static>(
        &self,
        channel: &Channel,
        sub_mesh: &SubMesh,
        endian: binrw::Endian,
    ) -> anyhow::Result<Vec<Vec<T>>> {
        let offset = self.get_stream_offset(channel.get_stream()? as u8)?;
        let stride = self.get_stream_stride(channel.get_stream()? as u8)?;
        let buff = self.get_data()?;
        let mut reader = Cursor::new(buff);

        let mut buff = vec![];
        for i in sub_mesh.get_first_vertex()?
            ..sub_mesh.get_vertex_count()? + sub_mesh.get_first_vertex()?
        {
            reader.seek(SeekFrom::Start(
                offset as u64 + i * stride as u64 + channel.get_offset()?,
            ))?;
            let sbuff = <Vec<T>>::read_options(
                &mut reader,
                endian,
                VecArgs {
                    count: channel.get_dimension()? as usize,
                    inner: (),
                },
            )?;
            buff.push(sbuff);
        }
        Ok(buff)
    }
}
