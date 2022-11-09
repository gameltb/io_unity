use super::AnimationClipObject;
use crate::classes::named_object::{self, NamedObjectObject};
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;
use supercow::Supercow;

def_type_tree_class!(AnimationClip);

impl named_object::DownCast for AnimationClip<'_> {
    fn downcast<'a>(&'a self) -> Supercow<Box<dyn NamedObjectObject + Send + 'a>> {
        Supercow::owned(Box::new(named_object::type_tree::NamedObject::new(
            &*self.inner,
        )))
    }
}

impl AnimationClipObject for AnimationClip<'_> {}

impl AnimationClip<'_> {}
