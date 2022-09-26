use binrw::binrw;

use crate::until::binrw_parser::AlignedString;

pub trait NamedObject {
    fn get_name(&self) -> &AlignedString;
}

#[binrw]
#[derive(Debug)]
pub struct SNamedObject {
    name: AlignedString,
}
