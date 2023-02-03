pub mod type_tree;

use crate::def_unity_class;

def_unity_class!(NamedObject);

pub trait NamedObjectObject {
    fn get_name(&self) -> anyhow::Result<String>;
}
