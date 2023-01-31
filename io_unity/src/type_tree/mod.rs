pub mod convert;
pub mod reader;
pub mod type_tree_json;

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
    time::Duration,
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
    DataOffset(u64),
    Fields(HashMap<String, Field>),
    Array(Box<ArrayField>),
    ArrayItems(Vec<Field>),
}

#[derive(Debug, Clone)]
pub struct ArrayField {
    array_size: Field,
    item_field: Option<Field>,
    item_field_size: Option<u64>,
    data: FieldValue,
}

#[derive(Debug, Clone)]
pub struct Field {
    field_type: Arc<Box<dyn TypeField + Send + Sync>>,
    data: FieldValue,
    time: Duration,
}

impl Field {
    pub fn get_name(&self) -> &String {
        self.field_type.get_name()
    }

    pub fn get_type(&self) -> &String {
        self.field_type.get_type()
    }

    pub fn try_get_buff_type_and_type_size(&self) -> Option<(&String, i32)> {
        if let FieldValue::Array(ar) = &self.data {
            if let FieldValue::DataOffset(_) = &ar.data {
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
            "{}/{} : {}({}) {:?}",
            p,
            self.field_type.get_name(),
            self.field_type.get_type(),
            self.field_type.get_byte_size(),
            &self.time
        );
        match &self.data {
            FieldValue::DataOffset(v) => {
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
                    FieldValue::ArrayItems(ai) => {
                        if let Some(aii) = ai.get(0) {
                            aii.display_field(&np, object_data_buff, field_cast_args);
                        }
                    }
                    _ => {
                        if let Some(item_field) = &ar.item_field {
                            item_field.display_field(&np, object_data_buff, field_cast_args);
                        }
                    }
                }
            }
            FieldValue::ArrayItems(_) => (),
        }
    }

    fn get_field(
        &self,
        path: &[String],
        field_offset: i64,
        type_tree_object: &TypeTreeObject,
    ) -> Option<(&Self, i64)> {
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
                                FieldValue::DataOffset(offset) => {
                                    assert_eq!(field_offset, 0);
                                    let item_size = fields.item_field_size?;
                                    let field = fields.item_field.as_ref()?;
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
                                        (*offset + item_size * index as u64) as i64,
                                        type_tree_object,
                                    );
                                }
                                FieldValue::ArrayItems(items) => {
                                    if let Some(field) = items.get(index as usize) {
                                        return field.get_field(
                                            path,
                                            field_offset,
                                            type_tree_object,
                                        );
                                    }
                                }
                                _ => (),
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

    pub fn get_field_by_path(&self, path: &str) -> Option<(&Field, i64)> {
        let path: Vec<String> = path
            .split("/")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        if path.len() < 1 {
            return None;
        }
        self.data_layout.get_field(&path[1..], 0, &self)
    }

    pub fn get_field_by_path_list(&self, path: &Vec<String>) -> Option<(&Field, i64)> {
        if path.len() == 0 {
            return Some((&self.data_layout, 0));
        }
        self.data_layout.get_field(&path[1..], 0, &self)
    }

    pub fn get_field_cast_args(&self) -> FieldCastArgs {
        FieldCastArgs {
            endian: self.endian.clone(),
            serialized_file_id: self.serialized_file_id,
            field_offset: 0,
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
            path: vec!["Base".to_owned()],
        }
    }
}

impl TypeTreeObjectRef {
    pub fn display_tree(&self) {
        self.inner.read().unwrap().display_tree()
    }

    pub fn get_endian(&self) -> binrw::Endian {
        self.inner.read().unwrap().endian.clone()
    }

    pub fn get_serialized_file_id(&self) -> i64 {
        self.inner.read().unwrap().serialized_file_id
    }

    pub fn get_field_cast_args(&self) -> FieldCastArgs {
        self.inner.read().unwrap().get_field_cast_args()
    }
}
