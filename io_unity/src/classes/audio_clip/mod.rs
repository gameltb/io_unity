pub mod type_tree;

use crate::{def_unity_class, unity_asset_view::UnityAssetViewer};

use std::{borrow::Cow, fmt};

def_unity_class!(AudioClip);

pub trait AudioClipObject: fmt::Debug {
    fn get_audio_data(&self, viewer: &UnityAssetViewer) -> anyhow::Result<Cow<Vec<u8>>>;
}
