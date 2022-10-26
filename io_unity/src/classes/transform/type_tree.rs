use glam::Mat4;

use crate::classes::component::Component;
use crate::classes::p_ptr::PPtr;
use crate::type_tree::TypeTreeObject;

use crate::def_type_tree_class;

use super::TransformObject;

def_type_tree_class!(Transform);

impl TransformObject for Transform {
    fn get_component(&self) -> &Component {
        todo!()
    }

    fn get_father(&self) -> &PPtr {
        todo!()
    }

    fn get_local_mat(&self) -> Mat4 {
        todo!()
    }
}

impl Transform {}
