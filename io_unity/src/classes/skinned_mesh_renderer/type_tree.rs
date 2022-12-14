use super::SkinnedMeshRendererObject;
use crate::classes::p_ptr::PPtr;
use crate::classes::renderer;
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;
use supercow::Supercow;

def_type_tree_class!(SkinnedMeshRenderer);

impl SkinnedMeshRendererObject for SkinnedMeshRenderer<'_> {
    fn get_bones(&self) -> Option<Supercow<Vec<PPtr>>> {
        Some(Supercow::owned(self.get_bones()?))
    }

    fn get_mesh(&self) -> Option<Supercow<PPtr>> {
        Some(Supercow::owned(self.get_mesh()?))
    }

    fn get_materials(&self) -> Option<Supercow<Vec<PPtr>>> {
        Some(Supercow::owned(
            renderer::type_tree::Renderer::new(&*self.inner).get_materials()?,
        ))
    }
}

impl SkinnedMeshRenderer<'_> {
    pub fn get_mesh(&self) -> Option<PPtr> {
        self.inner
            .get_object_by_path("/Base/m_Mesh")
            .and_then(|f| Some(PPtr::new(f)))
    }

    pub fn get_bones(&self) -> Option<Vec<PPtr>> {
        self.inner
            .get_array_object_by_path("/Base/m_Bones/Array")
            .and_then(|f| Some(f.into_iter().map(|i| PPtr::new(i)).collect()))
    }
}
