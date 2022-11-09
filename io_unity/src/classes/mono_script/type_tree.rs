use super::MonoScriptObject;
use crate::classes::named_object::{self, NamedObjectObject};
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;
use supercow::Supercow;

def_type_tree_class!(MonoScript);

impl named_object::DownCast for MonoScript<'_> {
    fn downcast<'a>(&'a self) -> Supercow<Box<dyn NamedObjectObject + Send + 'a>> {
        Supercow::owned(Box::new(named_object::type_tree::NamedObject::new(
            &*self.inner,
        )))
    }
}

impl MonoScriptObject for MonoScript<'_> {}

impl MonoScript<'_> {}
