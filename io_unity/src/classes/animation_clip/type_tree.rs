use super::AnimationClipObject;
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;
use supercow::Supercow;

def_type_tree_class!(AnimationClip);

impl AnimationClipObject for AnimationClip<'_> {
    fn get_name(&self) -> String {
        self.get_name().unwrap()
    }
}

impl AnimationClip<'_> {
    fn get_name(&self) -> Option<String> {
        self.inner.get_string_by_path("/Base/m_Name")
    }
}
