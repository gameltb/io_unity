pub mod binrw_parser;

use std::error::Error;

use binrw::binrw;
use num_enum::TryFromPrimitive;
use regex::Regex;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct UnityVersion {
    version: Vec<u32>,
    build_type: Option<String>,
}

impl UnityVersion {
    pub fn new(version: Vec<u32>, build_type: Option<String>) -> Self {
        UnityVersion {
            version,
            build_type,
        }
    }

    pub fn from_str(version: &str) -> Result<Self, Box<dyn Error>> {
        lazy_static! {
            static ref BUILD_TYPE_REGEX: Regex = Regex::new(r"([^\d.])").unwrap();
            static ref VERSION_REGEX: Regex = Regex::new(r"\D").unwrap();
        };

        Ok(UnityVersion {
            version: VERSION_REGEX
                .split(version)
                .map(|d| d.parse::<u32>().unwrap())
                .collect(),
            build_type: BUILD_TYPE_REGEX
                .captures(version)
                .and_then(|c| c.get(0))
                .and_then(|m| Some(m.as_str().to_owned())),
        })
    }

    pub fn is_alpha(&self) -> bool {
        self.build_type == Some("a".to_string())
    }

    pub fn is_patch(&self) -> bool {
        self.build_type == Some("p".to_string())
    }
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone)]
#[repr(u8)]
pub enum Endian {
    Little,
    Big,
}
