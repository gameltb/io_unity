use super::RendererObject;
use crate::classes::p_ptr::PPtr;
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;
use supercow::Supercow;

def_type_tree_class!(Renderer);

impl RendererObject for Renderer<'_> {
    fn get_materials(&self) -> Supercow<Vec<PPtr>> {
        Supercow::owned(self.get_materials().unwrap())
    }
}

impl Renderer<'_> {
    pub fn get_materials(&self) -> Option<Vec<PPtr>> {
        self.inner
            .get_array_object_by_path("/Base/m_Materials/Array")
            .and_then(|f| Some(f.into_iter().map(|i| PPtr::new(i)).collect()))
    }
}
