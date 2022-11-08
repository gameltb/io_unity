use super::NamedObjectObject;
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;

use supercow::Supercow;

def_type_tree_class!(NamedObject);

impl NamedObjectObject for NamedObject<'_> {
    fn get_name(&self) -> Option<String> {
        self.get_name()
    }
}

impl NamedObject<'_> {
    fn get_name(&self) -> Option<String> {
        self.inner.get_string_by_path("/Base/m_Name")
    }
}
