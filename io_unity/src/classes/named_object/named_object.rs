use super::NamedObjectObject;
use crate::until::binrw_parser::*;
use crate::SerializedFileMetadata;
use binrw::binrw;

impl NamedObjectObject for NamedObject {
    fn get_name(&self) -> Option<String> {
        Some(self.name.to_string())
    }
}

#[binrw]
#[brw(import_raw(_args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct NamedObject {
    name: AlignedString,
}
