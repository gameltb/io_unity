use super::PPtrObject;
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;
use supercow::Supercow;

def_type_tree_class!(PPtr);

impl PPtrObject for PPtr<'_> {
    fn get_path_id(&self) -> Option<i64> {
        self.get_path_id()
    }
}

impl PPtr<'_> {
    fn get_file_id(&self) -> Option<i64> {
        self.inner.get_int_by_path("/Base/m_FileID")
    }

    fn get_path_id(&self) -> Option<i64> {
        self.inner.get_int_by_path("/Base/m_PathID")
    }
}
