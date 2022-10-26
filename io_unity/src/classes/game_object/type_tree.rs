use crate::type_tree::TypeTreeObject;

use crate::def_type_tree_class;

use super::GameObjectObject;

def_type_tree_class!(GameObject);

impl GameObjectObject for GameObject {
    fn get_name(&self) -> &crate::until::binrw_parser::AlignedString {
        todo!()
    }
}

impl GameObject {}
