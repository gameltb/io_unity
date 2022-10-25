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
pub enum Value {
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
    Array(Vec<FieldValue>),
    ByteArray(Vec<u8>),
}

#[derive(Debug)]
pub enum FieldValue {
    Data(Vec<u8>),
    Fields(Vec<Field>),
    Array(Box<ArrayField>),
}

#[derive(Debug)]
pub struct ArrayField {
    array_size: Field,
    item_type_fields: Vec<Arc<Box<dyn TypeField + Send>>>,
    data: FieldValue,
}

#[derive(Debug)]
pub struct Field {
    field_type: Arc<Box<dyn TypeField + Send>>,
    data: FieldValue,
}

impl Field {
    fn get_name(&self) -> &String {
        self.field_type.get_name()
    }

    fn display_field(&self, p: &String) {
        let np = p.clone() + "/" + self.field_type.get_name();
        println!(
            "{}/{} : {}",
            p,
            self.field_type.get_name(),
            self.field_type.get_type()
        );
        match &self.data {
            FieldValue::Data(_) => (),
            FieldValue::Fields(fls) => fls.into_iter().map(|f| f.display_field(&np)).collect(),
            FieldValue::Array(ar) => {
                ar.array_size.display_field(&np);
                if let FieldValue::Fields(ai) = &ar.data {
                    if let Some(aii) = ai.get(0) {
                        aii.display_field(&np);
                    }
                }
            }
        }
    }

    fn get_value(&self, path: &[String], endian: &binrw::Endian) -> Option<Value> {
        if path.len() == 0 {
            if let FieldValue::Data(data) = &self.data {
                return Some(Value::ByteArray(data.clone()));
            } else {
                if "string" == self.field_type.get_type() {
                    if let FieldValue::Fields(fields) = &self.data {
                        if let Some(array) = fields.get(0) {
                            if let Some(data) = array.get_array() {
                                return Some(Value::String(
                                    String::from_utf8(data.to_vec()).unwrap(),
                                ));
                            }
                        }
                    }
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
                FieldValue::Array(_array) => todo!(),
                _ => (),
            }
        }
        None
    }

    fn get_array(&self) -> Option<Cow<Vec<u8>>> {
        if let FieldValue::Array(array) = &self.data {
            if let FieldValue::Data(array_data) = &array.data {
                return Some(Cow::Borrowed(&array_data));
            }
        }
        None
    }
}

#[derive(Debug)]
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
}

#[derive(Debug, Clone)]
pub struct TypeTreeObjectBinReadArgs {
    class_id: i32,
    type_fields: Vec<Arc<Box<dyn TypeField + Send>>>,
}

impl TypeTreeObjectBinReadArgs {
    pub fn new(class_id: i32, type_fields: Vec<Arc<Box<dyn TypeField + Send>>>) -> Self {
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
            type_fields: &Vec<Arc<Box<dyn TypeField + Send>>>,
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
    type Args = Arc<Box<dyn TypeField + Send>>;

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
