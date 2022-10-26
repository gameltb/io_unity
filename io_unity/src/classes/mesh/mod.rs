pub mod type_tree;
pub mod version_2018_2_0;
pub mod version_2020_0_0;

use std::{
    fmt,
    io::{Read, Seek, SeekFrom, Write},
};

use binrw::{binrw, BinRead, BinResult, BinWrite, ReadOptions, WriteOptions};

use crate::{
    def_unity_class,
    until::{binrw_parser::Mat4, UnityVersion},
    SerializedFileMetadata,
};

use crate::type_tree::TypeTreeObject;

def_unity_class!(Mesh, MeshObject);

pub trait MeshObject: fmt::Debug {
    fn get_index_buff(&self, sub_mesh_id: usize) -> Vec<u32>;
    fn get_vertex_buff(&self, sub_mesh_id: usize) -> Vec<f32>;
    fn get_normal_buff(&self, sub_mesh_id: usize) -> Vec<f32>;
    fn get_uv0_buff(&self, sub_mesh_id: usize) -> Vec<f32>;
    fn get_sub_mesh_count(&self) -> usize;
    fn get_bone_weights_buff(&self, sub_mesh_id: usize) -> Vec<BoneWeights>;
    fn get_bind_pose(&self) -> &Vec<Mat4>;
}

impl BinRead for Mesh {
    type Args = SerializedFileMetadata;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        if args.unity_version >= UnityVersion::new(vec![2020], None) {
            return Ok(Mesh(Box::new(version_2020_0_0::Mesh::read_options(
                reader, options, args,
            )?)));
        } else if args.unity_version >= UnityVersion::new(vec![2018, 2], None) {
            return Ok(Mesh(Box::new(version_2018_2_0::Mesh::read_options(
                reader, options, args,
            )?)));
        }
        Err(binrw::Error::NoVariantMatch {
            pos: reader.seek(SeekFrom::Current(0)).unwrap(),
        })
    }
}

impl BinWrite for Mesh {
    type Args = SerializedFileMetadata;

    fn write_options<W: Write + Seek>(
        &self,
        _writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        Ok(())
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
