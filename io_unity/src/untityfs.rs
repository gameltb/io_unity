use std::convert::TryFrom;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader, ErrorKind, SeekFrom};
use std::path::PathBuf;

use std::sync::{Arc, Mutex};

use binrw::{binrw, BinResult, NullString, ReadOptions};
use binrw::{io::Cursor, BinRead};
use lz4::block::decompress;

use num_enum::TryFromPrimitive;

use crate::until::binrw_parser::position_parser; // reading/writing utilities

pub trait UnityResource: std::io::Read + std::io::Seek {}

impl UnityResource for std::io::Cursor<Vec<u8>> {}
impl UnityResource for BufReader<File> {}

pub trait FS {
    fn get_resource_file_by_path(
        &self,
        path: String,
        search_path: Option<&String>,
    ) -> Option<Box<dyn UnityResource>>;
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
enum ArchiveFlags {
    CompressionTypeMask = 0x3f,
    BlocksAndDirectoryInfoCombined = 0x40,
    BlocksInfoAtTheEnd = 0x80,
    OldWebPluginCompatibility = 0x100,
    BlockInfoNeedPaddingAtStart = 0x200,
}

#[derive(Debug, Eq, PartialEq)]
enum StorageBlockFlags {
    CompressionTypeMask = 0x3f,
    Streamed = 0x40,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
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
    search_path: Option<String>,
}

#[binrw]
#[br(big)]
#[derive(Clone, Debug, PartialEq)]
pub struct UnityFSFile {
    signature: NullString,
    version: u32,
    unity_version: NullString,
    unity_revision: NullString,
    size: i64,
    compressed_blocks_info_size: u32,
    uncompressed_blocks_info_size: u32,
    flags: u32,
    #[br(parse_with = blocks_info_parser, args (version, compressed_blocks_info_size,uncompressed_blocks_info_size,flags))]
    blocks_info: BlocksInfo,
    #[br(parse_with = position_parser)]
    #[bw(ignore)]
    position: u64,
}

impl UnityFS {
    pub fn get_file_by_path(&self, path: String) -> std::io::Result<Vec<u8>> {
        for node in self.get_files() {
            if path == node.path() {
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
                {
                    let mut file_reader = self.file_reader.lock().unwrap();
                    file_reader.seek(SeekFrom::Start(
                        compressed_data_offset + self.content.position,
                    ))?;
                    file_reader.read_exact(&mut blocks_infocompressedd_stream)?;
                }

                let mut blocks_info_uncompressedd_stream = block_uncompressed(
                    sb.uncompressed_size as u64,
                    sb.flags as u32,
                    blocks_infocompressedd_stream,
                );
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

    pub fn get_cab_path(&self) -> Option<String> {
        for file in self.get_files() {
            let path = file.path();
            if path.starts_with("CAB-") && (path.len() == 36) {
                return Some(path);
            }
        }
        None
    }

    pub fn get_cab(&self) -> std::io::Result<Vec<u8>> {
        if let Some(path) = self.get_cab_path() {
            return self.get_file_by_path(path);
        }
        Err(std::io::Error::from(ErrorKind::NotFound))
    }

    pub fn read(mut file: Box<dyn UnityResource + Send>, search_path: Option<String>) -> BinResult<UnityFS> {
        Ok(UnityFS {
            content: UnityFSFile::read(&mut file)?,
            file_reader: Arc::new(Mutex::new(file)),
            search_path,
        })
    }
}

impl FS for UnityFS {
    fn get_resource_file_by_path(
        &self,
        path: String,
        search_path: Option<&String>,
    ) -> Option<Box<dyn UnityResource>> {
        let s = ".".to_owned();
        let search_path = if let Some(p) = search_path {
            p
        } else if let Some(p) = &self.search_path {
            p
        } else {
            &s
        };

        if let Some(file_name) = PathBuf::from(&path)
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
        {
            if path.starts_with("archive:/") {
                if let Ok(file) = self.get_file_by_path(file_name) {
                    let file_reader = Cursor::new(file);
                    return Some(Box::new(file_reader));
                }
            } else {
                let path = PathBuf::from(search_path).join(file_name);
                if let Ok(file) = OpenOptions::new().read(true).open(path) {
                    return Some(Box::new(BufReader::new(file)));
                }
            }
        }

        None
    }
}

fn block_uncompressed(
    uncompressed_size: u64,
    flag: u32,
    blocks_infocompressedd_stream: Vec<u8>,
) -> Vec<u8> {
    let blocks_info_uncompressedd_stream;
    match CompressionType::try_from(flag & ArchiveFlags::CompressionTypeMask as u32) {
        Ok(tp) => match tp {
            CompressionType::None => {
                blocks_info_uncompressedd_stream = blocks_infocompressedd_stream
            }
            CompressionType::Lzma => todo!(),
            CompressionType::Lz4 | CompressionType::Lz4HC => {
                blocks_info_uncompressedd_stream = decompress(
                    &blocks_infocompressedd_stream,
                    Some(uncompressed_size as i32),
                )
                .unwrap();
            }
            CompressionType::Lzham => todo!(),
        },
        Err(_) => todo!(),
    }
    blocks_info_uncompressedd_stream
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
    flags: u16,
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
    flags: (u32, u32, u32, u32),
) -> BinResult<BlocksInfo> {
    let (version, compressed_blocks_info_size, uncompressed_blocks_info_size, flags) = flags;

    if (flags & ArchiveFlags::BlocksInfoAtTheEnd as u32) != 0 {
        todo!();
    }

    if version >= 7 {
        let pos = reader.seek(SeekFrom::Current(0))?;
        if pos % 16 != 0 {
            reader.seek(SeekFrom::Current((16 - (pos % 16)) as i64))?;
        }
    }

    let mut blocks_infocompressedd_stream = vec![0u8; compressed_blocks_info_size as usize];
    reader.read_exact(&mut blocks_infocompressedd_stream)?;

    if (flags & ArchiveFlags::BlockInfoNeedPaddingAtStart as u32) != 0 {
        let pos = reader.seek(SeekFrom::Current(0))?;
        if pos % 16 != 0 {
            reader.seek(SeekFrom::Current((16 - (pos % 16)) as i64))?;
        }
    }

    let blocks_info_uncompressedd_stream = block_uncompressed(
        uncompressed_blocks_info_size as u64,
        flags,
        blocks_infocompressedd_stream,
    );

    let mut blocks_info_reader = Cursor::new(blocks_info_uncompressedd_stream);
    BlocksInfo::read(&mut blocks_info_reader)
}
