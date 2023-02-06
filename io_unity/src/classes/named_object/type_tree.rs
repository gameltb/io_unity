use super::{NamedObject, NamedObjectObject};
use crate::{error::ReadResult, type_tree::convert::TryCastFrom};

impl NamedObjectObject for NamedObject<'_> {
    fn get_name(&self) -> ReadResult<String> {
        String::try_cast_from(self.inner, "/Base/m_Name")
    }
}
