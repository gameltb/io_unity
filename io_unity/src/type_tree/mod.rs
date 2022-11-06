use std::{
    borrow::Cow,
    fmt::Debug,
    io::{Cursor, Read, Seek, SeekFrom},
    sync::Arc,
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

#[derive(Debug)]
pub enum Value<'a> {
    Bool(bool),
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float(f32),
    Double(f64),
    String(String),
    Array(Vec<TypeTreeObject>),
    Object(TypeTreeObject),
    ByteArray(Cow<'a, Vec<u8>>),
}

#[derive(Debug, Clone)]
pub enum FieldValue {
    Data(Vec<u8>),
    Fields(Vec<Field>),
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
}

impl Field {
    fn get_name(&self) -> &String {
        self.field_type.get_name()
    }

    fn display_field(&self, p: &String) {
        let np = p.clone() + "/" + self.field_type.get_name();
        println!(
            "{}/{} : {}({})",
            p,
            self.field_type.get_name(),
            self.field_type.get_type(),
            self.field_type.get_byte_size()
        );
        match &self.data {
            FieldValue::Data(_) => (),
            FieldValue::Fields(fls) => fls.into_iter().map(|f| f.display_field(&np)).collect(),
            FieldValue::Array(ar) => {
                ar.array_size.display_field(&np);
                match &ar.data {
                    FieldValue::Fields(ai) => {
                        if let Some(aii) = ai.get(0) {
                            aii.display_field(&np);
                        }
                    }
                    FieldValue::Data(_) => {
                        for item in &ar.item_type_fields {
                            println!(
                                "{}/{} : {}({})",
                                np,
                                item.get_name(),
                                item.get_type(),
                                item.get_byte_size()
                            );
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    fn get_value(&self, path: &[String], endian: &binrw::Endian) -> Option<Value> {
        if path.len() == 0 {
            match &self.data {
                FieldValue::Data(data) => {
                    let op = ReadOptions::new(endian.clone());
                    match self.field_type.get_type().as_str() {
                        "bool" => {
                            if let Some(i) = data.get(0) {
                                return Some(Value::Bool(*i != 0));
                            }
                        }
                        "SInt8" => {
                            if let Ok(i) = <i8>::read(&mut Cursor::new(data)) {
                                return Some(Value::Int8(i));
                            }
                        }
                        "SInt16" | "short" => {
                            if let Ok(i) = <i16>::read_options(&mut Cursor::new(data), &op, ()) {
                                return Some(Value::Int16(i));
                            }
                        }
                        "SInt32" | "int" => {
                            if let Ok(i) = <i32>::read_options(&mut Cursor::new(data), &op, ()) {
                                return Some(Value::Int32(i));
                            }
                        }
                        "SInt64" | "long long" => {
                            if let Ok(i) = <i64>::read_options(&mut Cursor::new(data), &op, ()) {
                                return Some(Value::Int64(i));
                            }
                        }
                        "UInt8" | "char" => {
                            if let Ok(i) = <u8>::read(&mut Cursor::new(data)) {
                                return Some(Value::UInt8(i));
                            }
                        }
                        "UInt16" | "unsigned short" => {
                            if let Ok(i) = <u16>::read_options(&mut Cursor::new(data), &op, ()) {
                                return Some(Value::UInt16(i));
                            }
                        }
                        "UInt32" | "unsigned int" => {
                            if let Ok(i) = <u32>::read_options(&mut Cursor::new(data), &op, ()) {
                                return Some(Value::UInt32(i));
                            }
                        }
                        "UInt64" | "unsigned long long" | "FileSize" => {
                            if let Ok(i) = <u64>::read_options(&mut Cursor::new(data), &op, ()) {
                                return Some(Value::UInt64(i));
                            }
                        }
                        "float" => {
                            if let Ok(i) = <f32>::read_options(&mut Cursor::new(data), &op, ()) {
                                return Some(Value::Float(i));
                            }
                        }
                        "double" => {
                            if let Ok(i) = <f64>::read_options(&mut Cursor::new(data), &op, ()) {
                                return Some(Value::Double(i));
                            }
                        }
                        &_ => (),
                    }
                    return Some(Value::ByteArray(Cow::Borrowed(data)));
                }
                FieldValue::Array(array_field) => match &array_field.data {
                    FieldValue::Data(array) => {
                        if array.len() > 0 {
                            return Some(Value::ByteArray(Cow::Borrowed(array)));
                        }
                    }
                    FieldValue::Fields(array_object) => {
                        if array_object.len() > 0 {
                            let array = array_object
                                .into_iter()
                                .map(|f| TypeTreeObject {
                                    endian: endian.clone(),
                                    class_id: 0,
                                    data: f.clone(),
                                })
                                .collect();
                            return Some(Value::Array(array));
                        }
                    }
                    _ => (),
                },
                FieldValue::Fields(fields) => {
                    if "string" == self.field_type.get_type() {
                        if let Some(array) = fields.get(0) {
                            if let Some(data) = array.get_sized_array_buff() {
                                return Some(Value::String(
                                    String::from_utf8(data.to_vec()).unwrap(),
                                ));
                            }
                        }
                    }
                    return Some(Value::Object(TypeTreeObject {
                        endian: endian.clone(),
                        class_id: 0,
                        data: self.clone(),
                    }));
                }
            }
        } else {
            match &self.data {
                FieldValue::Fields(fields) => {
                    if let Some((name, path)) = path.split_first() {
                        for field in fields {
                            if name == field.get_name() {
                                return field.get_value(path, endian);
                            }
                        }
                    }
                }
                FieldValue::Array(_array) => (),
                _ => (),
            }
        }
        None
    }

    fn get_sized_array_buff(&self) -> Option<Cow<Vec<u8>>> {
        if let FieldValue::Array(array) = &self.data {
            if let FieldValue::Data(array_data) = &array.data {
                return Some(Cow::Borrowed(&array_data));
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct TypeTreeObject {
    endian: binrw::Endian,
    class_id: i32,
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

    pub fn get_value_by_path(&self, path: &str) -> Option<Value> {
        let path: Vec<String> = path
            .split("/")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        if path.len() < 1 {
            return None;
        }
        self.data.get_value(&path[1..], &self.endian)
    }

    pub fn get_string_by_path(&self, path: &str) -> Option<String> {
        if let Some(v) = self.get_value_by_path(path) {
            if let Value::String(s) = v {
                return Some(s);
            }
        }
        None
    }

    pub fn get_byte_array_by_path(&self, path: &str) -> Option<Cow<Vec<u8>>> {
        if let Some(v) = self.get_value_by_path(path) {
            if let Value::ByteArray(ao) = v {
                return Some(ao);
            }
        }
        None
    }

    pub fn get_array_object_by_path(&self, path: &str) -> Option<Vec<TypeTreeObject>> {
        if let Some(v) = self.get_value_by_path(path) {
            if let Value::Array(ao) = v {
                return Some(ao);
            }
        }
        None
    }

    pub fn get_bool_by_path(&self, path: &str) -> Option<bool> {
        if let Some(v) = self.get_value_by_path(path) {
            return match v {
                Value::Bool(i) => Some(i),
                _ => None,
            };
        }
        None
    }

    pub fn get_float_by_path(&self, path: &str) -> Option<f32> {
        if let Some(v) = self.get_value_by_path(path) {
            return match v {
                Value::Float(i) => Some(i),
                _ => None,
            };
        }
        None
    }

    pub fn get_double_by_path(&self, path: &str) -> Option<f64> {
        if let Some(v) = self.get_value_by_path(path) {
            return match v {
                Value::Double(i) => Some(i),
                _ => None,
            };
        }
        None
    }

    pub fn get_int_by_path(&self, path: &str) -> Option<i64> {
        if let Some(v) = self.get_value_by_path(path) {
            return match v {
                Value::Int8(i) => Some(i as i64),
                Value::Int16(i) => Some(i as i64),
                Value::Int32(i) => Some(i as i64),
                Value::Int64(i) => Some(i as i64),
                _ => None,
            };
        }
        None
    }

    pub fn get_uint_by_path(&self, path: &str) -> Option<u64> {
        if let Some(v) = self.get_value_by_path(path) {
            return match v {
                Value::UInt8(i) => Some(i as u64),
                Value::UInt16(i) => Some(i as u64),
                Value::UInt32(i) => Some(i as u64),
                Value::UInt64(i) => Some(i as u64),
                _ => None,
            };
        }
        None
    }

    pub fn get_object_by_path(&self, path: &str) -> Option<TypeTreeObject> {
        if let Some(v) = self.get_value_by_path(path) {
            return match v {
                Value::Object(inner) => Some(inner),
                _ => None,
            };
        }
        None
    }

    pub fn get_quat_by_path(&self, path: &str) -> Option<glam::Quat> {
        if let Some(v) = self.get_value_by_path(path) {
            return match v {
                Value::Object(inner) => Some([0f32; 4])
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
                    .and_then(|a| Some(glam::Quat::from_array(a))),
                _ => None,
            };
        }
        None
    }

    pub fn get_vec3f_by_path(&self, path: &str) -> Option<glam::Vec3> {
        if let Some(v) = self.get_value_by_path(path) {
            return match v {
                Value::Object(inner) => Some([0f32; 3])
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
                    .and_then(|a| Some(glam::Vec3::from_array(a))),
                _ => None,
            };
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct TypeTreeObjectBinReadArgs {
    class_id: i32,
    type_fields: Vec<Arc<Box<dyn TypeField + Send + Sync>>>,
}

impl TypeTreeObjectBinReadArgs {
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
            let field = type_fields.get(*field_index).unwrap().to_owned();
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
                let item_type_field = type_fields.get(item_field_index).unwrap();
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

                if item_type_fields.len() == 1 {
                    let item_type = item_type_fields.get(0).unwrap();

                    let mut byte_size = item_type.get_byte_size() as usize;

                    if item_type.is_align() {
                        let i = byte_size % 4;
                        if i != 0 {
                            byte_size = byte_size - i + 4
                        }
                    }

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
                                data: FieldValue::Fields(array),
                            }
                            .into(),
                        ),
                    }
                }
            } else if let Some(next_field) = type_fields.get(*field_index + 1) {
                if next_field.get_level() == field_level + 1 {
                    let mut fields = Vec::new();
                    while let Some(next_field) = type_fields.get(*field_index + 1) {
                        if next_field.get_level() == field_level + 1 {
                            *field_index += 1;
                            fields.push(read(reader, options, type_fields, field_index)?);
                        } else if next_field.get_level() <= field_level {
                            break;
                        } else {
                            panic!("{:#?} {:#?} ", next_field.get_level(), fields);
                        }
                    }

                    Field {
                        field_type: field.clone(),
                        data: FieldValue::Fields(fields),
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

            Ok(field_value)
        }

        let mut index = 0;
        let data = read(reader, options, &args.type_fields, &mut index)?;

        Ok(TypeTreeObject {
            endian: options.endian(),
            class_id: args.class_id,
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
        })
    }
}
