use super::image::Image;
use crate::{convert_mut_slice_to_ptr, to_prim};
use anyhow::Result;
use std::ops::Deref;
use std::ops::DerefMut;
use windows_types::{IMAGE_LOAD_CONFIG_DIRECTORY, IMAGE_LOAD_CONFIG_DIRECTORY64};

const IMAGE_LOAD_CONFIG_DIRECTORY_INDEX: usize = 10;

#[derive(Debug)]
pub struct LoadConfigDirectory {
    raw: *mut dyn IMAGE_LOAD_CONFIG_DIRECTORY,
}

impl DerefMut for LoadConfigDirectory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut().unwrap() }
    }
}

impl Deref for LoadConfigDirectory {
    type Target = dyn IMAGE_LOAD_CONFIG_DIRECTORY;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref().unwrap() }
    }
}

impl LoadConfigDirectory {
    pub fn from_image(image: &Image) -> Result<Option<Self>> {
        let load_config_directory = image
            .get_nt_header()
            .get_optional_header()
            .get_data_directory()[IMAGE_LOAD_CONFIG_DIRECTORY_INDEX];

        if load_config_directory.virtual_address == 0 || load_config_directory.size == 0 {
            return Ok(None);
        }

        let _ = to_prim!(load_config_directory.size => usize);

        let load_config_directory: *mut dyn IMAGE_LOAD_CONFIG_DIRECTORY = convert_mut_slice_to_ptr!(
            image.bytes_mut() => IMAGE_LOAD_CONFIG_DIRECTORY64,
            image.rva_to_fo(to_prim!(load_config_directory.virtual_address => usize)).unwrap());

        Ok(Some(Self {
            raw: load_config_directory,
        }))
    }
}
