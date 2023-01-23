use std::fs::File;
use std::io::{prelude::*, BufReader, ErrorKind, SeekFrom};

use std::sync::{Arc, Mutex};

use binrw::{binrw, BinResult, NullString, ReadOptions};
use binrw::{io::Cursor, BinRead};
use lz4::block::decompress;
use modular_bitfield::specifiers::{B22, B9};
use modular_bitfield::{bitfield, BitfieldSpecifier};
use num_enum::TryFromPrimitive;

use crate::until::binrw_parser::position_parser; // reading/writing utilities

pub trait UnityResource: std::io::Read + std::io::Seek {}

impl UnityResource for std::io::Cursor<Vec<u8>> {}
impl UnityResource for BufReader<File> {}

#[bitfield]
#[derive(Clone, Copy, Debug, PartialEq)]
#[binrw]
#[br(map = |x:u32| Self::from_bytes(x.to_le_bytes()))]
#[bw(map = |&x| <u32>::from_le_bytes(Self::into_bytes(x)))]
pub struct ArchiveFlags {
    #[bits = 6]
    compression_type: CompressionType,
    blocks_and_directory_info_combined: bool,
    blocks_info_at_the_end: bool,
    old_web_plugin_compatibility: bool,
    block_info_need_padding_at_start: bool,
    #[skip]
    __: B22,
}

#[bitfield]
#[derive(Clone, Copy, Debug, PartialEq)]
#[binrw]
#[br(map = |x:u16| Self::from_bytes(x.to_le_bytes()))]
#[bw(map = |&x| <u16>::from_le_bytes(Self::into_bytes(x)))]
pub struct StorageBlockFlags {
    #[bits = 6]
    compression_type: CompressionType,
    streamed: bool,
    #[skip]
    __: B9,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, BitfieldSpecifier)]
#[repr(u32)]
#[bits = 6]
enum CompressionType {
    None = 0,
    Lzma,
    Lz4,
    Lz4HC,
    Lzham,
}

#[derive(Clone)]
pub struct UnityFS {
    content: UnityFSFile,
    file_reader: Arc<Mutex<Box<dyn UnityResource + Send>>>,
    pub resource_search_path: Option<String>,
}

#[binrw]
#[brw(big)]
#[brw(magic = b"UnityFS\0")]
#[derive(Clone, Debug, PartialEq)]
pub struct UnityFSFile {
    version: u32,
    unity_version: NullString,
    unity_revision: NullString,
    size: i64,
    compressed_blocks_info_size: u32,
    uncompressed_blocks_info_size: u32,
    flags: ArchiveFlags,
    #[br(parse_with = blocks_info_parser, args (version, compressed_blocks_info_size,uncompressed_blocks_info_size,flags))]
    blocks_info: BlocksInfo,
    #[br(parse_with = position_parser)]
    #[bw(ignore)]
    position: u64,
}

impl UnityFS {
    pub fn get_file_by_path(&self, path: &String) -> std::io::Result<Vec<u8>> {
        for node in self.get_files() {
            if path == &node.path() {
                return self.get_file_by_node(node);
            }
        }
        Err(std::io::Error::from(ErrorKind::NotFound))
    }

    fn get_file_by_node(&self, node: &Node) -> std::io::Result<Vec<u8>> {
        let mut compressed_data_offset = 0u64;
        let mut uncompressed_data_offset = 0u64;
        let mut file_block = Vec::new();
        for sb in &self.content.blocks_info.storage_blocks {
            if (uncompressed_data_offset + (sb.uncompressed_size as u64)) >= node.offset as u64 {
                let mut blocks_infocompressedd_stream = vec![0u8; sb.compressed_size as usize];
                if let Ok(mut file_reader) = self.file_reader.lock() {
                    file_reader.seek(SeekFrom::Start(
                        compressed_data_offset + self.content.position,
                    ))?;
                    file_reader.read_exact(&mut blocks_infocompressedd_stream)?;
                } else {
                    return Err(std::io::Error::from(ErrorKind::BrokenPipe));
                }

                let mut blocks_info_uncompressedd_stream = block_uncompressed(
                    sb.uncompressed_size as u64,
                    sb.flags.compression_type(),
                    blocks_infocompressedd_stream,
                )?;
                if uncompressed_data_offset < node.offset as u64 {
                    blocks_info_uncompressedd_stream = blocks_info_uncompressedd_stream
                        [(node.offset as u64 - uncompressed_data_offset) as usize..]
                        .to_vec();
                }
                file_block.extend(blocks_info_uncompressedd_stream);
                if file_block.len() >= node.size as usize {
                    return Ok(file_block.split_at(node.size as usize).0.to_vec());
                }
            }
            compressed_data_offset += sb.compressed_size as u64;
            uncompressed_data_offset += sb.uncompressed_size as u64;
        }
        Err(std::io::Error::from(ErrorKind::NotFound))
    }

    pub fn get_files(&self) -> &Vec<Node> {
        return &self.content.blocks_info.directory_info;
    }

    pub fn get_cab_path(&self) -> Vec<String> {
        let mut paths = vec![];
        for file in self.get_files() {
            let path = file.path();
            if path.starts_with("CAB-") && (path.len() == 36) {
                paths.push(path);
            }
        }
        paths
    }

    pub fn get_file_paths(&self) -> Vec<String> {
        let mut paths = vec![];
        for file in self.get_files() {
            paths.push(file.path());
        }
        paths
    }

    pub fn read(
        mut file: Box<dyn UnityResource + Send>,
        resource_search_path: Option<String>,
    ) -> BinResult<UnityFS> {
        Ok(UnityFS {
            content: UnityFSFile::read(&mut file)?,
            file_reader: Arc::new(Mutex::new(file)),
            resource_search_path,
        })
    }
}

fn block_uncompressed(
    uncompressed_size: u64,
    flag: CompressionType,
    blocks_infocompressedd_stream: Vec<u8>,
) -> std::io::Result<Vec<u8>> {
    let blocks_info_uncompressedd_stream;
    match flag {
        CompressionType::None => blocks_info_uncompressedd_stream = blocks_infocompressedd_stream,
        CompressionType::Lzma => todo!(),
        CompressionType::Lz4 | CompressionType::Lz4HC => {
            blocks_info_uncompressedd_stream = decompress(
                &blocks_infocompressedd_stream,
                Some(uncompressed_size as i32),
            )?;
        }
        CompressionType::Lzham => todo!(),
    }
    Ok(blocks_info_uncompressedd_stream)
}

#[binrw]
#[br(big)]
#[derive(Clone, Debug, PartialEq)]
struct BlocksInfo {
    uncompressed_data_hash: [u8; 16],
    blocks_info_count: u32,
    #[br(count = blocks_info_count)]
    storage_blocks: Vec<StorageBlock>,
    nodes_count: u32,
    #[br(count = nodes_count)]
    directory_info: Vec<Node>,
}

#[binrw]
#[br(big)]
#[derive(Clone, Debug, PartialEq)]
struct StorageBlock {
    uncompressed_size: u32,
    compressed_size: i32,
    flags: StorageBlockFlags,
}

#[binrw]
#[br(big)]
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    offset: i64,
    size: i64,
    flags: u32,
    path: NullString,
}

impl Node {
    pub fn path(&self) -> String {
        self.path.clone().to_string()
    }
}

fn blocks_info_parser<R: Read + Seek>(
    reader: &mut R,
    _ro: &ReadOptions,
    flags: (u32, u32, u32, ArchiveFlags),
) -> BinResult<BlocksInfo> {
    let (version, compressed_blocks_info_size, uncompressed_blocks_info_size, flags) = flags;

    if version >= 7 {
        let pos = reader.seek(SeekFrom::Current(0))?;
        if pos % 16 != 0 {
            reader.seek(SeekFrom::Current((16 - (pos % 16)) as i64))?;
        }
    }

    let mut blocks_infocompressedd_stream = vec![0u8; compressed_blocks_info_size as usize];

    if flags.blocks_info_at_the_end() {
        let pos = reader.seek(SeekFrom::Current(0))?;
        reader.seek(SeekFrom::End(-(compressed_blocks_info_size as i64)))?;
        reader.read_exact(&mut blocks_infocompressedd_stream)?;
        reader.seek(SeekFrom::Start(pos))?;
    } else {
        reader.read_exact(&mut blocks_infocompressedd_stream)?;
    }

    if flags.block_info_need_padding_at_start() {
        let pos = reader.seek(SeekFrom::Current(0))?;
        if pos % 16 != 0 {
            reader.seek(SeekFrom::Current((16 - (pos % 16)) as i64))?;
        }
    }

    let blocks_info_uncompressedd_stream = block_uncompressed(
        uncompressed_blocks_info_size as u64,
        flags.compression_type(),
        blocks_infocompressedd_stream,
    )?;

    let mut blocks_info_reader = Cursor::new(blocks_info_uncompressedd_stream);
    BlocksInfo::read(&mut blocks_info_reader)
}
