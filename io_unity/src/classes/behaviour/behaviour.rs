use super::BehaviourObject;
use crate::{classes::component::Component, until::binrw_parser::U8Bool, SerializedFileMetadata};
use binrw::binrw;

impl BehaviourObject for Behaviour {}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Behaviour {
    #[brw(args_raw = args)]
    component: Component,
    #[brw(align_after(4))]
    enabled: U8Bool,
}
