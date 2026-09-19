use crate::to_prim;
use std::ops::Deref;
use std::ops::DerefMut;
use windows_types::{IMAGE_NT_HEADERS, IMAGE_NT_HEADERS64};

#[derive(Debug)]
pub struct NtHeader {
    raw: *mut dyn IMAGE_NT_HEADERS,
}

impl DerefMut for NtHeader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut_unchecked() }
    }
}

impl Deref for NtHeader {
    type Target = dyn IMAGE_NT_HEADERS;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref_unchecked() }
    }
}

impl NtHeader {
    pub fn from_bytes(bytes: *mut Vec<u8>, e_lfanew: u32) -> Self {
        let raw = unsafe {
            (&mut *bytes)
                .as_mut_ptr()
                .byte_offset(to_prim!(e_lfanew as isize))
                .cast::<IMAGE_NT_HEADERS64>()
        };

        let machine = unsafe { raw.as_ref_unchecked().get_file_header().machine };

        if machine != 0x8664 {
            todo!("Implement support for the other architectures!");
        }

        Self { raw }
    }
}
