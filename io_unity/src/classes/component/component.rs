use super::ComponentObject;
use crate::{
    classes::{editor_extension::EditorExtension, p_ptr::PPtr},
    SerializedFileMetadata,
};
use binrw::binrw;
use supercow::Supercow;

impl ComponentObject for Component {
    fn get_game_object(&self) -> Option<Supercow<PPtr>> {
        Some(Supercow::borrowed(&self.game_object))
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
