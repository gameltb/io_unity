use crate::type_tree::TypeTreeObject;

use crate::def_type_tree_class;

use super::PPtrObject;

def_type_tree_class!(PPtr);

impl PPtrObject for PPtr {
    fn get_path_id(&self) -> i64 {
        todo!()
    }
}

impl PPtr {}
