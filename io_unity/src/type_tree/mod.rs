pub mod type_tree_json;

use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Debug,
    io::{Cursor, ErrorKind, Read, Seek, SeekFrom},
    sync::Arc,
    time::Duration,
};

use binrw::{BinRead, BinResult, ReadOptions, VecArgs};

pub trait TypeField: Debug {
    fn get_version(&self) -> u16;
    fn get_level(&self) -> u8;
    fn is_array(&self) -> bool;
    fn get_byte_size(&self) -> i32;
    fn get_index(&self) -> i32;
    fn get_meta_flag(&self) -> i32;
    fn is_align(&self) -> bool;
    fn get_ref_type_hash(&self) -> Option<u64>;
    fn get_type(&self) -> &String;
    fn get_name(&self) -> &String;
}

#[derive(Debug, Clone)]
pub enum FieldValue {
    Data(Vec<u8>),
    Fields(HashMap<String, Field>),
    ArrayFields(Vec<Field>),
    Array(Box<ArrayField>),
}

#[derive(Debug, Clone)]
pub struct ArrayField {
    array_size: Field,
    item_type_fields: Vec<Arc<Box<dyn TypeField + Send + Sync>>>,
    data: FieldValue,
}

#[derive(Debug, Clone)]
pub struct Field {
    field_type: Arc<Box<dyn TypeField + Send + Sync>>,
    data: FieldValue,
    time: Duration,
}

#[derive(Debug, Clone)]
pub struct FieldCastArgs {
    endian: binrw::Endian,
    serialized_file_id: i64,
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

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<bool, Self::Error> {
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

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<i8, Self::Error> {
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
        if let FieldValue::Data(data) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if ["SInt64", "long long"].contains(&self.field_type.get_type().as_str()) {
                return <i64>::read_options(&mut Cursor::new(data), &op, ()).map_err(|_| ());
            }
        }
        Err(())
    }
}

impl TryCast<u8> for Field {
    type Error = ();

    fn try_cast_to(&self, field_cast_args: &FieldCastArgs) -> Result<u8, Self::Error> {
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
        if let FieldValue::Data(data) = &self.data {
            let op = ReadOptions::new(field_cast_args.endian.clone());
            if ["UInt64", "unsigned long long"].contains(&self.field_type.get_type().as_str()) {
                return <u64>::read_options(&mut Cursor::new(data), &op, ()).map_err(|_| ());
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
        field_cast_args: &FieldCastArgs,
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

impl Field {
    fn get_name(&self) -> &String {
        self.field_type.get_name()
    }

    fn display_field(&self, p: &String) {
        let np = p.clone() + "/" + self.field_type.get_name();
        print!(
            "{}/{} : {}({}) {:?}",
            p,
            self.field_type.get_name(),
            self.field_type.get_type(),
            self.field_type.get_byte_size(),
            &self.time
        );
        match &self.data {
            FieldValue::Data(v) => {
                if self.field_type.get_type() == "SInt64" {
                    println!(
                        " data : {:?}",
                        i64::from_le_bytes(v.as_slice().try_into().unwrap())
                    );
                } else if v.len() <= 8 {
                    println!(" data : {:?}", v);
                } else {
                    println!(" data : {:?}...", &v[..8]);
                }
            }
            FieldValue::Fields(fls) => {
                println!("");
                fls.into_iter().map(|(n, f)| f.display_field(&np)).collect()
            }
            FieldValue::Array(ar) => {
                println!("");
                ar.array_size.display_field(&np);
                match &ar.data {
                    FieldValue::ArrayFields(ai) => {
                        if let Some(aii) = ai.get(0) {
                            aii.display_field(&np);
                        }
                    }
                    FieldValue::Data(_) => {
                        for item in &ar.item_type_fields {
                            println!(
                                "{}/?/{} : {}({}) at level [{}]",
                                np,
                                item.get_name(),
                                item.get_type(),
                                item.get_byte_size(),
                                item.get_level(),
                            );
                        }
                    }
                    _ => (),
                }
            }
            FieldValue::ArrayFields(_) => todo!(),
        }
    }

    fn get_field(&self, path: &[String]) -> Option<&Self> {
        if path.len() == 0 {
            return Some(self);
        } else {
            match &self.data {
                FieldValue::Fields(fields) => {
                    if let Some((name, path)) = path.split_first() {
                        if let Some(field) = fields.get(name) {
                            return field.get_field(path);
                        }
                    }
                }
                FieldValue::Array(_array) => (),
                _ => (),
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct TypeTreeObject {
    endian: binrw::Endian,
    pub class_id: i32,
    pub serialized_file_id: i64,
    data: Field,
}

impl TypeTreeObject {
    pub fn display_tree(&self) {
        println!("class_id : {}", self.class_id);
        self.data.display_field(&"".to_string());
    }

    pub fn get_endian(&self) -> binrw::Endian {
        self.endian.clone()
    }

    pub fn get_field_by_path(&self, path: &str) -> Option<&Field> {
        let path: Vec<String> = path
            .split("/")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        if path.len() < 1 {
            return None;
        }
        self.data.get_field(&path[1..])
    }

    pub fn get_field_cast_args(&self) -> FieldCastArgs {
        FieldCastArgs {
            endian: self.endian.clone(),
            serialized_file_id: self.serialized_file_id,
        }
    }

    pub fn get_string_by_path(&self, path: &str) -> Option<String> {
        String::try_cast_from(self, path).ok()
    }

    pub fn get_byte_array_by_path(&self, path: &str) -> Option<Cow<Vec<u8>>> {
        // <Cow<Vec<u8>>>::try_cast_from(self, path).ok()
        todo!()
    }

    pub fn get_array_object_by_path(&self, path: &str) -> Option<Vec<TypeTreeObject>> {
        <Vec<TypeTreeObject>>::try_cast_from(self, path).ok()
    }

    pub fn get_array_float_by_path(&self, path: &str) -> Option<Vec<f32>> {
        <Vec<f32>>::try_cast_from(self, path).ok()
    }

    pub fn get_array_uint16_by_path(&self, path: &str) -> Option<Vec<u16>> {
        <Vec<u16>>::try_cast_from(self, path).ok()
    }

    pub fn get_array_uint32_by_path(&self, path: &str) -> Option<Vec<u32>> {
        <Vec<u32>>::try_cast_from(self, path).ok()
    }

    pub fn get_array_uint64_by_path(&self, path: &str) -> Option<Vec<u64>> {
        <Vec<u64>>::try_cast_from(self, path).ok()
    }

    pub fn get_string_key_map_by_path(
        &self,
        path: &str,
    ) -> Option<HashMap<String, TypeTreeObject>> {
        if let Some(map_vec) = self.get_array_object_by_path(path) {
            let mut map = HashMap::new();
            for entry in map_vec {
                let key = entry.get_string_by_path("/Base/first");
                let value = entry.get_object_by_path("/Base/second");
                if let Some(key) = key {
                    if let Some(value) = value {
                        map.insert(key, value);
                    }
                }
            }
            return Some(map);
        }
        None
    }

    pub fn get_bool_by_path(&self, path: &str) -> Option<bool> {
        <bool>::try_cast_from(self, path).ok()
    }

    pub fn get_float_by_path(&self, path: &str) -> Option<f32> {
        <f32>::try_cast_from(self, path).ok()
    }

    pub fn get_double_by_path(&self, path: &str) -> Option<f64> {
        <f64>::try_cast_from(self, path).ok()
    }

    pub fn get_int_by_path(&self, path: &str) -> Option<i64> {
        // if let Some(v) = self.get_value_by_path(path) {
        //     return match v {
        //         Value::Int8(i) => Some(i as i64),
        //         Value::Int16(i) => Some(i as i64),
        //         Value::Int32(i) => Some(i as i64),
        //         Value::Int64(i) => Some(i as i64),
        //         _ => None,
        //     };
        // }
        todo!()
    }

    pub fn get_uint_by_path(&self, path: &str) -> Option<u64> {
        // if let Some(v) = self.get_value_by_path(path) {
        //     return match v {
        //         Value::UInt8(i) => Some(i as u64),
        //         Value::UInt16(i) => Some(i as u64),
        //         Value::UInt32(i) => Some(i as u64),
        //         Value::UInt64(i) => Some(i as u64),
        //         _ => None,
        //     };
        // }
        todo!()
    }

    pub fn get_object_by_path(&self, path: &str) -> Option<TypeTreeObject> {
        <TypeTreeObject>::try_cast_from(self, path).ok()
    }

    pub fn get_quat_by_path(&self, path: &str) -> Option<glam::Quat> {
        if let Some(inner) = self.get_object_by_path(path) {
            return Some([0f32; 4])
                .and_then(|_a| {
                    inner
                        .get_float_by_path("/Base/x")
                        .and_then(|x| Some([x, 0.0, 0.0, 0.0]))
                })
                .and_then(|a| {
                    inner
                        .get_float_by_path("/Base/y")
                        .and_then(|y| Some([a[0], y, 0.0, 0.0]))
                })
                .and_then(|a| {
                    inner
                        .get_float_by_path("/Base/z")
                        .and_then(|z| Some([a[0], a[1], z, 0.0]))
                })
                .and_then(|a| {
                    inner
                        .get_float_by_path("/Base/w")
                        .and_then(|w| Some([a[0], a[1], a[2], w]))
                })
                .and_then(|a| Some(glam::Quat::from_array(a)));
        }
        None
    }

    pub fn get_vec3f_by_path(&self, path: &str) -> Option<glam::Vec3> {
        if let Some(inner) = self.get_object_by_path(path) {
            return Some([0f32; 3])
                .and_then(|_a| {
                    inner
                        .get_float_by_path("/Base/x")
                        .and_then(|x| Some([x, 0.0, 0.0]))
                })
                .and_then(|a| {
                    inner
                        .get_float_by_path("/Base/y")
                        .and_then(|y| Some([a[0], y, 0.0]))
                })
                .and_then(|a| {
                    inner
                        .get_float_by_path("/Base/z")
                        .and_then(|z| Some([a[0], a[1], z]))
                })
                .and_then(|a| Some(glam::Vec3::from_array(a)));
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct TypeTreeObjectBinReadArgs {
    serialized_file_id: i64,
    class_args: TypeTreeObjectBinReadClassArgs,
}

impl TypeTreeObjectBinReadArgs {
    pub fn new(serialized_file_id: i64, class_args: TypeTreeObjectBinReadClassArgs) -> Self {
        Self {
            serialized_file_id,
            class_args,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeTreeObjectBinReadClassArgs {
    class_id: i32,
    type_fields: Vec<Arc<Box<dyn TypeField + Send + Sync>>>,
}

impl TypeTreeObjectBinReadClassArgs {
    pub fn new(class_id: i32, type_fields: Vec<Arc<Box<dyn TypeField + Send + Sync>>>) -> Self {
        Self {
            class_id,
            type_fields,
        }
    }
}

impl BinRead for TypeTreeObject {
    type Args = TypeTreeObjectBinReadArgs;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        fn read<R: Read + Seek>(
            reader: &mut R,
            options: &ReadOptions,
            type_fields: &Vec<Arc<Box<dyn TypeField + Send + Sync>>>,
            field_index: &mut usize,
        ) -> BinResult<Field> {
            let time = std::time::Instant::now();
            let field = type_fields
                .get(*field_index)
                .ok_or(std::io::Error::from(ErrorKind::NotFound))?;
            let field_level = field.get_level();
            let field_value = if field.is_array() {
                *field_index += 1;
                let size_field = read(reader, options, type_fields, field_index)?;
                let mut size_reader = Cursor::new(match &size_field.data {
                    FieldValue::Data(data) => data,
                    _ => unreachable!(),
                });
                let size = <u32>::read_options(&mut size_reader, options, ())?;
                *field_index += 1;
                let item_field_index = *field_index;
                let item_type_field = type_fields
                    .get(item_field_index)
                    .ok_or(std::io::Error::from(ErrorKind::NotFound))?;
                let item_level = item_type_field.get_level();
                let mut item_type_fields = Vec::new();
                item_type_fields.push(item_type_field.clone());

                while let Some(next_field) = type_fields.get(*field_index + 1) {
                    if next_field.get_level() < item_level {
                        break;
                    }
                    item_type_fields.push(next_field.clone());
                    *field_index += 1;
                }

                let pos = reader.seek(SeekFrom::Current(0))?;
                let is_pos_aligned = (pos % 4) == 0;
                let fix_item_size = calc_no_array_field_size(&item_type_fields, &mut 0, &mut 0);
                let mut buf_read_flag = false;
                if let Some(byte_size) = fix_item_size {
                    if is_pos_aligned && ((byte_size % 4) == 0) {
                        buf_read_flag = true;
                    } else if item_type_fields.len() == 1
                        && (item_type_fields.get(0).unwrap().is_align() == false)
                    {
                        buf_read_flag = true;
                    }
                }

                if let (Some(byte_size), true) = (fix_item_size, buf_read_flag) {
                    let array = <Vec<u8>>::read_options(
                        reader,
                        options,
                        VecArgs {
                            count: byte_size * size as usize,
                            inner: (),
                        },
                    )?;

                    Field {
                        field_type: field.clone(),
                        data: FieldValue::Array(
                            ArrayField {
                                array_size: size_field,
                                item_type_fields,
                                data: FieldValue::Data(array),
                            }
                            .into(),
                        ),
                        time: time.elapsed(),
                    }
                } else {
                    let mut array = Vec::new();
                    for _ in 0..size as usize {
                        *field_index = item_field_index;
                        array.push(read(reader, options, type_fields, field_index)?);
                    }

                    Field {
                        field_type: field.clone(),
                        data: FieldValue::Array(
                            ArrayField {
                                array_size: size_field,
                                item_type_fields,
                                data: FieldValue::ArrayFields(array),
                            }
                            .into(),
                        ),
                        time: time.elapsed(),
                    }
                }
            } else if let Some(next_field) = type_fields.get(*field_index + 1) {
                if next_field.get_level() == field_level + 1 {
                    let mut fields = HashMap::new();
                    while let Some(next_field) = type_fields.get(*field_index + 1) {
                        if next_field.get_level() == field_level + 1 {
                            *field_index += 1;
                            let field_data = read(reader, options, type_fields, field_index)?;
                            fields.insert(field_data.get_name().clone(), field_data);
                        } else if next_field.get_level() <= field_level {
                            break;
                        } else {
                            panic!("{:#?} {:#?} ", next_field.get_level(), fields);
                        }
                    }

                    Field {
                        field_type: field.clone(),
                        data: FieldValue::Fields(fields),
                        time: time.elapsed(),
                    }
                } else {
                    Field::read_options(reader, options, field.clone())?
                }
            } else {
                Field::read_options(reader, options, field.clone())?
            };

            if field.is_align() {
                let pos = reader.seek(SeekFrom::Current(0))?;
                if pos % 4 != 0 {
                    reader.seek(SeekFrom::Current((4 - (pos % 4)) as i64))?;
                }
            }
            // println!("pos {:?}",reader.seek(SeekFrom::Current(0)));
            // println!("{:?}",&field_value.data);
            // field_value.display_field(&"".to_owned());
            Ok(field_value)
        }

        let mut index = 0;
        let data = read(reader, options, &args.class_args.type_fields, &mut index)?;

        Ok(TypeTreeObject {
            endian: options.endian(),
            class_id: args.class_args.class_id,
            serialized_file_id: args.serialized_file_id,
            data,
        })
    }
}

impl BinRead for Field {
    type Args = Arc<Box<dyn TypeField + Send + Sync>>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        let buff = <Vec<u8>>::read_options(
            reader,
            options,
            VecArgs {
                count: args.get_byte_size() as usize,
                inner: (),
            },
        )?;

        Ok(Field {
            field_type: args,
            data: FieldValue::Data(buff),
            time: Duration::from_secs(0),
        })
    }
}

fn calc_no_array_field_size(
    type_fields: &Vec<Arc<Box<dyn TypeField + Send + Sync>>>,
    field_index: &mut usize,
    read_size: &mut usize,
) -> Option<usize> {
    let field = type_fields.get(*field_index)?;
    let field_level = field.get_level();
    if field.is_array() {
        return None;
    } else if let Some(next_field) = type_fields.get(*field_index + 1) {
        if next_field.get_level() == field_level + 1 {
            while let Some(next_field) = type_fields.get(*field_index + 1) {
                if next_field.get_level() == field_level + 1 {
                    *field_index += 1;
                    calc_no_array_field_size(type_fields, field_index, read_size)?;
                } else if next_field.get_level() <= field_level {
                    break;
                } else {
                    panic!("{:#?} {:#?} ", next_field.get_level(), field);
                }
            }
        } else {
            *read_size += field.get_byte_size() as usize;
        }
    } else {
        *read_size += field.get_byte_size() as usize;
    };

    if field.is_align() {
        if *read_size % 4 != 0 {
            *read_size = *read_size + 4 - (*read_size % 4)
        }
    }
    Some(*read_size)
}
