pub mod convert;
pub mod reader;
pub mod type_tree_json;

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use crate::type_tree::convert::TryCast;

use self::convert::FieldCastArgs;

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
    DataOffset(DataOffset),
    Fields(HashMap<String, Field>),
    Array(Box<ArrayField>),
}

#[derive(Debug, Clone)]
pub struct ArrayField {
    array_size: Field,
    item_type_fields: Vec<Arc<Box<dyn TypeField + Send + Sync>>>,
    item_field: Option<Field>,
    item_field_size: Option<u64>,
    data: ArrayFieldValue,
}

#[derive(Debug, Clone)]
pub enum ArrayFieldValue {
    DataOffset(DataOffset),
    ArrayItems(Vec<Field>),
}

#[derive(Debug, Clone)]
pub enum DataOffset {
    AbsDataOffset(u64),
    ArrayItemOffset(u64),
}

#[derive(Debug, Clone)]
pub struct Field {
    field_type: Arc<Box<dyn TypeField + Send + Sync>>,
    data: FieldValue,
}

impl Field {
    pub fn get_name(&self) -> &String {
        self.field_type.get_name()
    }

    pub fn get_type(&self) -> &String {
        self.field_type.get_type()
    }

    pub fn try_as_slice<'a>(
        &self,
        object_data_buff: &'a Vec<u8>,
        field_cast_args: &FieldCastArgs,
    ) -> Result<&'a [u8], ()> {
        let offset = field_cast_args.field_offset;
        let (pos, size) = match &self.data {
            FieldValue::DataOffset(data_offset) => {
                let pos = match data_offset {
                    DataOffset::AbsDataOffset(data) => *data,
                    DataOffset::ArrayItemOffset(data) => *data + offset.ok_or(())? as u64,
                };
                let size = self.field_type.get_byte_size() as u64;
                (pos, size)
            }
            FieldValue::Array(array) => match &array.data {
                ArrayFieldValue::DataOffset(data_offset) => {
                    let pos = match data_offset {
                        DataOffset::AbsDataOffset(data) => *data,
                        DataOffset::ArrayItemOffset(_) => return Err(()),
                    };
                    let array_size: i32 = array
                        .array_size
                        .try_cast_to(object_data_buff, field_cast_args)?;
                    if array_size < 0 {
                        return Err(());
                    }
                    let size = array.item_field_size.ok_or(())? * array_size as u64;
                    (pos, size)
                }
                ArrayFieldValue::ArrayItems(_) => return Err(()),
            },
            FieldValue::Fields(_) => return Err(()),
        };
        Ok(&object_data_buff[pos as usize..(pos + size) as usize])
    }

    pub fn try_get_buff_type_and_type_size(&self) -> Option<(&String, i32)> {
        if let FieldValue::Array(ar) = &self.data {
            if let ArrayFieldValue::DataOffset(_) = &ar.data {
                if let Some(item_field) = ar.item_field.as_ref() {
                    if let FieldValue::DataOffset(_) = item_field.data {
                        let item_type = &item_field.field_type;
                        return Some((item_type.get_type(), item_type.get_byte_size()));
                    }
                }
            }
        }
        None
    }

    fn display_field(
        &self,
        p: &String,
        object_data_buff: &Vec<u8>,
        field_cast_args: &mut FieldCastArgs,
    ) {
        let np = p.clone() + "/" + self.field_type.get_name();
        print!(
            "{}/{} : {}({})",
            p,
            self.field_type.get_name(),
            self.field_type.get_type(),
            self.field_type.get_byte_size(),
        );
        match &self.data {
            FieldValue::DataOffset(_v) => {
                let num: Result<i64, ()> = self.try_cast_to(object_data_buff, field_cast_args);
                if let Ok(num) = num {
                    println!(" data : {:?}", num);
                } else {
                    let num: Result<u64, ()> = self.try_cast_to(object_data_buff, field_cast_args);
                    if let Ok(num) = num {
                        println!(" data : {:?}", num);
                    } else {
                        let num: Result<f32, ()> =
                            self.try_cast_to(object_data_buff, field_cast_args);
                        if let Ok(num) = num {
                            println!(" data : {:?}", num);
                        } else {
                            println!("");
                        }
                    }
                }
            }
            FieldValue::Fields(fls) => {
                println!("");
                fls.into_iter()
                    .map(|(_n, f)| f.display_field(&np, object_data_buff, field_cast_args))
                    .collect()
            }
            FieldValue::Array(ar) => {
                println!("");
                ar.array_size
                    .display_field(&np, object_data_buff, field_cast_args);
                match &ar.data {
                    ArrayFieldValue::ArrayItems(ai) => {
                        if let Some(aii) = ai.get(0) {
                            aii.display_field(&np, object_data_buff, field_cast_args);
                        } else {
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
                    }
                    ArrayFieldValue::DataOffset(_) => {
                        if let Some(item_field) = &ar.item_field {
                            item_field.display_field(&np, object_data_buff, field_cast_args);
                        }
                    }
                }
            }
        }
    }

    fn get_field(
        &self,
        path: &[String],
        field_offset: Option<i64>,
        type_tree_object: &TypeTreeObject,
    ) -> Option<(&Self, Option<i64>)> {
        if path.len() == 0 {
            return Some((self, field_offset));
        } else {
            match &self.data {
                FieldValue::Fields(fields) => {
                    if let Some((name, path)) = path.split_first() {
                        if let Some(field) = fields.get(name) {
                            return field.get_field(path, field_offset, type_tree_object);
                        }
                    }
                }
                FieldValue::Array(fields) => {
                    if let Some((index, path)) = path.split_first() {
                        if let Ok(index) = index.parse::<i32>() {
                            match &fields.data {
                                ArrayFieldValue::DataOffset(offset) => {
                                    assert_eq!(field_offset, None);
                                    let item_size = fields.item_field_size?;
                                    let field = fields.item_field.as_ref()?;
                                    let DataOffset::AbsDataOffset(offset) = offset else {
                                        return None;
                                    };
                                    let size: i32 = fields
                                        .array_size
                                        .try_cast_to(
                                            &type_tree_object.data_buff,
                                            &type_tree_object.get_field_cast_args(),
                                        )
                                        .ok()?;
                                    if size <= index {
                                        return None;
                                    }
                                    return field.get_field(
                                        path,
                                        Some((*offset + item_size * index as u64) as i64),
                                        type_tree_object,
                                    );
                                }
                                ArrayFieldValue::ArrayItems(items) => {
                                    if let Some(field) = items.get(index as usize) {
                                        return field.get_field(
                                            path,
                                            field_offset,
                                            type_tree_object,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }
        None
    }
}

// todo: cache get layout
#[derive(Debug, Clone)]
pub struct TypeTreeObject {
    endian: binrw::Endian,
    pub class_id: i32,
    pub serialized_file_id: i64,
    data_layout: Field,
    data_buff: Vec<u8>,
    pub external_data: Option<Vec<u8>>,
}

impl TypeTreeObject {
    pub fn display_tree(&self) {
        println!("class_id : {}", self.class_id);
        self.data_layout.display_field(
            &"".to_string(),
            &self.data_buff,
            &mut self.get_field_cast_args(),
        );
    }

    pub fn get_endian(&self) -> binrw::Endian {
        self.endian.clone()
    }

    pub fn try_as_slice(&self, path: &str) -> Result<&[u8], ()> {
        self.get_field_by_path(path)
            .and_then(|(feild, offset)| {
                let mut field_cast_args = self.get_field_cast_args();
                field_cast_args.field_offset = offset;
                feild.try_as_slice(&self.data_buff, &field_cast_args).ok()
            })
            .ok_or(())
    }

    pub(super) fn get_field_by_path(&self, path: &str) -> Option<(&Field, Option<i64>)> {
        let path: Vec<String> = path
            .split("/")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        if path.len() < 1 {
            return None;
        }
        self.data_layout.get_field(&path[1..], None, &self)
    }

    pub(super) fn get_field_by_path_list(&self, path: &[String]) -> Option<(&Field, Option<i64>)> {
        if path.len() == 0 {
            return Some((&self.data_layout, None));
        }
        self.data_layout.get_field(&path, None, &self)
    }

    pub(super) fn get_field_cast_args(&self) -> FieldCastArgs {
        FieldCastArgs {
            endian: self.endian.clone(),
            field_offset: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeTreeObjectRef {
    inner: Arc<RwLock<Box<TypeTreeObject>>>,
    pub path: Vec<String>,
}

impl Into<TypeTreeObjectRef> for TypeTreeObject {
    fn into(self) -> TypeTreeObjectRef {
        TypeTreeObjectRef {
            inner: Arc::new(RwLock::new(Box::new(self))),
            path: vec![],
        }
    }
}

impl TypeTreeObjectRef {
    pub fn inner(&self) -> &Arc<RwLock<Box<TypeTreeObject>>> {
        &self.inner
    }

    pub fn get_name(&self) -> Option<String> {
        Some(
            self.inner
                .read()
                .unwrap()
                .get_field_by_path_list(&self.path)?
                .0
                .get_name()
                .to_owned(),
        )
    }

    pub fn get_type(&self) -> Option<String> {
        Some(
            self.inner
                .read()
                .unwrap()
                .get_field_by_path_list(&self.path)?
                .0
                .get_type()
                .to_owned(),
        )
    }

    pub fn try_get_buff_type_and_type_size(&self) -> Option<(String, i32)> {
        self.inner
            .read()
            .unwrap()
            .get_field_by_path_list(&self.path)?
            .0
            .try_get_buff_type_and_type_size()
            .and_then(|(n, s)| Some((n.to_owned(), s)))
    }

    pub fn display_tree(&self) {
        self.inner.read().unwrap().display_tree()
    }

    pub fn get_endian(&self) -> binrw::Endian {
        self.inner.read().unwrap().endian.clone()
    }

    pub fn get_serialized_file_id(&self) -> i64 {
        self.inner.read().unwrap().serialized_file_id
    }

    pub fn get_class_id(&self) -> i32 {
        self.inner.read().unwrap().class_id
    }
}
