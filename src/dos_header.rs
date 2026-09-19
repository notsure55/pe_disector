use crate::convert_unsafe_cell_bytes;
use std::ops::Deref;
use std::ops::DerefMut;
use windows_types::IMAGE_DOS_HEADER;

#[derive(Debug)]
pub struct DosHeader {
    raw: *mut IMAGE_DOS_HEADER,
}

impl DerefMut for DosHeader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut_unchecked() }
    }
}

impl Deref for DosHeader {
    type Target = IMAGE_DOS_HEADER;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref_unchecked() }
    }
}

impl DosHeader {
    pub fn from_bytes(bytes: *mut Vec<u8>) -> Self {
        let raw = convert_unsafe_cell_bytes!(bytes => IMAGE_DOS_HEADER);

        Self { raw }
    }
}
