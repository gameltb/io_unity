pub mod convert;
pub mod reader;
pub mod type_tree_json;

use std::{collections::HashMap, fmt::Debug, sync::Arc, time::Duration};

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
    Data(Vec<u8>),
    Fields(HashMap<String, Field>),
    Array(Box<ArrayField>),
    ArrayItems(Vec<Field>),
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

impl Field {
    pub fn get_name(&self) -> &String {
        self.field_type.get_name()
    }

    pub fn get_type(&self) -> &String {
        self.field_type.get_type()
    }

    pub fn try_get_buff_type_and_type_size(&self) -> Option<(&String, i32)> {
        if let FieldValue::Array(ar) = &self.data {
            if let FieldValue::Data(_) = &ar.data {
                if ar.item_type_fields.len() == 1 {
                    let item_type = &ar.item_type_fields.get(0).unwrap();
                    return Some((item_type.get_type(), item_type.get_byte_size()));
                }
            }
        }
        None
    }

    fn display_field(&self, p: &String, field_cast_args: &FieldCastArgs) {
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
                let num: Result<i64, ()> = self.try_cast_to(field_cast_args);
                if let Ok(num) = num {
                    println!(" data : {:?}", num);
                } else if v.len() <= 8 {
                    println!(" data : {:?}", v);
                } else {
                    println!(" data : {:?}...", &v[..8]);
                }
            }
            FieldValue::Fields(fls) => {
                println!("");
                fls.into_iter()
                    .map(|(_n, f)| f.display_field(&np, field_cast_args))
                    .collect()
            }
            FieldValue::Array(ar) => {
                println!("");
                ar.array_size.display_field(&np, field_cast_args);
                match &ar.data {
                    FieldValue::ArrayItems(ai) => {
                        if let Some(aii) = ai.get(0) {
                            aii.display_field(&np, field_cast_args);
                        }
                    }
                    _ => {
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
            }
            FieldValue::ArrayItems(_) => (),
        }
    }

    pub fn get_field(&self, path: &[String]) -> Option<&Self> {
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
        self.data
            .display_field(&"".to_string(), &self.get_field_cast_args());
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

    pub fn get_field_by_name(&self, name: &str) -> Option<&Field> {
        if name.len() == 0 {
            return Some(&self.data);
        }
        self.data.get_field(&vec![name.to_string()])
    }

    pub fn get_field_cast_args(&self) -> FieldCastArgs {
        FieldCastArgs {
            endian: self.endian.clone(),
            serialized_file_id: self.serialized_file_id,
        }
    }
}
