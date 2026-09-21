use crate::image::Image;
use crate::{convert_mut_slice_to_ptr, to_prim};
use anyhow::Result;
use ref_mut_field::RefMutFields;
use std::fmt;
use windows_types::IMAGE_FUNCTION_ENTRY64;

const IMAGE_EXCEPTION_DIRECTORY_INDEX: usize = 3;

#[derive(RefMutFields)]
pub struct ExceptionTable {
    #[ref_]
    entrys: Vec<*mut IMAGE_FUNCTION_ENTRY64>,
}

impl fmt::Debug for ExceptionTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in self.entrys.iter() {
            writeln!(f, "{:#X?}", unsafe { entry.as_ref().unwrap() })?;
        }

        Ok(())
    }
}

impl ExceptionTable {
    pub fn from_image(image: &Image) -> Result<Option<Self>> {
        let exception_directory = image
            .get_nt_header()
            .get_optional_header()
            .get_data_directory()[IMAGE_EXCEPTION_DIRECTORY_INDEX];

        if exception_directory.virtual_address == 0 || exception_directory.size == 0 {
            return Ok(None);
        }

        let size = to_prim!(exception_directory.size => usize);

        let exception_directory = convert_mut_slice_to_ptr!(image.bytes_mut() => IMAGE_FUNCTION_ENTRY64,
                                                            image.rva_to_fo(to_prim!(exception_directory.virtual_address => usize)).unwrap());

        let entrys: Vec<_> = unsafe {
            std::slice::from_raw_parts_mut(
                exception_directory,
                size / std::mem::size_of::<IMAGE_FUNCTION_ENTRY64>(),
            )
            .into_iter()
            .map(|entry| entry as *mut _)
            .collect()
        };

        Ok(Some(Self { entrys }))
    }
}
