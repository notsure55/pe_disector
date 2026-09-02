use super::pe::*;
use super::windows_types::*;
use crate::{c_str, to_usize};
use anyhow::Result;

const EXPORT_DIRECTORY_TABLE_INDEX: usize = 0;

#[derive(Debug)]
pub struct ExportTable {
    raw: IMAGE_EXPORT_DIRECTORY,
    entrys: Vec<ExportAddressEntry>,
}

impl ExportTable {
    pub fn from_image(image: &impl Image) -> Result<Option<Self>> {
        let export_directory_image = image
            .headers()
            .nt_header()
            .optional_header()
            .data_directory()[EXPORT_DIRECTORY_TABLE_INDEX];

        if export_directory_image.virtual_address == 0 || export_directory_image.size == 0 {
            return Ok(None);
        }

        let export_directory = image.read_from_rva::<IMAGE_EXPORT_DIRECTORY>(Rva::try_from(
            export_directory_image.virtual_address,
        )?)?;

        let mut export_address_functions = image
            .read_slice_from_rva::<u32>(
                Rva::try_from(export_directory.address_of_functions)?,
                to_usize!(export_directory.number_of_functions),
            )?
            .into_iter()
            .map(|r| -> Rva { Rva::try_from(*r).unwrap() })
            .collect::<Vec<Rva>>()
            .into_iter();

        let mut export_address_names = image
            .read_slice_from_rva::<u32>(
                Rva::try_from(export_directory.address_of_names)?,
                to_usize!(export_directory.number_of_names),
            )?
            .into_iter()
            .map(|r| -> Option<String> {
                let rva = Rva::try_from(*r).ok()?;
                image.read_c_str_from_rva(rva).ok()
            })
            .collect::<Vec<Option<String>>>()
            .into_iter();

        let mut export_ordinals = image
            .read_slice_from_rva::<u16>(
                Rva::try_from(export_directory.address_of_name_ordinals)?,
                to_usize!(export_directory.number_of_functions),
            )?
            .into_iter();

        let number = to_usize!(export_directory.number_of_functions);

        let mut address_entrys = Vec::with_capacity(number);

        for _ in 0..number {
            if let (Some(function_rva), Some(name), Some(ordinal)) = (
                export_address_functions.next(),
                export_address_names.next(),
                export_ordinals.next(),
            ) {
                address_entrys.push(ExportAddressEntry::new(
                    function_rva,
                    name,
                    to_usize!(*ordinal),
                ));
            }
        }

        Ok(Some(Self {
            raw: export_directory,
            entrys: address_entrys,
        }))
    }
}

#[derive(Debug)]
pub struct ExportAddressEntry {
    export_rva: Rva,
    name: Option<String>,
    ordinal: usize,
}

impl ExportAddressEntry {
    pub fn new(export_rva: Rva, name: Option<String>, ordinal: usize) -> Self {
        Self {
            export_rva,
            name,
            ordinal,
        }
    }
}
