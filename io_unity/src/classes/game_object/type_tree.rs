use super::GameObjectObject;
use crate::def_type_tree_class;
use crate::type_tree::convert::TryCastFrom;
use crate::type_tree::TypeTreeObject;
use supercow::Supercow;

def_type_tree_class!(GameObject);

impl GameObjectObject for GameObject<'_> {
    fn get_name(&self) -> Option<String> {
        self.get_name()
    }
}

impl GameObject<'_> {
    fn get_name(&self) -> Option<String> {
        String::try_cast_from(&self.inner, "/Base/m_Name").ok()
    }
}
