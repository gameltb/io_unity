pub mod type_tree;

use crate::{def_unity_class, unity_asset_view::UnityAssetViewer};

def_unity_class!(AudioClip);

pub trait AudioClipObject {
    fn get_audio_data(&self, viewer: &UnityAssetViewer) -> anyhow::Result<Vec<u8>>;
}
