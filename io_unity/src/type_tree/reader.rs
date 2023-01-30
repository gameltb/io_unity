use std::{
    collections::HashMap,
    fmt::Debug,
    io::{Cursor, ErrorKind, Read, Seek, SeekFrom},
    sync::Arc,
    time::Duration,
};

use binrw::{BinRead, BinResult, ReadOptions, VecArgs};

use crate::type_tree::{ArrayField, FieldValue, TypeTreeObject};

use super::{Field, TypeField};

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
                                data: FieldValue::ArrayItems(array),
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
            external_data: None,
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
