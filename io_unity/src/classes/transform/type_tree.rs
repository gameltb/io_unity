use super::TransformObject;
use crate::classes::component;
use crate::classes::p_ptr::PPtr;
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;
use glam::Mat4;
use supercow::Supercow;

def_type_tree_class!(Transform);

impl TransformObject for Transform<'_> {
    fn get_game_object(&self) -> Supercow<PPtr> {
        Supercow::owned(
            component::type_tree::Component::new(&*self.inner)
                .get_game_object()
                .unwrap(),
        )
    }

    fn get_father(&self) -> Supercow<PPtr> {
        Supercow::owned(self.get_father().unwrap())
    }

    fn get_local_mat(&self) -> Mat4 {
        todo!()
    }
}

impl Transform<'_> {
    fn get_father(&self) -> Option<PPtr> {
        self.inner
            .get_object_by_path("/Base/m_Father")
            .and_then(|f| Some(PPtr::new(f)))
    }
}
