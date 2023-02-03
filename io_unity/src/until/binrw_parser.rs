use std::fmt;

use std::io::prelude::*;
use std::ops::Deref;

use binrw::{binrw, BinResult, ReadOptions, WriteOptions};
use binrw::{BinRead, BinWrite};

// reading/writing utilities

pub fn position_parser<R: Read + Seek>(reader: &mut R, _ro: &ReadOptions, _: ()) -> BinResult<u64> {
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
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        let val = <u8>::read_options(reader, options, ())?;
        Ok(U8Bool(val != 0))
    }
}

impl BinWrite for U8Bool {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _options: &WriteOptions,
        _args: Self::Args,
    ) -> BinResult<()> {
        let buf = if **self { [1u8; 1] } else { [0u8; 1] };
        writer.write_all(&buf)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Vec2(glam::Vec2);

impl Deref for Vec2 {
    type Target = glam::Vec2;

    fn deref(&self) -> &glam::Vec2 {
        &self.0
    }
}

impl BinRead for Vec2 {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        Ok(Vec2(glam::Vec2::from_array(<[f32; 2]>::read_options(
            reader,
            options,
            (),
        )?)))
    }
}

impl BinWrite for Vec2 {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: Self::Args,
    ) -> BinResult<()> {
        self.0.to_array().write_options(writer, options, args)?;
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Vec3(glam::Vec3);

impl Deref for Vec3 {
    type Target = glam::Vec3;

    fn deref(&self) -> &glam::Vec3 {
        &self.0
    }
}

impl BinRead for Vec3 {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        Ok(Vec3(glam::Vec3::from_array(<[f32; 3]>::read_options(
            reader,
            options,
            (),
        )?)))
    }
}

impl BinWrite for Vec3 {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: Self::Args,
    ) -> BinResult<()> {
        self.0.to_array().write_options(writer, options, args)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Vec4(glam::Vec4);

impl Deref for Vec4 {
    type Target = glam::Vec4;

    fn deref(&self) -> &glam::Vec4 {
        &self.0
    }
}

impl BinRead for Vec4 {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        Ok(Vec4(glam::Vec4::from(<[f32; 4]>::read_options(
            reader,
            options,
            (),
        )?)))
    }
}

impl BinWrite for Vec4 {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: Self::Args,
    ) -> BinResult<()> {
        self.0.to_array().write_options(writer, options, args)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Mat4(pub glam::Mat4);

impl Deref for Mat4 {
    type Target = glam::Mat4;

    fn deref(&self) -> &glam::Mat4 {
        &self.0
    }
}

impl BinRead for Mat4 {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        Ok(Mat4(glam::Mat4::from_cols_array(
            &<[f32; 16]>::read_options(reader, options, ())?,
        )))
    }
}

impl BinWrite for Mat4 {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: Self::Args,
    ) -> BinResult<()> {
        self.0
            .to_cols_array()
            .write_options(writer, options, args)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Quat(glam::Quat);

impl Deref for Quat {
    type Target = glam::Quat;

    fn deref(&self) -> &glam::Quat {
        &self.0
    }
}

impl BinRead for Quat {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        Ok(Quat(glam::Quat::from_array(<[f32; 4]>::read_options(
            reader,
            options,
            (),
        )?)))
    }
}

impl BinWrite for Quat {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: Self::Args,
    ) -> BinResult<()> {
        self.0.to_array().write_options(writer, options, args)?;
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
pub struct AABB {
    center: Vec3,
    extent: Vec3,
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
