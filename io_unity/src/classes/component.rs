use binrw::binrw;

use crate::SerializedFileMetadata;

use super::{editor_extension::EditorExtension, p_ptr::PPtr};

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Component {
    #[brw(args_raw = args.clone())]
    editor_extension: EditorExtension,
    #[brw(args_raw = args)]
    game_object: PPtr,
}

impl Component {
    pub fn get_game_object(&self) -> &PPtr {
        &self.game_object
    }
}
