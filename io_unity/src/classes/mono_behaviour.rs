use binrw::binrw;

use crate::{until::binrw_parser::AlignedString, SerializedFileMetadata};

use super::{behaviour::Behaviour, p_ptr::PPtr};

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
