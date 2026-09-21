use crate::image::{Fo, Image};
use crate::{convert_mut_slice_to_ptr, to_prim};
use anyhow::Result;
use ref_mut_field::RefMutFields;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use windows_types::IMAGE_EXPORT_DIRECTORY;

const IMAGE_EXPORT_DIRECTORY_INDEX: usize = 0;

#[derive(Debug, RefMutFields)]
pub struct Export {
    #[ref_]
    function: Fo,
    #[ref_]
    ordinal: u16,
}

#[derive(Debug, RefMutFields)]
pub struct ExportTable {
    raw: *mut IMAGE_EXPORT_DIRECTORY,
    #[ref_]
    exports: HashMap<String, Export>,
}

impl DerefMut for ExportTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut_unchecked() }
    }
}

impl Deref for ExportTable {
    type Target = IMAGE_EXPORT_DIRECTORY;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref_unchecked() }
    }
}

impl ExportTable {
    pub fn from_image(image: &Image) -> Result<Option<Self>> {
        let export_directory = image
            .get_nt_header()
            .get_optional_header()
            .get_data_directory()[IMAGE_EXPORT_DIRECTORY_INDEX];

        if export_directory.virtual_address == 0 || export_directory.size == 0 {
            return Ok(None);
        }

        let export_directory = convert_mut_slice_to_ptr!(image.bytes_mut() => IMAGE_EXPORT_DIRECTORY,
                                                         image.rva_to_fo(to_prim!(export_directory.virtual_address => usize)).unwrap());
        let export_directory_ref = unsafe { export_directory.as_ref().unwrap() };

        let addresses_of_functions = unsafe {
            std::slice::from_raw_parts_mut(
                convert_mut_slice_to_ptr!(image.bytes_mut() => u32,
                                      image.rva_to_fo(to_prim!(export_directory_ref.address_of_functions => usize)).unwrap()),
                to_prim!(export_directory_ref.number_of_functions => usize),
            )
        }.into_iter();

        let addresses_of_names = unsafe {
            std::slice::from_raw_parts_mut(
                convert_mut_slice_to_ptr!(image.bytes_mut() => u32,
                                      image.rva_to_fo(to_prim!(export_directory_ref.address_of_names => usize)).unwrap()),
                to_prim!(export_directory_ref.number_of_names => usize),
            )
        }.into_iter();

        let addresses_of_ordinals = unsafe {
            std::slice::from_raw_parts_mut(
                convert_mut_slice_to_ptr!(image.bytes_mut() => u16,
                                      image.rva_to_fo(to_prim!(export_directory_ref.address_of_name_ordinals => usize)).unwrap()),
                to_prim!(export_directory_ref.number_of_names => usize),
            )
        }.into_iter();

        let exports: HashMap<_, _> = addresses_of_functions
            .zip(addresses_of_names)
            .zip(addresses_of_ordinals)
            .into_iter()
            .map(|((function, name), ordinal)| {
                let name = image
                    .read_cstr_from_fo(image.rva_to_fo(to_prim!(*name => usize)).unwrap())
                    .unwrap();

                let function_addr_fo = image.rva_to_fo(to_prim!(*function => usize)).unwrap();

                (
                    name,
                    Export {
                        function: function_addr_fo,
                        ordinal: *ordinal,
                    },
                )
            })
            .collect();

        Ok(Some(Self {
            raw: export_directory,
            exports,
        }))
    }
}
