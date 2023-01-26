use super::RendererObject;
use crate::classes::p_ptr::PPtr;
use crate::def_type_tree_class;
use crate::type_tree::convert::TryCastFrom;
use crate::type_tree::TypeTreeObject;
use supercow::Supercow;

def_type_tree_class!(Renderer);

impl RendererObject for Renderer<'_> {
    fn get_materials(&self) -> Option<Supercow<Vec<PPtr>>> {
        Some(Supercow::owned(self.get_materials()?))
    }
}

impl Renderer<'_> {
    pub fn get_materials(&self) -> Option<Vec<PPtr>> {
        <Vec<TypeTreeObject>>::try_cast_from(&self.inner, "/Base/m_Materials/Array")
            .ok()
            .and_then(|f| Some(f.into_iter().map(|i| PPtr::new(i)).collect()))
    }
}
