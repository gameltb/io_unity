use super::EditorExtensionObject;
use crate::{def_type_tree_class, type_tree::TypeTreeObject};
use supercow::Supercow;

def_type_tree_class!(EditorExtension);

impl EditorExtensionObject for EditorExtension<'_> {}
