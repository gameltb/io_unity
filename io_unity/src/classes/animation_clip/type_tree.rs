use crate::type_tree::TypeTreeObject;

use crate::def_type_tree_class;

use super::AnimationClipObject;

def_type_tree_class!(AnimationClip);

impl AnimationClipObject for AnimationClip {
    fn get_name(&self) -> String {
        todo!()
    }
}

impl AnimationClip {}
