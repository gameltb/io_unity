use binrw::binrw;

use crate::SerializedFileMetadata;

use super::renderer::Renderer;

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct MeshRenderer {
    #[brw(args_raw = args)]
    renderer: Renderer,
}
