use super::MonoScriptObject;
use crate::classes::named_object::{self, NamedObject, NamedObjectObject};
use crate::until::binrw_parser::*;
use crate::SerializedFileMetadata;
use binrw::binrw;
use supercow::Supercow;

impl named_object::DownCast for MonoScript {
    fn downcast<'a>(&'a self) -> Supercow<Box<dyn NamedObjectObject + Send + 'a>> {
        Supercow::borrowed(&*self.name)
    }
}

impl MonoScriptObject for MonoScript {}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct MonoScript {
    #[brw(args_raw = args)]
    name: NamedObject,
    execution_order: i32,
    properties_hash: [u8; 16],
    class_name: AlignedString,
    namespace: AlignedString,
    assembly_name: AlignedString,
}
