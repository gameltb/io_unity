use super::{NamedObject, NamedObjectObject};
use crate::type_tree::convert::TryCastFrom;

impl NamedObjectObject for NamedObject<'_> {
    fn get_name(&self) -> anyhow::Result<String> {
        String::try_cast_from(self.inner, "/Base/m_Name")
    }
}
