use binrw::binrw;

use crate::{
    classes::{behaviour::Behaviour, p_ptr::PPtr},
    until::binrw_parser::AlignedString,
    SerializedFileMetadata,
};

use super::MonoBehaviourObject;

impl MonoBehaviourObject for MonoBehaviour {}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct MonoBehaviour {
    #[brw(args_raw = args.clone())]
    behaviour: Behaviour,
    #[brw(args_raw = args)]
    script: PPtr,
    name: AlignedString,
}
