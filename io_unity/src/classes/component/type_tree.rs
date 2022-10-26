use crate::{def_type_tree_class, type_tree::TypeTreeObject};

use super::ComponentObject;

impl ComponentObject for Component {
    fn get_game_object(&self) -> &crate::classes::p_ptr::PPtr {
        todo!()
    }
}

def_type_tree_class!(Component);
