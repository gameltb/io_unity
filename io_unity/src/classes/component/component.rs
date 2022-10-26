use binrw::binrw;

use crate::{
    classes::{editor_extension::EditorExtension, p_ptr::PPtr},
    SerializedFileMetadata,
};

use super::ComponentObject;

impl ComponentObject for Component {
    fn get_game_object(&self) -> &PPtr {
        &self.game_object
    }
}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Component {
    #[brw(args_raw = args.clone())]
    editor_extension: EditorExtension,
    #[brw(args_raw = args)]
    game_object: PPtr,
}
