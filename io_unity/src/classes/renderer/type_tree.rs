use crate::type_tree::TypeTreeObject;

use crate::def_type_tree_class;

use super::RendererObject;

def_type_tree_class!(Renderer);

impl RendererObject for Renderer {
    fn get_materials(&self) -> &Vec<crate::classes::p_ptr::PPtr> {
        todo!()
    }
}

impl Renderer {}
