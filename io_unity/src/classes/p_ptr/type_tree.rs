use super::{PPtr, PPtrObject};
use crate::type_tree::convert::TryCastFrom;

impl PPtrObject for PPtr {
    fn get_file_id(&self) -> Option<i64> {
        i64::try_cast_from(&self.inner, "/Base/m_FileID").ok()
    }

    fn get_path_id(&self) -> Option<i64> {
        i64::try_cast_from(&self.inner, "/Base/m_PathID").ok()
    }
}
