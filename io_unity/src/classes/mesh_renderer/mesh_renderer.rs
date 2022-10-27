use super::MeshRendererObject;
use crate::{classes::renderer::Renderer, SerializedFileMetadata};
use binrw::binrw;

impl MeshRendererObject for MeshRenderer {}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct MeshRenderer {
    #[brw(args_raw = args)]
    renderer: Renderer,
}
