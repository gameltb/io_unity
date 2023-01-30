pub mod type_tree;

use crate::def_unity_class;

use std::fmt;

def_unity_class!(NamedObject);

pub trait NamedObjectObject: fmt::Debug {
    fn get_name(&self) -> Option<String>;
}
