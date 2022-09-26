use binrw::binrw;

use crate::SerializedFileMetadata;

use super::PPtrObject;

impl PPtrObject for PPtr {
    fn get_path_id(&self) -> i64 {
        self.path_id as i64
    }
}
#[binrw]
#[brw(import_raw(_args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct PPtr {
    file_id: i32,
    path_id: i32,
}
