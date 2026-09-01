use super::pe::*;
use super::windows_types::*;
use anyhow::{anyhow, Result};

const EXCEPTION_DIRECTORY_TABLE_INDEX: usize = 3;
const UNWIND_CODE_SIZE: usize = 2;

#[derive(Debug)]
struct UnwindInfo {
    raw: UNWIND_INFO,
    unwind_codes: Vec<u16>,
}

impl UnwindInfo {
    pub fn from_image<I: Image>(unwind_info_address: Rva, image: &I) -> Result<Self> {
        let raw_unwind_info = image.read_from_rva::<UNWIND_INFO>(unwind_info_address)?;

        let code_count = usize::from(raw_unwind_info.count_of_unwind_codes);

        if code_count <= 0 {
            return Ok(Self {
                raw: raw_unwind_info,
                unwind_codes: vec![],
            });
        }

        let unwind_codes_raw = image.read_from_rva_to_buf(
            unwind_info_address + std::mem::size_of::<UNWIND_INFO>(),
            code_count * UNWIND_CODE_SIZE,
        )?;

        let unwind_codes = unsafe {
            std::slice::from_raw_parts(unwind_codes_raw.as_ptr() as *const u16, code_count).to_vec()
        };

        Ok(Self {
            raw: raw_unwind_info,
            unwind_codes,
        })
    }
}

#[derive(Debug)]
struct ExceptionTableEntry {
    func_entry: IMAGE_FUNCTION_ENTRY64,
    unwind_info: UnwindInfo,
}

impl ExceptionTableEntry {
    pub fn from_image<I: Image>(func_entry: IMAGE_FUNCTION_ENTRY64, image: &I) -> Result<Self> {
        let unwind_info =
            UnwindInfo::from_image(Rva::try_from(func_entry.unwind_info_address)?, image)?;

        Ok(Self {
            func_entry,
            unwind_info,
        })
    }
}

#[derive(Debug)]
pub struct ExceptionTable {
    entrys: Vec<ExceptionTableEntry>,
}

impl ExceptionTable {
    pub fn from_image<I: Image>(image: &I) -> Result<Self> {
        let exception_directory_image = image
            .headers()
            .nt_header()
            .optional_header()
            .data_directory()[EXCEPTION_DIRECTORY_TABLE_INDEX];

        if exception_directory_image.virtual_address == 0 && exception_directory_image.size == 0 {
            return Err(anyhow!("exception not availabe? check pe in 010"));
        }

        let exception_table_entrys_bytes = image.read_from_rva_to_buf(
            Rva::try_from(exception_directory_image.virtual_address)?,
            exception_directory_image.size.try_into()?,
        )?;

        let exception_table_raw_entrys = unsafe {
            std::slice::from_raw_parts(
                exception_table_entrys_bytes.as_ptr() as *const IMAGE_FUNCTION_ENTRY64,
                usize::try_from(exception_directory_image.size)?
                    / std::mem::size_of::<IMAGE_FUNCTION_ENTRY64>(),
            )
        };

        let exception_table_entrys: Vec<_> = exception_table_raw_entrys
            .iter()
            .map(|entry| {
                ExceptionTableEntry::from_image(*entry, image)
                    .map_err(|err| {
                        panic!("{err}");
                    })
                    .unwrap()
            })
            .collect();

        Ok(Self {
            entrys: exception_table_entrys,
        })
    }
}
