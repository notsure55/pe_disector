use anyhow::Result;
use std::ffi::CStr;
use std::fmt;
use std::ptr;
use std::vec::Vec;

use crate::*;

use crate::windows_types::*;

pub struct SectionHeaderOwned {
    name: String,
    raw: IMAGE_SECTION_HEADER,
    section_bytes: Vec<u8>,
}

impl SectionHeaderOwned {
    pub fn from_binary(raw_header: IMAGE_SECTION_HEADER, binary_bytes: &[u8]) -> Result<Self> {
        let bytes = binary_bytes
            .get(
                to_usize!(raw_header.pointer_to_raw_data)
                    ..to_usize!(raw_header.pointer_to_raw_data)
                        + to_usize!(raw_header.size_of_raw_data),
            )
            .unwrap()
            .to_vec();

        let name: String = c_str!(&raw_header.name);

        Ok(Self::new(name, raw_header, bytes))
    }
    pub fn new(
        name: impl ToString,
        raw_header: IMAGE_SECTION_HEADER,
        section_bytes: Vec<u8>,
    ) -> Self {
        Self {
            name: name.to_string(),
            raw: raw_header,
            section_bytes,
        }
    }
}

impl SectionHeader for SectionHeaderOwned {
    fn raw(&self) -> &IMAGE_SECTION_HEADER {
        &self.raw
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn bytes(&self) -> &[u8] {
        &self.section_bytes
    }
}

pub struct SectionHeaderRef<'a> {
    name: String,
    raw: IMAGE_SECTION_HEADER,
    section_bytes: &'a [u8],
}

impl<'a> SectionHeaderRef<'a> {
    pub fn from_binary(raw_header: IMAGE_SECTION_HEADER, binary_bytes: &'a [u8]) -> Result<Self> {
        let bytes = binary_bytes
            .get(
                to_usize!(raw_header.pointer_to_raw_data)
                    ..to_usize!(raw_header.pointer_to_raw_data)
                        + to_usize!(raw_header.size_of_raw_data),
            )
            .unwrap();

        let name: String = c_str!(&raw_header.name);

        Ok(Self::new(name, raw_header, bytes))
    }
    pub fn new(
        name: impl ToString,
        raw_header: IMAGE_SECTION_HEADER,
        section_bytes: &'a [u8],
    ) -> Self {
        Self {
            name: name.to_string(),
            raw: raw_header,
            section_bytes,
        }
    }
    pub fn to_owned(self) -> SectionHeaderOwned {
        SectionHeaderOwned::new(self.name, self.raw, self.section_bytes.to_vec())
    }
}

impl<'a> SectionHeader for SectionHeaderRef<'a> {
    fn raw(&self) -> &IMAGE_SECTION_HEADER {
        &self.raw
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn bytes(&self) -> &[u8] {
        &self.section_bytes
    }
}

pub trait SectionHeader {
    fn raw(&self) -> &IMAGE_SECTION_HEADER;
    fn name(&self) -> &str;
    fn bytes(&self) -> &[u8];
}

impl fmt::Debug for dyn SectionHeader + '_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "Section {}", self.name());
        writeln!(f, "Header {:#?}", self.raw());

        Ok(())
    }
}
