use super::SkinnedMeshRendererObject;
use crate::classes::p_ptr::PPtr;
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;
use supercow::Supercow;

def_type_tree_class!(SkinnedMeshRenderer);

impl SkinnedMeshRendererObject for SkinnedMeshRenderer<'_> {
    fn get_bones(&self) -> &Vec<PPtr> {
        todo!()
    }

    fn get_mesh(&self) -> &PPtr {
        todo!()
    }

    fn get_materials(&self) -> &Vec<PPtr> {
        todo!()
    }
}

impl SkinnedMeshRenderer<'_> {}
