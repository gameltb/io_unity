use std::{
    collections::HashMap,
    fmt::Debug,
    io::{Cursor, Read, Seek},
};

use binrw::{BinRead, ReadOptions, VecArgs};

use super::{ArrayFieldValue, DataOffset, Field, FieldValue, TypeTreeObject, TypeTreeObjectRef};

#[derive(Debug, Clone)]
pub struct FieldCastArgs {
    pub endian: binrw::Endian,
    pub field_offset: Option<i64>,
}

pub trait TryRead<T>: Sized {
    type Error;

    fn try_read_to<R: Read + Seek>(
        &self,
        object_data_reader: &mut R,
        field_cast_args: &FieldCastArgs,
    ) -> Result<T, Self::Error>;
}

impl TryRead<i32> for Field {
    type Error = anyhow::Error;

    fn try_read_to<R: Read + Seek>(
        &self,
        object_data_reader: &mut R,
        field_cast_args: &FieldCastArgs,
    ) -> Result<i32, Self::Error> {
        if ["SInt32", "int"].contains(&self.field_type.get_type().as_str()) {
            let op = ReadOptions::new(field_cast_args.endian);
            return Ok(<i32>::read_options(object_data_reader, &op, ())?);
        }
        Err(anyhow!("Type not match."))
    }
}

pub trait TryCast<T>: Sized {
    type Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<T, Self::Error>;
}

pub trait TryCastFrom<T, P>: Sized {
    type Error;

    fn try_cast_from(value: T, path: P) -> Result<Self, Self::Error>;
}

impl<T, E> TryCastFrom<E, &str> for T
where
    for<'a> T: TryCastFrom<E, &'a [String], Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    fn try_cast_from(value: E, path: &str) -> Result<Self, Self::Error> {
        let path: Vec<String> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        <T>::try_cast_from(value, &path[1..])
    }
}

impl<T> TryCastFrom<&TypeTreeObject, &[String]> for T
where
    Field: TryCast<T, Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    fn try_cast_from(value: &TypeTreeObject, path: &[String]) -> Result<Self, Self::Error> {
        value
            .get_field_by_path_list(path)
            .map(|(feild, offset)| {
                let mut field_cast_args = value.get_field_cast_args();
                field_cast_args.field_offset = offset;
                feild.try_cast_to(&value.data_buff, &field_cast_args)
            })
            .ok_or(anyhow!(format!("cannot get field {path:?}")))?
    }
}

impl<'a, T> TryCastFrom<&TypeTreeObjectRef, &'a [String]> for T
where
    Field: TryCast<T, Error = anyhow::Error>,
{
    type Error = anyhow::Error;

    fn try_cast_from(value: &TypeTreeObjectRef, path: &'a [String]) -> Result<Self, Self::Error> {
        let mut self_path: Vec<String> = value.path.clone();
        self_path.extend_from_slice(path);
        let type_tree_obj = value.inner.read().map_err(|e| anyhow!(e.to_string()))?;
        type_tree_obj
            .get_field_by_path_list(&self_path)
            .map(|(feild, offset)| {
                let mut field_cast_args = type_tree_obj.get_field_cast_args();
                field_cast_args.field_offset = offset;
                feild.try_cast_to(&type_tree_obj.data_buff, &field_cast_args)
            })
            .ok_or(anyhow!(format!("cannot get field {self_path:?}")))?
    }
}

impl TryCastFrom<&TypeTreeObjectRef, &[String]> for TypeTreeObjectRef {
    type Error = anyhow::Error;

    fn try_cast_from(value: &TypeTreeObjectRef, path: &[String]) -> Result<Self, Self::Error> {
        let mut self_path: Vec<String> = value.path.clone();
        self_path.extend_from_slice(path);
        let type_tree_obj = value.inner.read().map_err(|e| anyhow!(e.to_string()))?;
        if type_tree_obj.get_field_by_path_list(&self_path).is_some() {
            return Ok(TypeTreeObjectRef {
                inner: value.inner.clone(),
                path: self_path,
            });
        }
        Err(anyhow!("cannot get TypeTreeObjectRef field {self_path:?}"))
    }
}

impl TryCastFrom<&TypeTreeObjectRef, &[String]> for Vec<TypeTreeObjectRef> {
    type Error = anyhow::Error;

    fn try_cast_from(value: &TypeTreeObjectRef, path: &[String]) -> Result<Self, Self::Error> {
        let mut self_path: Vec<String> = value.path.clone();
        self_path.extend_from_slice(path);
        let type_tree_obj = value.inner.read().map_err(|e| anyhow!(e.to_string()))?;
        if let Some((array_field, _offset)) = type_tree_obj.get_field_by_path_list(&self_path) {
            if let FieldValue::Array(array) = &array_field.data {
                let size: i32 = array.array_size.try_cast_to(
                    &type_tree_obj.data_buff,
                    &type_tree_obj.get_field_cast_args(),
                )?;
                let mut vec = Vec::new();
                for i in 0..size {
                    let mut self_path = self_path.clone();
                    self_path.push(i.to_string());
                    vec.push(TypeTreeObjectRef {
                        inner: value.inner.clone(),
                        path: self_path,
                    });
                }
                return Ok(vec);
            }
        }
        Err(anyhow!(
            "cannot get  Vec<TypeTreeObjectRef> field {self_path:?}"
        ))
    }
}

impl TryCastFrom<&TypeTreeObjectRef, &[String]> for HashMap<String, TypeTreeObjectRef> {
    type Error = anyhow::Error;

    fn try_cast_from(value: &TypeTreeObjectRef, path: &[String]) -> Result<Self, Self::Error> {
        let entites = <Vec<TypeTreeObjectRef>>::try_cast_from(value, path)?;
        let mut map = HashMap::new();
        for entry in entites {
            let key = String::try_cast_from(&entry, "/Base/first");
            let value = TypeTreeObjectRef::try_cast_from(&entry, "/Base/second");
            if let Ok(key) = key {
                if let Ok(value) = value {
                    map.insert(key, value);
                }
            }
        }
        Ok(map)
    }
}

#[inline]
fn gen_reader<'a>(
    object_data_buff: &'a [u8],
    data_offset: &DataOffset,
    field_cast_args: &FieldCastArgs,
) -> anyhow::Result<Cursor<&'a [u8]>> {
    let mut reader = Cursor::new(object_data_buff);
    match data_offset {
        DataOffset::AbsDataOffset(data) => reader.set_position(*data),
        DataOffset::ArrayItemOffset(data) => reader.set_position(
            *data
                + field_cast_args
                    .field_offset
                    .ok_or(anyhow!("ArrayItemOffset use without field offset."))?
                    as u64,
        ),
    }
    Ok(reader)
}

impl TryCast<bool> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<bool, Self::Error> {
        if let FieldValue::DataOffset(data) = &self.data {
            if ["bool"].contains(&self.field_type.get_type().as_str()) {
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                return Ok(<u8>::read(&mut reader)? != 0);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<i8> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<i8, Self::Error> {
        if let FieldValue::DataOffset(data) = &self.data {
            if ["SInt8"].contains(&self.field_type.get_type().as_str()) {
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                return Ok(<i8>::read(&mut reader)?);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<i16> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<i16, Self::Error> {
        if let FieldValue::DataOffset(data) = &self.data {
            if ["SInt16", "short"].contains(&self.field_type.get_type().as_str()) {
                let op = ReadOptions::new(field_cast_args.endian);
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                return Ok(<i16>::read_options(&mut reader, &op, ())?);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<i32> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<i32, Self::Error> {
        if let FieldValue::DataOffset(data) = &self.data {
            if ["SInt32", "int"].contains(&self.field_type.get_type().as_str()) {
                let op = ReadOptions::new(field_cast_args.endian);
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                return Ok(<i32>::read_options(&mut reader, &op, ())?);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<i64> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<i64, Self::Error> {
        if ["SInt64", "long long"].contains(&self.field_type.get_type().as_str()) {
            if let FieldValue::DataOffset(data) = &self.data {
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                let op = ReadOptions::new(field_cast_args.endian);
                return Ok(<i64>::read_options(&mut reader, &op, ())?);
            }
        } else {
            let value: Result<i32, _> = self.try_cast_to(object_data_buff, field_cast_args);
            if let Ok(value) = value {
                return Ok(value as i64);
            }
            let value: Result<i16, _> = self.try_cast_to(object_data_buff, field_cast_args);
            if let Ok(value) = value {
                return Ok(value as i64);
            }
            let value: Result<i8, _> = self.try_cast_to(object_data_buff, field_cast_args);
            if let Ok(value) = value {
                return Ok(value as i64);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<u8> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<u8, Self::Error> {
        if let FieldValue::DataOffset(data) = &self.data {
            if ["UInt8", "char"].contains(&self.field_type.get_type().as_str()) {
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                return Ok(<u8>::read(&mut reader)?);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<u16> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<u16, Self::Error> {
        if let FieldValue::DataOffset(data) = &self.data {
            if ["UInt16", "unsigned short"].contains(&self.field_type.get_type().as_str()) {
                let op = ReadOptions::new(field_cast_args.endian);
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                return Ok(<u16>::read_options(&mut reader, &op, ())?);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<u32> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<u32, Self::Error> {
        if let FieldValue::DataOffset(data) = &self.data {
            if ["UInt32", "unsigned int"].contains(&self.field_type.get_type().as_str()) {
                let op = ReadOptions::new(field_cast_args.endian);
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                return Ok(<u32>::read_options(&mut reader, &op, ())?);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<u64> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<u64, Self::Error> {
        if ["UInt64", "unsigned long long"].contains(&self.field_type.get_type().as_str()) {
            if let FieldValue::DataOffset(data) = &self.data {
                let op = ReadOptions::new(field_cast_args.endian);
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                return Ok(<u64>::read_options(&mut reader, &op, ())?);
            }
        } else {
            let value: Result<u32, _> = self.try_cast_to(object_data_buff, field_cast_args);
            if let Ok(value) = value {
                return Ok(value as u64);
            }
            let value: Result<u16, _> = self.try_cast_to(object_data_buff, field_cast_args);
            if let Ok(value) = value {
                return Ok(value as u64);
            }
            let value: Result<u8, _> = self.try_cast_to(object_data_buff, field_cast_args);
            if let Ok(value) = value {
                return Ok(value as u64);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<usize> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<usize, Self::Error> {
        if ["FileSize"].contains(&self.field_type.get_type().as_str()) {
            if let FieldValue::DataOffset(data) = &self.data {
                let op = ReadOptions::new(field_cast_args.endian);
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                return Ok(<u64>::read_options(&mut reader, &op, ())? as usize);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<f32> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<f32, Self::Error> {
        if let FieldValue::DataOffset(data) = &self.data {
            if ["float"].contains(&self.field_type.get_type().as_str()) {
                let op = ReadOptions::new(field_cast_args.endian);
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                return Ok(<f32>::read_options(&mut reader, &op, ())?);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<f64> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<f64, Self::Error> {
        if let FieldValue::DataOffset(data) = &self.data {
            if ["double"].contains(&self.field_type.get_type().as_str()) {
                let op = ReadOptions::new(field_cast_args.endian);
                let mut reader = gen_reader(object_data_buff, data, field_cast_args)?;
                return Ok(<f64>::read_options(&mut reader, &op, ())?);
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<Vec<f32>> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<Vec<f32>, Self::Error> {
        if let FieldValue::Array(array_field) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian);
            if let ArrayFieldValue::DataOffset(array) = &array_field.data {
                let size: i32 = array_field
                    .array_size
                    .try_cast_to(object_data_buff, field_cast_args)?;
                if let Some(item_field) = &array_field.item_field {
                    if item_field.field_type.get_type().as_str() == "float" {
                        let mut reader = gen_reader(object_data_buff, array, field_cast_args)?;
                        return Ok(<Vec<f32>>::read_options(
                            &mut reader,
                            &op,
                            VecArgs {
                                count: size as usize,
                                inner: (),
                            },
                        )?);
                    }
                }
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<Vec<f64>> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<Vec<f64>, Self::Error> {
        if let FieldValue::Array(array_field) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian);
            if let ArrayFieldValue::DataOffset(array) = &array_field.data {
                let size: i32 = array_field
                    .array_size
                    .try_cast_to(object_data_buff, field_cast_args)?;
                if let Some(item_field) = &array_field.item_field {
                    if item_field.field_type.get_type().as_str() == "double" {
                        let mut reader = gen_reader(object_data_buff, array, field_cast_args)?;
                        return Ok(<Vec<f64>>::read_options(
                            &mut reader,
                            &op,
                            VecArgs {
                                count: size as usize,
                                inner: (),
                            },
                        )?);
                    }
                }
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<Vec<u8>> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<Vec<u8>, Self::Error> {
        if let FieldValue::Array(array_field) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian);
            if let ArrayFieldValue::DataOffset(array) = &array_field.data {
                let size: i32 = array_field
                    .array_size
                    .try_cast_to(object_data_buff, field_cast_args)?;
                if let Some(item_field) = &array_field.item_field {
                    if let FieldValue::DataOffset(_) = item_field.data {
                        if item_field.field_type.get_byte_size() == 1 {
                            let mut reader = gen_reader(object_data_buff, array, field_cast_args)?;
                            return Ok(<Vec<u8>>::read_options(
                                &mut reader,
                                &op,
                                VecArgs {
                                    count: size as usize,
                                    inner: (),
                                },
                            )?);
                        }
                    }
                }
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<Vec<u16>> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<Vec<u16>, Self::Error> {
        if let FieldValue::Array(array_field) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian);
            if let ArrayFieldValue::DataOffset(array) = &array_field.data {
                let size: i32 = array_field
                    .array_size
                    .try_cast_to(object_data_buff, field_cast_args)?;
                if let Some(item_field) = &array_field.item_field {
                    if let FieldValue::DataOffset(_) = item_field.data {
                        if item_field.field_type.get_byte_size() == 2 {
                            let mut reader = gen_reader(object_data_buff, array, field_cast_args)?;
                            return Ok(<Vec<u16>>::read_options(
                                &mut reader,
                                &op,
                                VecArgs {
                                    count: size as usize,
                                    inner: (),
                                },
                            )?);
                        }
                    }
                }
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<Vec<u32>> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<Vec<u32>, Self::Error> {
        if let FieldValue::Array(array_field) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian);
            if let ArrayFieldValue::DataOffset(array) = &array_field.data {
                let size: i32 = array_field
                    .array_size
                    .try_cast_to(object_data_buff, field_cast_args)?;
                if let Some(item_field) = &array_field.item_field {
                    if let FieldValue::DataOffset(_) = item_field.data {
                        if item_field.field_type.get_byte_size() == 4 {
                            let mut reader = gen_reader(object_data_buff, array, field_cast_args)?;
                            return Ok(<Vec<u32>>::read_options(
                                &mut reader,
                                &op,
                                VecArgs {
                                    count: size as usize,
                                    inner: (),
                                },
                            )?);
                        }
                    }
                }
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<Vec<u64>> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<Vec<u64>, Self::Error> {
        if let FieldValue::Array(array_field) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian);
            if let ArrayFieldValue::DataOffset(array) = &array_field.data {
                let size: i32 = array_field
                    .array_size
                    .try_cast_to(object_data_buff, field_cast_args)?;
                if let Some(item_field) = &array_field.item_field {
                    if let FieldValue::DataOffset(_) = item_field.data {
                        if item_field.field_type.get_byte_size() == 8 {
                            let mut reader = gen_reader(object_data_buff, array, field_cast_args)?;
                            return Ok(<Vec<u64>>::read_options(
                                &mut reader,
                                &op,
                                VecArgs {
                                    count: size as usize,
                                    inner: (),
                                },
                            )?);
                        }
                    }
                }
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<String> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<String, Self::Error> {
        if let FieldValue::Fields(fields) = &self.data {
            if "string" == self.field_type.get_type() {
                if let Some(array) = fields.get("Array") {
                    let data = array.try_as_slice(object_data_buff, field_cast_args)?;
                    return Ok(String::from_utf8_lossy(data).to_string());
                }
            }
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<glam::Quat> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<glam::Quat, Self::Error> {
        if let FieldValue::Fields(fields) = &self.data {
            let x: f32 = fields
                .get("x")
                .ok_or(anyhow!("Cannot get x."))?
                .try_cast_to(object_data_buff, field_cast_args)?;
            let y: f32 = fields
                .get("y")
                .ok_or(anyhow!("Cannot get y."))?
                .try_cast_to(object_data_buff, field_cast_args)?;
            let z: f32 = fields
                .get("z")
                .ok_or(anyhow!("Cannot get z."))?
                .try_cast_to(object_data_buff, field_cast_args)?;
            let w: f32 = fields
                .get("w")
                .ok_or(anyhow!("Cannot get w."))?
                .try_cast_to(object_data_buff, field_cast_args)?;
            return Ok(glam::Quat::from_xyzw(x, y, z, w));
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<glam::Vec3> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<glam::Vec3, Self::Error> {
        if let FieldValue::Fields(fields) = &self.data {
            let x: f32 = fields
                .get("x")
                .ok_or(anyhow!("Cannot get x."))?
                .try_cast_to(object_data_buff, field_cast_args)?;
            let y: f32 = fields
                .get("y")
                .ok_or(anyhow!("Cannot get y."))?
                .try_cast_to(object_data_buff, field_cast_args)?;
            let z: f32 = fields
                .get("z")
                .ok_or(anyhow!("Cannot get z."))?
                .try_cast_to(object_data_buff, field_cast_args)?;
            return Ok(glam::Vec3::new(x, y, z));
        }
        Err(anyhow!("Type not match."))
    }
}

impl TryCast<glam::Vec2> for Field {
    type Error = anyhow::Error;

    fn try_cast_to(
        &self,
        object_data_buff: &[u8],
        field_cast_args: &FieldCastArgs,
    ) -> Result<glam::Vec2, Self::Error> {
        if let FieldValue::Fields(fields) = &self.data {
            let x: f32 = fields
                .get("x")
                .ok_or(anyhow!("Cannot get x."))?
                .try_cast_to(object_data_buff, field_cast_args)?;
            let y: f32 = fields
                .get("y")
                .ok_or(anyhow!("Cannot get y."))?
                .try_cast_to(object_data_buff, field_cast_args)?;
            return Ok(glam::Vec2::new(x, y));
        }
        Err(anyhow!("Type not match."))
    }
}
