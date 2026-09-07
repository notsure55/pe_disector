use crate::windows_types::*;

use crate::pe::Va;

use std::fmt;

pub trait OptionalHeader: fmt::Debug {
    fn magic(&self) -> u16;
    fn image_base(&self) -> Va;
    fn set_image_base(&mut self, new_image_base: Va);
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
    fn image_base(&self) -> Va {
        Va::try_from(self.0.image_base).expect("Failed to turn image_base from u32 into va?")
    }
    fn set_image_base(&mut self, new_image_base: Va) {
        self.0.image_base =
            u32::try_from(*new_image_base).expect("Failed to turn image_base from va to u32?");
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
    fn image_base(&self) -> Va {
        Va::try_from(self.0.image_base).expect("Failed to turn image_base from u64 into va?")
    }
    fn set_image_base(&mut self, new_image_base: Va) {
        self.0.image_base =
            u64::try_from(*new_image_base).expect("Failed to turn image_base from va to u64?");
    }
    fn data_directory(&self) -> &[IMAGE_DATA_DIRECTORY; 16] {
        &self.0.data_directory
    }
}
