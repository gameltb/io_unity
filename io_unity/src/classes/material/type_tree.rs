use super::MaterialObject;
use crate::classes::named_object::{self, NamedObjectObject};
use crate::def_type_tree_class;
use crate::type_tree::TypeTreeObject;
use supercow::Supercow;

def_type_tree_class!(Material);

impl named_object::DownCast for Material<'_> {
    fn downcast<'a>(&'a self) -> Supercow<Box<dyn NamedObjectObject + Send + 'a>> {
        Supercow::owned(Box::new(named_object::type_tree::NamedObject::new(
            &*self.inner,
        )))
    }
}

impl MaterialObject for Material<'_> {}

impl Material<'_> {}
