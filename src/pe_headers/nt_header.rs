use crate::arch;
use crate::windows_types::*;

use super::file_header::*;
use super::optional_header::*;
use std::fmt;

pub trait NtHeader: fmt::Debug {
    fn file_header(&self) -> &FileHeader;
    fn optional_header(&self) -> &dyn OptionalHeader;
    fn size(&self) -> usize;
    fn is_optional_invalid_size(&self) -> bool {
        let file_header = self.file_header();

        match self.file_header().arch() {
            arch::Architecture::X32 => {
                usize::from(file_header.size_of_optional_header())
                    != std::mem::size_of::<IMAGE_OPTIONAL_HEADER32>()
            }
            arch::Architecture::X64 => {
                usize::from(file_header.size_of_optional_header())
                    != std::mem::size_of::<IMAGE_OPTIONAL_HEADER64>()
            }
        }
    }
}

#[derive(Debug)]
pub struct NtHeaders32 {
    raw: IMAGE_NT_HEADERS32,
    file_header: FileHeader,
    optional_header: OptionalHeader32,
}

impl NtHeaders32 {
    pub fn new(raw: IMAGE_NT_HEADERS32) -> Self {
        let file_header = FileHeader::new(raw.file_header);
        let optional_header = OptionalHeader32::new(raw.optional_header);

        Self {
            raw,
            file_header,
            optional_header,
        }
    }
}

impl NtHeader for NtHeaders32 {
    fn file_header(&self) -> &FileHeader {
        &self.file_header
    }
    fn optional_header(&self) -> &dyn OptionalHeader {
        &self.optional_header
    }
    fn size(&self) -> usize {
        std::mem::size_of::<IMAGE_NT_HEADERS32>()
    }
}

#[derive(Debug)]
pub struct NtHeaders64 {
    raw: IMAGE_NT_HEADERS64,
    file_header: FileHeader,
    optional_header: OptionalHeader64,
}

impl NtHeaders64 {
    pub fn new(raw: IMAGE_NT_HEADERS64) -> Self {
        let file_header = FileHeader::new(raw.file_header);
        let optional_header = OptionalHeader64::new(raw.optional_header);

        Self {
            raw,
            file_header,
            optional_header,
        }
    }
}

impl NtHeader for NtHeaders64 {
    fn file_header(&self) -> &FileHeader {
        &self.file_header
    }
    fn optional_header(&self) -> &dyn OptionalHeader {
        &self.optional_header
    }
    fn size(&self) -> usize {
        std::mem::size_of::<IMAGE_NT_HEADERS64>()
    }
}
