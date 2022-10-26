use crate::type_tree::TypeTreeObject;

use crate::def_type_tree_class;

use super::SkinnedMeshRendererObject;

def_type_tree_class!(SkinnedMeshRenderer);

impl SkinnedMeshRendererObject for SkinnedMeshRenderer {
    fn get_bones(&self) -> &Vec<crate::classes::p_ptr::PPtr> {
        todo!()
    }

    fn get_mesh(&self) -> &crate::classes::p_ptr::PPtr {
        todo!()
    }

    fn get_materials(&self) -> &Vec<crate::classes::p_ptr::PPtr> {
        todo!()
    }
}

impl SkinnedMeshRenderer {}
