use std::fmt;
use std::io::prelude::*;
use std::ops::Deref;

use binrw::{binrw, BinResult, Endian};
use binrw::{BinRead, BinWrite};

// reading/writing utilities

#[allow(unused_variables)]
#[binrw::parser(reader, endian)]
pub fn position_parser() -> BinResult<u64> {
    Ok(reader.stream_position()?)
}

#[derive(Debug, PartialEq, Clone)]
pub struct U8Bool(bool);

impl Deref for U8Bool {
    type Target = bool;

    fn deref(&self) -> &bool {
        &self.0
    }
}

impl BinRead for U8Bool {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let val = <u8>::read_options(reader, endian, ())?;
        Ok(U8Bool(val != 0))
    }
}

impl BinWrite for U8Bool {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {
        let buf = if **self { [1u8; 1] } else { [0u8; 1] };
        writer.write_all(&buf)?;

        Ok(())
    }
}

#[binrw]
#[derive(Eq, PartialEq)]
pub struct AlignedString {
    string_length: i32,
    #[br(count(string_length), align_after(4))]
    string: Vec<u8>,
}

impl fmt::Display for AlignedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from_utf8(self.string.clone()).unwrap())
    }
}

impl fmt::Debug for AlignedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[binrw]
#[derive(Debug)]
pub struct PackedFloatVector {
    num_items: u32,
    range: f32,
    start: f32,
    num_data: i32,
    #[br(count(num_data), align_after(4))]
    data: Vec<u8>,
    #[br(align_after(4))]
    bit_size: u8,
}

#[binrw]
#[derive(Debug)]
pub struct PackedIntVector {
    num_items: u32,
    num_data: i32,
    #[br(count(num_data), align_after(4))]
    data: Vec<u8>,
    #[br(align_after(4))]
    bit_size: u8,
}

#[binrw]
#[derive(Debug)]
pub struct PackedQuatVector {
    num_items: u32,
    num_data: i32,
    #[br(count(num_data), align_after(4))]
    data: Vec<u8>,
}
