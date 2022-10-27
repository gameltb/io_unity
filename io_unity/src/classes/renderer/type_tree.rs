use super::RendererObject;
use crate::classes::p_ptr::PPtr;
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;
use supercow::Supercow;

def_type_tree_class!(Renderer);

impl RendererObject for Renderer<'_> {
    fn get_materials(&self) -> &Vec<PPtr> {
        todo!()
    }
}

impl Renderer<'_> {}
