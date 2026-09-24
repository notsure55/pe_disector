use crate::convert_mut_slice_to_ptr;
use std::ops::Deref;
use std::ops::DerefMut;
use windows_types::{IMAGE_NT_HEADERS, IMAGE_NT_HEADERS64};

#[derive(Debug)]
pub struct NtHeader {
    raw: *mut dyn IMAGE_NT_HEADERS,
}

impl DerefMut for NtHeader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut().unwrap() }
    }
}

impl Deref for NtHeader {
    type Target = dyn IMAGE_NT_HEADERS;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref().unwrap() }
    }
}

impl NtHeader {
    pub fn from_bytes(bytes: &mut [u8], e_lfanew: u32) -> Self {
        let raw = convert_mut_slice_to_ptr!(bytes => IMAGE_NT_HEADERS64, e_lfanew);

        let machine = unsafe { raw.as_ref().unwrap().get_file_header().machine };

        if machine != 0x8664 {
            todo!("Implement support for the other architectures!");
        }

        Self { raw }
    }
}
