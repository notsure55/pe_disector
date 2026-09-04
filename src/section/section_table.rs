use anyhow::Result;
use std::ffi::CStr;
use std::fmt;
use std::ptr;
use std::vec::Vec;

use super::section_header::*;
use crate::windows_types::*;
use crate::*;

pub struct SectionTableOwned {
    sections: Vec<SectionHeaderOwned>,
}

impl SectionTable for SectionTableOwned {
    fn section_headers(&self) -> Box<dyn Iterator<Item = &dyn SectionHeader> + '_> {
        Box::new(self.sections.iter().map(|sec| sec as &dyn SectionHeader))
    }
}

impl SectionTableOwned {
    pub fn from_bytes(bytes: &[u8], section_count: usize, start: usize) -> Result<Self> {
        // parses section headers
        let sections: Result<Vec<_>> = unsafe {
            &*ptr::slice_from_raw_parts(
                bytes
                    .as_ptr()
                    .byte_offset(start.try_into()?)
                    .cast::<IMAGE_SECTION_HEADER>(),
                section_count,
            )
        }
        .iter()
        .map(|raw_header| SectionHeaderOwned::from_binary(raw_header.clone(), bytes, start))
        .collect();

        Ok(Self {
            sections: sections?,
        })
    }
    pub fn from_sections(sections: Vec<SectionHeaderOwned>) -> Self {
        Self { sections }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut section_header_bytes =
            Vec::with_capacity(std::mem::size_of::<IMAGE_SECTION_HEADER>() * self.sections.len());

        let mut section_bytes = Vec::new();

        for section in self.sections.iter() {
            let header_bytes = to_bytes!(*section.raw());

            section_header_bytes.extend_from_slice(header_bytes);
            section_bytes.extend_from_slice(section.bytes());
        }

        section_header_bytes.append(&mut section_bytes);

        section_header_bytes
    }
}

pub struct SectionTableRef<'a> {
    sections: Vec<SectionHeaderRef<'a>>,
}

impl<'a> SectionTable for SectionTableRef<'a> {
    fn section_headers(&self) -> Box<dyn Iterator<Item = &dyn SectionHeader> + '_> {
        Box::new(self.sections.iter().map(|sec| sec as &dyn SectionHeader))
    }
}

impl<'a> SectionTableRef<'a> {
    pub fn from_bytes(bytes: &'a [u8], section_count: usize, start: usize) -> Result<Self> {
        // parses section headers
        let sections: Result<Vec<_>> = unsafe {
            &*ptr::slice_from_raw_parts(
                bytes
                    .as_ptr()
                    .byte_offset(start.try_into()?)
                    .cast::<IMAGE_SECTION_HEADER>(),
                section_count,
            )
        }
        .iter()
        .map(|raw_header| SectionHeaderRef::from_binary(raw_header.clone(), bytes))
        .collect();

        Ok(Self {
            sections: sections?,
        })
    }
    pub fn from_sections(sections: Vec<SectionHeaderRef<'a>>) -> Self {
        Self { sections }
    }
    pub fn to_owned(self) -> SectionTableOwned {
        SectionTableOwned::from_sections(
            self.sections
                .into_iter()
                .map(|s| s.to_owned())
                .collect::<Vec<SectionHeaderOwned>>(),
        )
    }
}

pub trait SectionTable {
    fn section_headers(&self) -> Box<dyn Iterator<Item = &dyn SectionHeader> + '_>;
}

impl fmt::Debug for dyn SectionTable + '_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for sec_header in self.section_headers() {
            writeln!(f, "Section {}", sec_header.name());
            writeln!(f, "Header {:#?}", sec_header.raw());
        }

        Ok(())
    }
}
