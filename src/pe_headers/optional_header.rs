use crate::windows_types::*;

use std::fmt;

pub trait OptionalHeader: fmt::Debug {
    fn magic(&self) -> u16;
    fn image_base(&self) -> usize;
    fn is_amd_intel(&self) -> bool {
        let magic = self.magic();

        if magic != 0x20b && magic != 0x10b {
            false
        } else {
            true
        }
    }
    fn data_directory(&self) -> &[IMAGE_DATA_DIRECTORY; 16];
}

#[derive(Debug)]
pub struct OptionalHeader32(IMAGE_OPTIONAL_HEADER32);

impl OptionalHeader32 {
    pub fn new(raw: IMAGE_OPTIONAL_HEADER32) -> Self {
        Self { 0: raw }
    }
}

impl OptionalHeader for OptionalHeader32 {
    fn magic(&self) -> u16 {
        self.0.magic
    }
    fn image_base(&self) -> usize {
        usize::try_from(self.0.image_base).expect("Failed to turn image_base from u32 into usize?")
    }
    fn data_directory(&self) -> &[IMAGE_DATA_DIRECTORY; 16] {
        &self.0.data_directory
    }
}

#[derive(Debug)]
pub struct OptionalHeader64(IMAGE_OPTIONAL_HEADER64);

impl OptionalHeader64 {
    pub fn new(raw: IMAGE_OPTIONAL_HEADER64) -> Self {
        Self { 0: raw }
    }
}

impl OptionalHeader for OptionalHeader64 {
    fn magic(&self) -> u16 {
        self.0.magic
    }
    fn image_base(&self) -> usize {
        usize::try_from(self.0.image_base).expect("Failed to turn image_base from u32 into usize?")
    }
    fn data_directory(&self) -> &[IMAGE_DATA_DIRECTORY; 16] {
        &self.0.data_directory
    }
}
