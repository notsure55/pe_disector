use super::dll_import::DllImport;
use crate::image::{Image, Va};
use crate::{convert_mut_slice_to_ptr, to_prim};
use std::collections::BTreeMap;
use windows_types::IMAGE_IMPORT_DESCRIPTOR;

use anyhow::{anyhow, Result};

use super::ImportSimple;

const IMAGE_IMPORT_DIRECTORY_INDEX: usize = 1;

pub struct ImportTable {
    import_descriptors: BTreeMap<String, DllImport>,
}

// TODO add ability to modify memory of https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#the-idata-section
// when the image is loaded into memory the loader, resolves all virtual addresses for the symbols and loads them into the import
// address table, make a good algorithm to do this also, wil be used for manual mapping, ideas for now.
// - Use EnumProcessModules, on the handle to the process, to get a list of all modules, then manually enumerate all functions
// get there virtual address of the symbol then write that into memory
impl ImportTable {
    pub fn from_image(image: &Image) -> Option<Self> {
        let import_directory = image
            .get_nt_header()
            .get_optional_header()
            .get_data_directory()[IMAGE_IMPORT_DIRECTORY_INDEX];

        if import_directory.virtual_address == 0 || import_directory.size == 0 {
            return None;
        }

        let import_descriptors: BTreeMap<String, DllImport> = unsafe {
            std::slice::from_raw_parts_mut(
                convert_mut_slice_to_ptr!(image.bytes_mut() => IMAGE_IMPORT_DESCRIPTOR,
                                          image.rva_to_fo(to_prim!(import_directory.virtual_address => usize)).unwrap()),
                to_prim!(import_directory.size => usize)
                    / std::mem::size_of::<IMAGE_IMPORT_DESCRIPTOR>() - 1,
            ).iter_mut().map(|desc| {
                let dll_import = DllImport::from_raw_and_image(desc as _, &image).unwrap();
                let name = image.read_cstr_from_fo(image.rva_to_fo(to_prim!(dll_import.name => usize)).unwrap()).unwrap();
                (name, dll_import)
            }).collect()
        };

        println!("{:#X?}", import_descriptors);

        Some(Self { import_descriptors })
    }
    pub fn update_iat(
        &mut self,
        dll_name: impl AsRef<str>,
        function_ident: impl Into<ImportSimple>,
        function_va: Va,
    ) -> Result<()> {
        let function_ident = function_ident.into();

        if let Some(dll_import) = self.import_descriptors.get_mut(dll_name.as_ref()) {
            let import_index =
                dll_import
                    .raw_function_imports
                    .iter()
                    .enumerate()
                    .find_map(|(i, import)| {
                        if function_ident == import {
                            Some(i)
                        } else {
                            None
                        }
                    });

            if let Some(index) = import_index {
                println!("Found {index:?} index for {function_ident:#X?}");

                let iat = dll_import.iat_mut();

                iat[index] = function_va;
            } else {
                return Err(anyhow!(
                    "Failed to find function index for {function_ident:?} in {}",
                    dll_name.as_ref()
                ));
            }
        } else {
            return Err(anyhow!("Failed to {}", dll_name.as_ref()));
        }

        Ok(())
    }
}
