use super::{PPtr, PPtrObject};
use crate::type_tree::convert::TryCastFrom;

impl PPtrObject for PPtr<'_> {
    fn get_file_id(&self) -> anyhow::Result<i64> {
        i64::try_cast_from(self.inner, "/Base/m_FileID")
    }

    fn get_path_id(&self) -> anyhow::Result<i64> {
        i64::try_cast_from(self.inner, "/Base/m_PathID")
    }
}
