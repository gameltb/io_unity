use super::AssetBundleObject;
use crate::classes::p_ptr::PPtr;
use crate::until::binrw_parser::AlignedString;
use crate::SerializedFileMetadata;
use binrw::binrw;

impl AssetBundleObject for AssetBundle {}

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct AssetBundle {
    name: AlignedString,
    preload_table_size: i32,
    #[br(count = preload_table_size,args { inner: args.clone() })]
    #[bw(args_raw = args.clone())]
    preload_table: Vec<PPtr>,
    container_size: i32,
    #[br(count = container_size,args { inner: AssetInfoBinReadArgs { args : args } })]
    #[bw(args { args : args })]
    container: Vec<AssetInfo>,
}

#[binrw]
#[brw(import{ args: SerializedFileMetadata })]
#[derive(Debug)]
struct AssetInfo {
    name: AlignedString,
    preload_index: i32,
    preload_size: i32,
    #[brw(args_raw = args)]
    asset: PPtr,
}
