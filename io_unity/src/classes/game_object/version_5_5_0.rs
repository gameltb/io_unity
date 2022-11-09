use super::GameObjectObject;
use crate::{
    classes::{editor_extension::EditorExtension, p_ptr::PPtr},
    until::binrw_parser::AlignedString,
    SerializedFileMetadata,
};
use binrw::binrw;

impl GameObjectObject for GameObject {
    fn get_name(&self) -> Option<String> {
        Some(self.name.to_string())
    }
}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct GameObject {
    #[brw(args_raw = args.clone())]
    editor_extension: EditorExtension,
    component_size: i32,
    #[br(count = component_size ,args { inner: args })]
    #[bw(args_raw = args.clone())]
    components: Vec<PPtr>,
    layer: i32,
    name: AlignedString,
}
