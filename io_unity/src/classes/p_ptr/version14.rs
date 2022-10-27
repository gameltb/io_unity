use super::PPtrObject;
use crate::SerializedFileMetadata;
use binrw::binrw;

impl PPtrObject for PPtr {
    fn get_path_id(&self) -> i64 {
        self.path_id
    }
}

#[binrw]
#[brw(import_raw(_args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct PPtr {
    file_id: i32,
    path_id: i64,
}
