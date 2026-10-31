use crate::image::{Image, Va};
use crate::{convert_mut_slice_to_ptr, to_prim};
use anyhow::Result;
use std::ops::Deref;
use std::ops::DerefMut;
use windows_types::user::{IMAGE_IMPORT_BY_NAME, IMAGE_IMPORT_BY_ORDINAL, IMAGE_IMPORT_DESCRIPTOR};

use super::Import;

#[derive(Debug)]
pub struct DllImport {
    raw: *mut IMAGE_IMPORT_DESCRIPTOR,
    // used for modifying symbols at runtime
    iat: Vec<*mut Va>,
    // must remain unordered so cant use hash or btreemap as the index of it matters
    pub raw_function_imports: Vec<Import>,
}

impl DerefMut for DllImport {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut_unchecked() }
    }
}

impl Deref for DllImport {
    type Target = IMAGE_IMPORT_DESCRIPTOR;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref_unchecked() }
    }
}

impl DllImport {
    pub fn from_raw_and_image(raw: *mut IMAGE_IMPORT_DESCRIPTOR, image: &Image) -> Option<Self> {
        let raw_ref = unsafe { raw.as_ref() }.unwrap();

        // TODO handle 32 bit array not just 64 bit array!
        let mut raw_function_imports = Vec::new();

        unsafe {
            let address = image.rva_to_fo(to_prim!(raw_ref.original_first_thunk => usize))?;
            let ptr = convert_mut_slice_to_ptr!(image.bytes_mut() => usize, address);

            for i in 0..0x1000 {
                let import = ptr.offset(i);

                if import.read_unaligned() == 0 {
                    break;
                } else {
                    if import.read_unaligned() & 0x8000000000000000 == 1 {
                        raw_function_imports
                            .push(Import::Ordinal(import.cast::<IMAGE_IMPORT_BY_ORDINAL>()));
                    } else {
                        if let Some(address) = image.rva_to_fo(import.read_unaligned()) {
                            let name_table = convert_mut_slice_to_ptr!(image.bytes_mut() => IMAGE_IMPORT_BY_NAME, address);

                            raw_function_imports.push(Import::Name(name_table));
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        let iat = unsafe {
            let ptr = convert_mut_slice_to_ptr!(image.bytes_mut() => Va, image.rva_to_fo(to_prim!(raw_ref.first_thunk => usize)).unwrap());

            let mut iat = Vec::with_capacity(raw_function_imports.len());

            for i in 0..raw_function_imports.len() {
                iat.push(ptr.offset(i as isize));
            }

            iat
        };

        if iat.is_empty() || raw_function_imports.is_empty() {
            None
        } else {
            Some(Self {
                raw,
                iat,
                raw_function_imports,
            })
        }
    }
    pub fn iat_mut(&self) -> &mut [*mut Va] {
        unsafe {
            let mut_slice: &mut [*mut Va] =
                std::mem::transmute(self.iat.as_slice() as *const [*mut Va]);
            mut_slice
        }
    }
}
