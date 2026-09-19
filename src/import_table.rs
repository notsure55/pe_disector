use super::image::Image;
use crate::to_prim;
use anyhow::Result;
use windows_types::{IMAGE_IMPORT_BY_NAME, IMAGE_IMPORT_BY_ORDINAL, IMAGE_IMPORT_DESCRIPTOR};

const IMAGE_IMPORT_DIRECTORY_INDEX: usize = 1;

enum Import {
    Name(*mut IMAGE_IMPORT_BY_NAME),
    Ordinal(*mut IMAGE_IMPORT_BY_ORDINAL),
}

use std::fmt;

impl fmt::Debug for Import {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(ptr) => writeln!(f, "{:?}", unsafe { ptr.as_ref().unwrap() }),
            Self::Ordinal(ptr) => writeln!(f, "{:?}", unsafe { ptr.as_ref().unwrap() }),
        }
    }
}

#[derive(Debug)]
struct DllImport {
    name: String,
    raw: *mut IMAGE_IMPORT_DESCRIPTOR,
    raw_function_imports: Vec<Import>,
}

impl DllImport {
    pub fn from_raw_and_image(raw: *mut IMAGE_IMPORT_DESCRIPTOR, image: &Image) -> Result<Self> {
        let raw_ref = unsafe { raw.as_ref() }.unwrap();

        let name =
            image.read_cstr_from_fo(image.rva_to_fo(to_prim!(raw_ref.name => usize)).unwrap())?;

        println!("{name}");

        // TODO handle 32 bit array not just 64 bit array!
        let mut raw_function_imports = Vec::new();

        unsafe {
            let imports = std::slice::from_raw_parts_mut(
                image
                    .bytes_mut()
                    .as_mut_ptr()
                    .byte_offset(
                        to_prim!(image.rva_to_fo(to_prim!(raw_ref.original_first_thunk => usize)).unwrap() => isize),
                    )
                    .cast::<usize>(),
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
                        let name_table = image
                            .bytes_mut()
                            .as_mut_ptr()
                            .byte_offset(to_prim!(image.rva_to_fo(*import).unwrap() => isize))
                            .cast::<IMAGE_IMPORT_BY_NAME>();

                        raw_function_imports.push(Import::Name(name_table));
                    }
                }
            }
        }

        dbg!(&raw_function_imports);

        Ok(Self {
            name,
            raw,
            raw_function_imports,
        })
    }
}

pub struct ImportTable {
    import_descriptors: Vec<DllImport>,
}

impl ImportTable {
    pub fn from_image(image: &Image) -> Option<Self> {
        let import_directory = image
            .get_nt_header()
            .get_optional_header()
            .get_data_directory()[IMAGE_IMPORT_DIRECTORY_INDEX];

        let import_descriptors: Vec<_> = unsafe {
            std::slice::from_raw_parts_mut(
                image
                    .bytes_mut()
                    .as_mut_ptr()
                    .byte_offset(
                        to_prim!(image.rva_to_fo(to_prim!(import_directory.virtual_address => usize)).unwrap() => isize),
                    )
                    .cast::<IMAGE_IMPORT_DESCRIPTOR>(),
                to_prim!(import_directory.size => usize)
                    / std::mem::size_of::<IMAGE_IMPORT_DESCRIPTOR>() - 1,
            ).iter_mut().map(|desc| {
                DllImport::from_raw_and_image(desc as _, &image).unwrap()
            }).collect()
        };

        println!("{:#X?}", import_descriptors);

        None
    }
}
