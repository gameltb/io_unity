use super::ComponentObject;
use crate::{
    classes::p_ptr::PPtr,
    def_type_tree_class,
    type_tree::{convert::TryCastFrom, TypeTreeObject},
};
use supercow::Supercow;

def_type_tree_class!(Component);

impl ComponentObject for Component<'_> {
    fn get_game_object(&self) -> Option<Supercow<PPtr>> {
        Some(Supercow::owned(self.get_game_object()?))
    }
}

impl Component<'_> {
    pub fn get_game_object(&self) -> Option<PPtr> {
        TypeTreeObject::try_cast_from(&self.inner, "/Base/m_GameObject")
            .ok()
            .and_then(|f| Some(PPtr::new(f)))
    }
}
