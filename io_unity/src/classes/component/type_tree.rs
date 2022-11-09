use super::ComponentObject;
use crate::{classes::p_ptr::PPtr, def_type_tree_class, type_tree::TypeTreeObject};
use supercow::Supercow;

def_type_tree_class!(Component);

impl ComponentObject for Component<'_> {
    fn get_game_object(&self) -> Option<Supercow<PPtr>> {
        Some(Supercow::owned(self.get_game_object()?))
    }
}

impl Component<'_> {
    pub fn get_game_object(&self) -> Option<PPtr> {
        self.inner
            .get_object_by_path("/Base/m_GameObject")
            .and_then(|f| Some(PPtr::new(f)))
    }
}
