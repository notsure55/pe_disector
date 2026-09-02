use super::windows_types::*;
use anyhow::{anyhow, Result};
use std::ffi::CStr;
use std::fmt;
use std::io::Read;
use std::path::Path;

use super::arch;
use super::export;
use super::import;
use super::pe_headers::*;
use super::section::section_table;
use crate::*;

impl_arithmetic_traits_for_wrappers!(Fo, usize);
impl_arithmetic_traits_for_wrappers!(Rva, usize);
impl_arithmetic_traits_for_wrappers!(Va, usize);

pub struct ImageOwned {
    bytes: Vec<u8>,
    headers: PeHeaders,
    section_table: section_table::SectionTableOwned,
    import_table: Option<import::ImportTable>,
    exception_table: exception::ExceptionTable,
    export_table: Option<export::ExportTable>,
}

impl fmt::Debug for ImageOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "{:#X?}", self.headers);
        write!(
            f,
            "{:?}",
            &self.section_table as &dyn section_table::SectionTable
        );
        writeln!(f, "{:#X?}", self.import_table);
        writeln!(f, "{:#X?}", self.exception_table);
        writeln!(f, "{:#X?}", self.export_table);

        Ok(())
    }
}

impl ImageOwned {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;

        let mut image_ref = ImageRef::from_binary_bytes(&bytes)?;

        let import_table = import::ImportTable::from_image(&image_ref)?;

        let exception_table = exception::ExceptionTable::from_image(&image_ref)?;

        let export_table = export::ExportTable::from_image(&image_ref)?;

        let headers = image_ref.headers_to_owned().unwrap();

        let section_table = image_ref.section_table_to_owned().unwrap();

        Ok(Self {
            bytes,
            headers,
            section_table,
            import_table,
            exception_table,
            export_table,
        })
    }
}

impl Image for ImageOwned {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn headers(&self) -> &PeHeaders {
        &self.headers
    }

    fn section_table(&self) -> &dyn section_table::SectionTable {
        &self.section_table
    }
}

pub struct ImageRef<'a> {
    bytes: &'a [u8],
    headers: Option<PeHeaders>,
    section_table: Option<section_table::SectionTableRef<'a>>,
}

impl<'a> ImageRef<'a> {
    pub fn from_binary_bytes(bytes: &'a [u8]) -> Result<Self> {
        let headers = PeHeaders::from_bytes(bytes);

        let section_table = section_table::SectionTableRef::from_bytes(
            &bytes,
            headers.nt_header().file_header().number_of_sections(),
            to_usize!(headers.dos_header().e_lfanew) + std::mem::size_of::<IMAGE_NT_HEADERS64>(),
        )?;

        Ok(Self {
            bytes,
            headers: Some(headers),
            section_table: Some(section_table),
        })
    }

    pub fn section_table_to_owned(&mut self) -> Option<section_table::SectionTableOwned> {
        if let Some(table) = self.section_table.take() {
            Some(table.to_owned())
        } else {
            None
        }
    }

    pub fn headers_to_owned(&mut self) -> Option<PeHeaders> {
        self.headers.take()
    }

    pub fn to_owned(
        self,
        import_table: Option<import::ImportTable>,
        exception_table: exception::ExceptionTable,
        export_table: Option<export::ExportTable>,
    ) -> ImageOwned {
        ImageOwned {
            bytes: self.bytes.to_vec(),
            headers: self.headers.unwrap(),
            section_table: self.section_table.unwrap().to_owned(),
            import_table,
            exception_table,
            export_table,
        }
    }
}

impl<'a> fmt::Debug for ImageRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "PeHeaders: {:#X?}", self.headers);
        if let Some(table) = self.section_table.as_ref() {
            write!(f, "{:?}", table as &dyn section_table::SectionTable);
        }

        Ok(())
    }
}

impl<'a> Image for ImageRef<'a> {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    fn headers(&self) -> &PeHeaders {
        self.headers.as_ref().unwrap()
    }
    fn section_table(&self) -> &dyn section_table::SectionTable {
        self.section_table.as_ref().unwrap()
    }
}

pub trait Image: fmt::Debug {
    fn read_slice_from_rva<T>(&self, rva: Rva, size: usize) -> Result<&[T]> {
        let fo = self.rva_to_fo(rva)?;

        let slice = unsafe {
            std::slice::from_raw_parts(
                self.bytes()
                    .get(*fo..*fo + size)
                    .ok_or_else(|| anyhow!("failed to get slice of bytes from rva and size"))?
                    .as_ptr() as *const T,
                size,
            )
        };

        Ok(slice)
    }
    fn rva_to_fo(&self, rva: Rva) -> Result<Fo> {
        for section_header in self.section_table().section_headers() {
            let raw = section_header.raw();
            if *rva < to_usize!(raw.virtual_address + raw.virtual_size)
                && *rva >= to_usize!(raw.virtual_address)
            {
                return Ok(Fo::from(
                    *rva - to_usize!(raw.virtual_address) + to_usize!(raw.pointer_to_raw_data),
                ));
            }
        }

        Err(anyhow!(
            "Failed to find file offset for rva, did you enter in a correct rva?"
        ))
    }
    fn va_to_fo(&self, va: Va) -> Result<Fo> {
        let image_base_va = to_usize!(self.headers().nt_header().optional_header().image_base());

        let rva = Rva::from(*va - image_base_va);

        self.rva_to_fo(rva)
    }
    fn read_from_rva<T>(&self, rva: Rva) -> Result<T> {
        Ok(self.read_from_fo::<T>(self.rva_to_fo(rva)?))
    }
    fn read_c_str_from_rva(&self, rva: Rva) -> Result<String> {
        self.read_c_str_from_fo(self.rva_to_fo(rva)?)
    }
    fn read_c_str_from_fo(&self, fo: Fo) -> Result<String> {
        let bytes = self.bytes().get(*fo..).unwrap();
        Ok(c_str!(bytes))
    }
    fn read_from_fo<T>(&self, fo: Fo) -> T {
        unsafe {
            self.bytes()
                .get(*fo..*fo + std::mem::size_of::<T>())
                .unwrap()
                .as_ptr()
                .cast::<T>()
                .read_unaligned()
        }
    }
    fn read_from_rva_to_buf(&self, rva: Rva, size: usize) -> Result<Vec<u8>> {
        Ok(self.read_from_fo_to_buf(self.rva_to_fo(rva)?, size))
    }
    fn read_from_fo_to_buf(&self, fo: Fo, size: usize) -> Vec<u8> {
        self.bytes().get(*fo..*fo + size).unwrap().to_vec()
    }
    fn arch(&self) -> arch::Architecture {
        self.headers().nt_header().file_header().arch()
    }
    fn bytes(&self) -> &[u8];
    fn headers(&self) -> &PeHeaders;
    fn section_table(&self) -> &dyn section_table::SectionTable;
}
