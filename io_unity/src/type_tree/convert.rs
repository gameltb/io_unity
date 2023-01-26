use std::{borrow::Cow, collections::HashMap, fmt::Debug, io::Cursor};

use binrw::{BinRead, ReadOptions, VecArgs};

use super::{
    reader::{TypeTreeObjectBinReadArgs, TypeTreeObjectBinReadClassArgs},
    Field, FieldValue, TypeTreeObject,
};

#[derive(Debug, Clone)]
pub struct FieldCastArgs {
    pub endian: binrw::Endian,
    pub serialized_file_id: i64,
}

pub trait TryCastRef<T>: Sized {
    type Error;

    fn try_cast_as<'a>(&'a self, field_cast_args: &FieldCastArgs) -> Result<&'a T, Self::Error>;
}

pub trait TryCastRefFrom<T>: Sized {
    type Error;

    fn try_cast_as_from<'a>(value: &'a T, path: &str) -> Result<&'a Self, Self::Error>;
}

impl<T> TryCastRefFrom<TypeTreeObject> for T
where
    Field: TryCastRef<T>,
{
    type Error = ();

    fn try_cast_as_from<'a>(
        value: &'a TypeTreeObject,
        path: &str,
    ) -> Result<&'a Self, Self::Error> {
        value
            .get_field_by_path(path)
            .and_then(|f| f.try_cast_as(&value.get_field_cast_args()).ok())
            .ok_or(())
    }
}

impl TryCastRef<Vec<u8>> for Field {
    type Error = ();

    fn try_cast_as(&self, _field_cast_args: &FieldCastArgs) -> Result<&Vec<u8>, Self::Error> {
        match &self.data {
            FieldValue::Array(array_field) => {
                if let FieldValue::Data(array) = &array_field.data {
                    return Ok(&array);
                }
            }
            FieldValue::Data(data_buff) => {
                return Ok(data_buff);
            }
            _ => (),
        }
        Err(())
    }
}

pub trait TryCast<T>: Sized {
    type Error;

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<T, Self::Error>;
}

pub trait TryCastFrom<T>: Sized {
    type Error;

    fn try_cast_from(value: &T, path: &str) -> Result<Self, Self::Error>;
}

impl<T> TryCastFrom<TypeTreeObject> for T
where
    Field: TryCast<T>,
{
    type Error = ();

    fn try_cast_from(value: &TypeTreeObject, path: &str) -> Result<Self, Self::Error> {
        value
            .get_field_by_path(path)
            .and_then(|f| f.try_cast_to(&value.get_field_cast_args()).ok())
            .ok_or(())
    }
}

impl TryCast<bool> for Field {
    type Error = ();

    fn try_cast_to(&self, _field_cast_args: &FieldCastArgs) -> Result<bool, Self::Error> {
        if let FieldValue::Data(data) = &self.data {
            if ["bool"].contains(&self.field_type.get_type().as_str()) {
                if let Some(i) = data.get(0) {
                    return Ok(*i != 0);
                }
            }
        }
        Err(())
    }
}

impl TryCast<i8> for Field {
    type Error = ();

    fn try_cast_to(&self, _field_cast_args: &FieldCastArgs) -> Result<i8, Self::Error> {
        if let FieldValue::Data(data) = &self.data {
            if ["SInt8"].contains(&self.field_type.get_type().as_str()) {
                return <i8>::read(&mut Cursor::new(data)).map_err(|_| ());
            }
        }
        Err(())
    }
}

impl TryCast<i16> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<i16, Self::Error> {
        if let FieldValue::Data(data) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if ["SInt16", "short"].contains(&self.field_type.get_type().as_str()) {
                return <i16>::read_options(&mut Cursor::new(data), &op, ()).map_err(|_| ());
            }
        }
        Err(())
    }
}

impl TryCast<i32> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<i32, Self::Error> {
        if let FieldValue::Data(data) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if ["SInt32", "int"].contains(&self.field_type.get_type().as_str()) {
                return <i32>::read_options(&mut Cursor::new(data), &op, ()).map_err(|_| ());
            }
        }
        Err(())
    }
}

impl TryCast<i64> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<i64, Self::Error> {
        if ["SInt64", "long long"].contains(&self.field_type.get_type().as_str()) {
            if let FieldValue::Data(data) = &self.data {
                let op = ReadOptions::new(field_cast_args.endian.clone());
                return <i64>::read_options(&mut Cursor::new(data), &op, ()).map_err(|_| ());
            }
        } else {
            let value: Result<i32, ()> = self.try_cast_to(field_cast_args);
            if let Ok(value) = value {
                return Ok(value as i64);
            }
            let value: Result<i16, ()> = self.try_cast_to(field_cast_args);
            if let Ok(value) = value {
                return Ok(value as i64);
            }
            let value: Result<i8, ()> = self.try_cast_to(field_cast_args);
            if let Ok(value) = value {
                return Ok(value as i64);
            }
        }
        Err(())
    }
}

impl TryCast<u8> for Field {
    type Error = ();

    fn try_cast_to(&self, _field_cast_args: &FieldCastArgs) -> Result<u8, Self::Error> {
        if let FieldValue::Data(data) = &self.data {
            if ["UInt8", "char"].contains(&self.field_type.get_type().as_str()) {
                return <u8>::read(&mut Cursor::new(data)).map_err(|_| ());
            }
        }
        Err(())
    }
}

impl TryCast<u16> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<u16, Self::Error> {
        if let FieldValue::Data(data) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if ["UInt16", "unsigned short"].contains(&self.field_type.get_type().as_str()) {
                return <u16>::read_options(&mut Cursor::new(data), &op, ()).map_err(|_| ());
            }
        }
        Err(())
    }
}

impl TryCast<u32> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<u32, Self::Error> {
        if let FieldValue::Data(data) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if ["UInt32", "unsigned int"].contains(&self.field_type.get_type().as_str()) {
                return <u32>::read_options(&mut Cursor::new(data), &op, ()).map_err(|_| ());
            }
        }
        Err(())
    }
}

impl TryCast<u64> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<u64, Self::Error> {
        if ["UInt64", "unsigned long long"].contains(&self.field_type.get_type().as_str()) {
            if let FieldValue::Data(data) = &self.data {
                let op = ReadOptions::new(field_cast_args.endian.clone());
                return <u64>::read_options(&mut Cursor::new(data), &op, ()).map_err(|_| ());
            }
        } else {
            let value: Result<u32, ()> = self.try_cast_to(field_cast_args);
            if let Ok(value) = value {
                return Ok(value as u64);
            }
            let value: Result<u16, ()> = self.try_cast_to(field_cast_args);
            if let Ok(value) = value {
                return Ok(value as u64);
            }
            let value: Result<u8, ()> = self.try_cast_to(field_cast_args);
            if let Ok(value) = value {
                return Ok(value as u64);
            }
        }
        Err(())
    }
}

impl TryCast<f32> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<f32, Self::Error> {
        if let FieldValue::Data(data) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if ["float"].contains(&self.field_type.get_type().as_str()) {
                return <f32>::read_options(&mut Cursor::new(data), &op, ()).map_err(|_| ());
            }
        }
        Err(())
    }
}

impl TryCast<f64> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<f64, Self::Error> {
        if let FieldValue::Data(data) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if ["double"].contains(&self.field_type.get_type().as_str()) {
                return <f64>::read_options(&mut Cursor::new(data), &op, ()).map_err(|_| ());
            }
        }
        Err(())
    }
}

impl TryCast<TypeTreeObject> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<TypeTreeObject, Self::Error> {
        Ok(TypeTreeObject {
            endian: field_cast_args.endian.clone(),
            class_id: 0,
            serialized_file_id: field_cast_args.serialized_file_id,
            data: self.clone(),
        })
    }
}

impl TryCast<Vec<TypeTreeObject>> for Field {
    type Error = ();

    fn try_cast_to(
        &self,
        field_cast_args: &FieldCastArgs,
    ) -> Result<Vec<TypeTreeObject>, Self::Error> {
        match &self.data {
            FieldValue::Array(array_field) => match &array_field.data {
                FieldValue::Data(array_data_buff) => {
                    let size: i32 = array_field.array_size.try_cast_to(field_cast_args)?;
                    let class_args = TypeTreeObjectBinReadClassArgs::new(
                        0,
                        array_field.item_type_fields.clone(),
                    );
                    let mut obj_array = vec![];
                    let op = ReadOptions::new(field_cast_args.endian.clone());

                    let args = TypeTreeObjectBinReadArgs::new(
                        field_cast_args.serialized_file_id,
                        class_args,
                    );
                    for _ in 0..size {
                        obj_array.push(
                            TypeTreeObject::read_options(
                                &mut Cursor::new(array_data_buff),
                                &op,
                                args.clone(),
                            )
                            .map_err(|_| ())?,
                        )
                    }
                    return Ok(obj_array);
                }
                FieldValue::ArrayFields(array_fields) => {
                    return Ok(array_fields
                        .iter()
                        .map(|f| TypeTreeObject {
                            endian: field_cast_args.endian.clone(),
                            class_id: 0,
                            serialized_file_id: field_cast_args.serialized_file_id,
                            data: f.clone(),
                        })
                        .collect());
                }
                _ => (),
            },
            _ => (),
        }
        Err(())
    }
}

impl<'a> TryCast<Cow<'a, Vec<u8>>> for &'a Field {
    type Error = ();

    fn try_cast_to(
        &self,
        _field_cast_args: &FieldCastArgs,
    ) -> Result<Cow<'a, Vec<u8>>, Self::Error> {
        if let FieldValue::Array(array_field) = &self.data {
            if let FieldValue::Data(array) = &array_field.data {
                return Ok(Cow::Borrowed(array));
            }
        }
        Err(())
    }
}

impl TryCast<Vec<f32>> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<Vec<f32>, Self::Error> {
        if let FieldValue::Array(array_field) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if let FieldValue::Data(array) = &array_field.data {
                let size: i32 = array_field.array_size.try_cast_to(field_cast_args)?;
                if array_field.item_type_fields.len() == 1 {
                    let item_type_field = array_field.item_type_fields.get(0).unwrap();
                    if item_type_field.get_type().as_str() == "float" {
                        return <Vec<f32>>::read_options(
                            &mut Cursor::new(array),
                            &op,
                            VecArgs {
                                count: size as usize,
                                inner: (),
                            },
                        )
                        .map_err(|_| ());
                    }
                }
            }
        }
        Err(())
    }
}

impl TryCast<Vec<u16>> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<Vec<u16>, Self::Error> {
        if let FieldValue::Array(array_field) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if let FieldValue::Data(array) = &array_field.data {
                let size: i32 = array_field.array_size.try_cast_to(field_cast_args)?;
                if array_field.item_type_fields.len() == 1 {
                    let item_type_field = array_field.item_type_fields.get(0).unwrap();
                    if item_type_field.get_byte_size() == 2 {
                        return <Vec<u16>>::read_options(
                            &mut Cursor::new(array),
                            &op,
                            VecArgs {
                                count: size as usize,
                                inner: (),
                            },
                        )
                        .map_err(|_| ());
                    }
                }
            }
        }
        Err(())
    }
}

impl TryCast<Vec<u32>> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<Vec<u32>, Self::Error> {
        if let FieldValue::Array(array_field) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if let FieldValue::Data(array) = &array_field.data {
                let size: i32 = array_field.array_size.try_cast_to(field_cast_args)?;
                if array_field.item_type_fields.len() == 1 {
                    let item_type_field = array_field.item_type_fields.get(0).unwrap();
                    if item_type_field.get_byte_size() == 4 {
                        return <Vec<u32>>::read_options(
                            &mut Cursor::new(array),
                            &op,
                            VecArgs {
                                count: size as usize,
                                inner: (),
                            },
                        )
                        .map_err(|_| ());
                    }
                }
            }
        }
        Err(())
    }
}

impl TryCast<Vec<u64>> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<Vec<u64>, Self::Error> {
        if let FieldValue::Array(array_field) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if let FieldValue::Data(array) = &array_field.data {
                let size: i32 = array_field.array_size.try_cast_to(field_cast_args)?;
                if array_field.item_type_fields.len() == 1 {
                    let item_type_field = array_field.item_type_fields.get(0).unwrap();
                    if item_type_field.get_byte_size() == 8 {
                        return <Vec<u64>>::read_options(
                            &mut Cursor::new(array),
                            &op,
                            VecArgs {
                                count: size as usize,
                                inner: (),
                            },
                        )
                        .map_err(|_| ());
                    }
                }
            }
        }
        Err(())
    }
}

impl TryCast<String> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<String, Self::Error> {
        if let FieldValue::Fields(fields) = &self.data {
            if "string" == self.field_type.get_type() {
                if let Some(array) = fields.values().next() {
                    let data: Cow<Vec<u8>> = (&array).try_cast_to(field_cast_args)?;
                    if let Ok(string) = String::from_utf8(data.to_vec()) {
                        return Ok(string);
                    }
                }
            }
        }
        Err(())
    }
}

impl TryCast<HashMap<String, TypeTreeObject>> for Field {
    type Error = ();

    fn try_cast_to(
        &self,
        field_cast_args: &FieldCastArgs,
    ) -> Result<HashMap<String, TypeTreeObject>, Self::Error> {
        let entites: Vec<TypeTreeObject> = self.try_cast_to(field_cast_args)?;
        let mut map = HashMap::new();
        for entry in entites {
            let key = String::try_cast_from(&entry, "/Base/first");
            let value = TypeTreeObject::try_cast_from(&entry, "/Base/second");
            if let Ok(key) = key {
                if let Ok(value) = value {
                    map.insert(key, value);
                }
            }
        }
        Ok(map)
    }
}

impl TryCast<glam::Quat> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<glam::Quat, Self::Error> {
        if let FieldValue::Fields(fields) = &self.data {
            let x: f32 = fields.get("x").ok_or(())?.try_cast_to(field_cast_args)?;
            let y: f32 = fields.get("y").ok_or(())?.try_cast_to(field_cast_args)?;
            let z: f32 = fields.get("z").ok_or(())?.try_cast_to(field_cast_args)?;
            let w: f32 = fields.get("w").ok_or(())?.try_cast_to(field_cast_args)?;
            return Ok(glam::Quat::from_xyzw(x, y, z, w));
        }
        Err(())
    }
}

impl TryCast<glam::Vec3> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<glam::Vec3, Self::Error> {
        if let FieldValue::Fields(fields) = &self.data {
            let x: f32 = fields.get("x").ok_or(())?.try_cast_to(field_cast_args)?;
            let y: f32 = fields.get("y").ok_or(())?.try_cast_to(field_cast_args)?;
            let z: f32 = fields.get("z").ok_or(())?.try_cast_to(field_cast_args)?;
            return Ok(glam::Vec3::new(x, y, z));
        }
        Err(())
    }
}
