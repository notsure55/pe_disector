use std::boxed::Box;

use super::windows_types::*;

pub mod file_header;
pub mod nt_header;
pub mod optional_header;

use super::arch;
use nt_header::*;

#[derive(Debug)]
pub struct PeHeaders {
    dos_header: IMAGE_DOS_HEADER,
    nt_header: Box<dyn NtHeader>,
}

impl PeHeaders {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        unsafe {
            let dos_header = bytes.as_ptr().cast::<IMAGE_DOS_HEADER>().read();

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
}
