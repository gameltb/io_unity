use crate::type_tree::TypeTreeObject;

use crate::def_type_tree_class;

use super::MeshObject;

def_type_tree_class!(Mesh);

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

    fn get_bind_pose(&self) -> &Vec<crate::until::binrw_parser::Mat4> {
        todo!()
    }
}

impl Mesh {}
