use super::image::{Image, Va};
use crate::{convert_mut_slice_to_ptr, to_prim};
use anyhow::Result;
use std::ops::Deref;
use std::ops::DerefMut;
use windows_types::{IMAGE_TLS_DIRECTORY, IMAGE_TLS_DIRECTORY64};

const IMAGE_TLS_DIRECTORY_INDEX: usize = 9;

pub struct TlsDirectory {
    raw: *mut dyn IMAGE_TLS_DIRECTORY,
}

impl DerefMut for TlsDirectory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut().unwrap() }
    }
}

impl Deref for TlsDirectory {
    type Target = dyn IMAGE_TLS_DIRECTORY;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref().unwrap() }
    }
}

impl TlsDirectory {
    pub fn from_image(image: &Image) -> Result<Option<Self>> {
        let tls_directory = image
            .get_nt_header()
            .get_optional_header()
            .get_data_directory()[IMAGE_TLS_DIRECTORY_INDEX];

        if tls_directory.virtual_address == 0 || tls_directory.size == 0 {
            return Ok(None);
        }

        let _ = to_prim!(tls_directory.size => usize);

        let tls_directory: *mut dyn IMAGE_TLS_DIRECTORY = convert_mut_slice_to_ptr!(
            image.bytes_mut() => IMAGE_TLS_DIRECTORY64,
            image.rva_to_fo(to_prim!(tls_directory.virtual_address => usize)).unwrap());

        let tls_directory_ref = unsafe { tls_directory.as_ref().unwrap() };

        let address_of_callbacks_fo = image
            .va_to_fo(to_prim!(tls_directory_ref.get_address_of_callbacks() => Va))
            .unwrap();

        if image.read_from_fo::<u32>(address_of_callbacks_fo) > 0 {
            println!("must handle tls callbacks!");
        }

        Ok(Some(Self { raw: tls_directory }))
    }
}
