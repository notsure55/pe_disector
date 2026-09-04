use std::boxed::Box;

use super::windows_types::*;

use crate::to_bytes;

pub mod file_header;
pub mod nt_header;
pub mod optional_header;

use super::arch;
use nt_header::*;

#[repr(C)]
#[derive(Debug)]
struct IMAGE_DOS_STUB {
    reserved: [u8; 0xA0],
}

#[derive(Debug)]
pub struct PeHeaders {
    dos_header: IMAGE_DOS_HEADER,
    dos_stub: IMAGE_DOS_STUB,
    nt_header: Box<dyn NtHeader>,
}

impl PeHeaders {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        unsafe {
            let dos_header = bytes.as_ptr().cast::<IMAGE_DOS_HEADER>().read_unaligned();

            let dos_stub = bytes
                .as_ptr()
                .offset(std::mem::size_of::<IMAGE_DOS_HEADER>().try_into().unwrap())
                .cast::<IMAGE_DOS_STUB>()
                .read_unaligned();

            let nt_header: Box<dyn NtHeader> = {
                let nt_header_ptr = bytes.as_ptr().byte_offset(dos_header.e_lfanew as isize);
                let header = nt_header_ptr.cast::<IMAGE_NT_HEADERS64>().read();

                match arch::Architecture::try_from(header.file_header.machine).unwrap() {
                    arch::Architecture::X32 => Box::new(NtHeaders32::new(
                        nt_header_ptr.cast::<IMAGE_NT_HEADERS32>().read(),
                    )),
                    arch::Architecture::X64 => Box::new(NtHeaders64::new(header)),
                }
            };

            // TODO make this good?
            if nt_header.is_optional_invalid_size() {
                panic!("INVALID SIZE OF OPTIONAL HEADER WTF!?")
            }

            if !nt_header.optional_header().is_amd_intel() {
                panic!("NOT PE64 or PE32 NOT HANDLED YET")
            }

            Self {
                dos_header,
                dos_stub,
                nt_header,
            }
        }
    }
    pub fn dos_header(&self) -> &IMAGE_DOS_HEADER {
        &self.dos_header
    }
    pub fn nt_header(&self) -> &dyn NtHeader {
        self.nt_header.as_ref()
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut dos_header_bytes = to_bytes!(self.dos_header).to_vec();
        dos_header_bytes.extend_from_slice(&self.dos_stub.reserved);
        dos_header_bytes.extend_from_slice(self.nt_header().to_bytes());
        dos_header_bytes
    }
}
