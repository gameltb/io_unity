use thiserror::Error;

use crate::serialized_file::Object;

pub type ReadResult<T, E = Error> = core::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("TypeTreeObjectBinReadArgs can not build")]
    TypeTreeObjectBinReadArgsBuild,
    #[error("IO error while read : {0}")]
    IOError(#[from] std::io::Error),
    #[error("Binrw error while read : {0}")]
    BinrwError(#[from] binrw::Error),
    #[error("error while read object. data_offset: {data_offset} object : {object_meta:?} error : {source:?}")]
    ObjectReadError {
        source: Box<Error>,
        data_offset: u64,
        object_meta: Object,
    },
    #[error(
        "invalid type (want to cast to {want_to_cast:?}, found type name {found_type_name:?})"
    )]
    TypeMisMatch {
        want_to_cast: &'static str,
        found_type_name: String,
    },
    #[error("Field not found. path : {0:?}")]
    FieldNotFound(Vec<String>),
    #[error("Array field not found. path : {0:?}")]
    ArrayFieldNotFound(Vec<String>),
    #[error("can not find serialized file")]
    SerializedFileNotFound,
    #[error(
        "cannot find external serialized file. The serialized file may not has add to Viewer."
    )]
    ExternalSerializedFileNotFound,
    #[error("{0}")]
    AsSliceError(&'static str),
    #[error("ArrayItemOffset use without field offset.")]
    ArrayItemOffsetError,
    #[error("{0}")]
    Other(String),
    #[error("unknown error")]
    Unknown,
}
