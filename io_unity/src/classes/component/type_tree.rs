use super::ComponentObject;
use crate::{classes::p_ptr::PPtr, def_type_tree_class, type_tree::TypeTreeObject};
use supercow::Supercow;

def_type_tree_class!(Component);

impl ComponentObject for Component<'_> {
    fn get_game_object(&self) -> Supercow<PPtr> {
        Supercow::owned(self.get_game_object().unwrap())
    }
}

impl Component<'_> {
    pub fn get_game_object(&self) -> Option<PPtr> {
        self.inner
            .get_object_by_path("/Base/m_GameObject")
            .and_then(|f| Some(PPtr::new(f)))
    }
}
