use super::BehaviourObject;
use crate::{def_type_tree_class, type_tree::TypeTreeObject};
use supercow::Supercow;

def_type_tree_class!(Behaviour);

impl BehaviourObject for Behaviour<'_> {}
