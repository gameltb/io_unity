use super::{NamedObject, NamedObjectObject};
use crate::type_tree::convert::TryCastFrom;

impl NamedObjectObject for NamedObject {
    fn get_name(&self) -> Option<String> {
        String::try_cast_from(&self.inner, "/Base/m_Name").ok()
    }
}
