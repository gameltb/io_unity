use binrw::binrw;

use crate::until::binrw_parser::*;
use crate::SerializedFileMetadata;

use super::MonoScriptObject;

impl MonoScriptObject for MonoScript {}

#[binrw]
#[brw(import_raw(_args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct MonoScript {
    name: AlignedString,
    execution_order: i32,
    properties_hash: [u8; 16],
    class_name: AlignedString,
    namespace: AlignedString,
    assembly_name: AlignedString,
}
