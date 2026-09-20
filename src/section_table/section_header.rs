use crate::to_prim;
use std::ops::Deref;
use std::ops::DerefMut;
use windows_types::IMAGE_SECTION_HEADER;

#[derive(Debug)]
pub struct SectionHeader {
    raw: *mut IMAGE_SECTION_HEADER,
    section_bytes: *mut [u8],
}

impl DerefMut for SectionHeader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut_unchecked() }
    }
}

impl Deref for SectionHeader {
    type Target = IMAGE_SECTION_HEADER;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref_unchecked() }
    }
}

impl SectionHeader {
    pub fn from_bytes(raw: *mut IMAGE_SECTION_HEADER, bytes: &mut [u8]) -> Self {
        unsafe {
            let pointer_to_raw_data = to_prim!((&*raw).pointer_to_raw_data => usize);
            let size_of_raw_data = to_prim!((&*raw).size_of_raw_data => usize);

            let section_bytes = bytes
                .get_mut(pointer_to_raw_data..pointer_to_raw_data + size_of_raw_data)
                .unwrap() as *mut [u8];

            Self { raw, section_bytes }
        }
    }
    pub fn get_bytes(&self) -> &[u8] {
        unsafe { self.section_bytes.as_ref_unchecked() }
    }
    pub fn get_bytes_mut(&self) -> &mut [u8] {
        unsafe { self.section_bytes.as_mut_unchecked() }
    }
}
