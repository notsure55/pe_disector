use super::dos_header::DosHeader;
use super::exception_table::ExceptionTable;
use super::exports::export_table::ExportTable;
use super::imports::import_table::ImportTable;
use super::load_configuration::LoadConfigDirectory;
use super::nt_header::NtHeader;
use super::reloc::relocation_table::RelocationTable;
use super::section_table::SectionTable;
use super::tls::TlsDirectory;
use crate::to_prim;
use anyhow::Result;
use ref_mut_field::RefMutFields;
use std::cell::UnsafeCell;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use std::fmt;

#[derive(RefMutFields)]
pub struct Image {
    bytes: UnsafeCell<Vec<u8>>,
    #[ref_mut]
    dos_header: DosHeader,
    #[ref_mut]
    nt_header: NtHeader,
    #[ref_mut]
    section_table: SectionTable,
    #[ref_mut]
    import_table: Option<ImportTable>,
    #[ref_mut]
    export_table: Option<ExportTable>,
    #[ref_mut]
    exception_table: Option<ExceptionTable>,
    #[ref_mut]
    relocation_table: Option<RelocationTable>,
    #[ref_mut]
    tls_directory: Option<TlsDirectory>,
    #[ref_mut]
    load_config_directory: Option<LoadConfigDirectory>,
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = writeln!(f, "{:#X?}", *self.dos_header);
        writeln!(f, "{:#X?}", &*self.nt_header)
    }
}

impl Image {
    pub fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path)?;

        let mut buf = Vec::new();

        file.read_to_end(&mut buf)?;

        let bytes = UnsafeCell::new(buf);
        let bytes_mut_slice = unsafe { bytes.get().as_mut_unchecked().as_mut_slice() };

        let dos_header = DosHeader::from_bytes(bytes_mut_slice);
        let nt_header = NtHeader::from_bytes(bytes_mut_slice, dos_header.e_lfanew);

        let section_table = SectionTable::from_bytes(
            bytes_mut_slice,
            to_prim!(dos_header.e_lfanew => usize)
                + std::mem::size_of_val(&nt_header.get_file_header())
                + std::mem::size_of_val(&nt_header.get_signature())
                + to_prim!(nt_header.get_file_header().size_of_optional_header => usize),
            to_prim!(nt_header.get_file_header().number_of_sections => usize),
        );

        let mut image = Self {
            bytes,
            dos_header,
            nt_header,
            section_table,
            import_table: None,
            export_table: None,
            exception_table: None,
            relocation_table: None,
            tls_directory: None,
            load_config_directory: None,
        };

        image.import_table = ImportTable::from_image(&image);
        image.export_table = ExportTable::from_image(&image)?;
        image.exception_table = ExceptionTable::from_image(&image)?;
        image.relocation_table = RelocationTable::from_image(&image)?;
        image.tls_directory = TlsDirectory::from_image(&image)?;
        image.load_config_directory = LoadConfigDirectory::from_image(&image)?;

        Ok(image)
    }

    // Equivelent to LoadLibrary
    pub fn map_image_to_virtual_memory(&self) -> Option<()> {
        None
    }

    pub fn bytes(&self) -> &[u8] {
        unsafe { &*self.bytes.get() }
    }

    pub fn bytes_mut(&self) -> &mut [u8] {
        unsafe { &mut *self.bytes.get() }
    }

    pub fn rva_to_fo(&self, rva: Rva) -> Option<Fo> {
        for header in self.section_table.get_headers().iter() {
            let header_rva = to_prim!(header.virtual_address => usize);
            let header_vs = to_prim!(header.virtual_size => usize);
            let header_range = to_prim!(header_rva => usize) + to_prim!(header_vs => usize);

            if rva < header_range && rva >= header_rva {
                return Some(rva - header_rva + to_prim!(header.pointer_to_raw_data => usize));
            }
        }

        todo!("Handle invalid address inputs");
    }

    pub fn va_to_rva(&self, va: Va) -> Rva {
        va - to_prim!(self.get_nt_header().get_optional_header().get_image_base() => usize)
    }

    pub fn va_to_fo(&self, va: Va) -> Option<Fo> {
        self.rva_to_fo(self.va_to_rva(va))
    }

    pub fn read_from_va<T>(&self, va: Va) -> T {
        let fo = self.va_to_fo(va).unwrap();
        self.read_from_fo(fo)
    }

    pub fn read_from_fo<T>(&self, fo: Fo) -> T {
        let bytes = self.bytes();

        if let Some(slice) = bytes.get(fo..fo + std::mem::size_of::<T>()) {
            unsafe { slice.as_ptr().cast::<T>().read_unaligned() }
        } else {
            todo!("Handle unreachable file offset")
        }
    }

    pub fn read_from_rva<T>(&self, rva: Rva) -> T {
        let fo = self.rva_to_fo(rva).unwrap();
        self.read_from_fo(fo)
    }

    pub fn read_cstr_from_fo(&self, fo: Fo) -> Result<String> {
        let bytes = self.bytes();

        Ok(std::ffi::CStr::from_bytes_until_nul(&bytes[fo..])?
            .to_string_lossy()
            .into_owned())
    }

    pub fn write_to_va<T>(&self, va: Va, value: T) {
        let fo = self.va_to_fo(va).unwrap();
        self.write_to_fo(fo, value)
    }

    pub fn write_to_fo<T>(&self, fo: Fo, value: T) {
        let bytes = self.bytes_mut();

        if let Some(slice) = bytes.get_mut(fo..fo + std::mem::size_of::<T>()) {
            unsafe { slice.as_mut_ptr().cast::<T>().write_unaligned(value) }
        } else {
            todo!("Handle unreachable file offset")
        }
    }

    pub fn write_to_rva<T>(&self, rva: Rva, value: T) {
        let fo = self.rva_to_fo(rva).unwrap();
        self.write_to_fo(fo, value)
    }

    pub fn write_bytes_to_va(&self, va: Va, value: &[u8]) {
        let fo = self.va_to_fo(va).unwrap();
        self.write_bytes_to_fo(fo, value)
    }

    pub fn write_bytes_to_fo(&self, fo: Fo, value: &[u8]) {
        let bytes = self.bytes_mut();

        if let Some(slice) = bytes.get_mut(fo..fo + value.len()) {
            unsafe {
                slice
                    .as_mut_ptr()
                    .cast::<u8>()
                    .copy_from(value.as_ptr(), value.len())
            }
        } else {
            todo!("Handle unreachable file offset")
        }
    }

    pub fn write_bytes_to_rva(&self, rva: Rva, value: &[u8]) {
        let fo = self.rva_to_fo(rva).unwrap();
        self.write_bytes_to_fo(fo, value)
    }
}

pub type Fo = usize;
pub type Rva = usize;
pub type Va = usize;
