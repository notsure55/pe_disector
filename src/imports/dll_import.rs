use crate::image::{Image, Va};
use crate::{convert_mut_slice_to_ptr, to_prim};
use anyhow::Result;
use std::ops::Deref;
use std::ops::DerefMut;
use windows_types::{IMAGE_IMPORT_BY_NAME, IMAGE_IMPORT_BY_ORDINAL, IMAGE_IMPORT_DESCRIPTOR};

use super::Import;

#[derive(Debug)]
pub struct DllImport {
    raw: *mut IMAGE_IMPORT_DESCRIPTOR,
    // used for modifying symbols at runtime
    iat: *mut [Va],
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
    pub fn from_raw_and_image(raw: *mut IMAGE_IMPORT_DESCRIPTOR, image: &Image) -> Result<Self> {
        let raw_ref = unsafe { raw.as_ref() }.unwrap();

        // TODO handle 32 bit array not just 64 bit array!
        let mut raw_function_imports = Vec::new();

        unsafe {
            let imports = std::slice::from_raw_parts_mut(
                convert_mut_slice_to_ptr!(image.bytes_mut() => usize,
                                          image.rva_to_fo(to_prim!(raw_ref.original_first_thunk => usize)).unwrap()),
                0x1000,
            );

            for import in imports.iter_mut() {
                if *import == 0 {
                    break;
                } else {
                    if *import & 0x8000000000000000 == 1 {
                        raw_function_imports.push(Import::Ordinal(
                            (import as *mut usize).cast::<IMAGE_IMPORT_BY_ORDINAL>(),
                        ));
                    } else {
                        let name_table = convert_mut_slice_to_ptr!(image.bytes_mut() => IMAGE_IMPORT_BY_NAME,
                                                                   image.rva_to_fo(*import).unwrap());

                        raw_function_imports.push(Import::Name(name_table));
                    }
                }
            }
        }

        let iat = unsafe {
            std::slice::from_raw_parts_mut(
                convert_mut_slice_to_ptr!(image.bytes_mut() => Va, image.rva_to_fo(to_prim!(raw_ref.first_thunk => usize)).unwrap()),
                raw_function_imports.len(),
            )
        };

        Ok(Self {
            raw,
            iat,
            raw_function_imports,
        })
    }
    pub fn iat_mut(&self) -> &mut [Va] {
        unsafe { self.iat.as_mut().unwrap() }
    }
}
