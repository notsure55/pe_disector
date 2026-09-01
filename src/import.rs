use super::arch;
use super::pe::*;
use super::windows_types::*;
use anyhow::Result;

const IMPORT_DIRECTORY_TABLE_INDEX: usize = 1;

#[derive(Debug)]
pub struct ImportTable {
    imports: Vec<ImportDescriptor>,
}

#[derive(Debug)]
struct ByName {
    name: String,
    address_entry: Fo,
}

#[derive(Debug)]
struct ByOrdinal {
    number: usize,
    address_entry: Fo,
}

#[derive(Debug)]
enum ImportTableEntry {
    Name(ByName),
    Ordinal(ByOrdinal),
}

#[derive(Debug)]
struct ImportDescriptor {
    name: String,
    forwarder_chain: u32,
    import_lookup_table: Fo,
    import_address_table: Fo,
    import_table_entrys: Vec<ImportTableEntry>,
}

impl ImportDescriptor {
    pub fn new(
        name: String,
        forwarder_chain: u32,
        import_lookup_table: Fo,
        import_address_table: Fo,
        import_table_entrys: Vec<ImportTableEntry>,
    ) -> Self {
        Self {
            name,
            forwarder_chain,
            import_lookup_table,
            import_address_table,
            import_table_entrys,
        }
    }
}

// https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#the-idata-section
impl ImportTable {
    pub fn from_image(pe_image: &impl Image) -> Result<Option<Self>> {
        let import_directory_image = pe_image
            .headers()
            .nt_header()
            .optional_header()
            .data_directory()[IMPORT_DIRECTORY_TABLE_INDEX];

        if import_directory_image.virtual_address == 0 && import_directory_image.size == 0 {
            return Ok(None);
        }

        let import_descriptors_bytes: Vec<u8> = pe_image.read_from_rva_to_buf(
            Rva::try_from(import_directory_image.virtual_address)?,
            usize::try_from(import_directory_image.size)?
                - std::mem::size_of::<IMAGE_IMPORT_DESCRIPTOR>(),
        )?;

        let import_descriptor_count = (usize::try_from(import_directory_image.size)?
            - std::mem::size_of::<IMAGE_IMPORT_DESCRIPTOR>())
            / std::mem::size_of::<IMAGE_IMPORT_DESCRIPTOR>();

        let import_descriptors = {
            unsafe {
                std::slice::from_raw_parts(
                    import_descriptors_bytes.as_ptr() as *const IMAGE_IMPORT_DESCRIPTOR,
                    import_descriptor_count,
                )
            }
        };

        let mut descriptors = Vec::with_capacity(import_descriptor_count);

        for desc in import_descriptors.iter() {
            let name = pe_image.read_c_str_from_rva(Rva::try_from(desc.name)?)?;

            let import_lookup_table =
                pe_image.rva_to_fo(Rva::try_from(desc.original_first_thunk)?)?;

            let import_address_table = pe_image.rva_to_fo(Rva::try_from(desc.first_thunk)?)?;

            let arch = pe_image.arch();

            let mut table_entrys = Vec::new();

            for i in 0..1024 {
                let table_entry = {
                    let (name_address, address_entry) = match arch {
                        arch::Architecture::X64 => {
                            let address_entry = import_address_table + i * 8;

                            let name_address =
                                pe_image.read_from_fo::<Rva>(import_lookup_table + i * 8);

                            (name_address, address_entry)
                        }
                        arch::Architecture::X32 => {
                            let address_entry = import_address_table + i * 4;

                            let name_address = Rva::try_from(
                                pe_image.read_from_fo::<u32>(import_lookup_table + i * 4),
                            )?;

                            (name_address, address_entry)
                        }
                    };

                    // IS ORDINAL
                    if *name_address & 0x8000000000000000 > 0 || *name_address & 0x80000000 > 0 {
                        let ordinal = *name_address & 0b1111111111111111;

                        ImportTableEntry::Ordinal(ByOrdinal {
                            number: ordinal,
                            address_entry,
                        })
                    } else {
                        if *name_address > 0 {
                            let name = pe_image.read_c_str_from_rva(name_address + 2)?;

                            ImportTableEntry::Name(ByName {
                                name,
                                address_entry,
                            })
                        } else {
                            break;
                        }
                    }
                };

                table_entrys.push(table_entry);
            }

            descriptors.push(ImportDescriptor::new(
                name,
                desc.forwarder_chain,
                import_lookup_table,
                import_address_table,
                table_entrys,
            ));
        }

        Ok(Some(Self {
            imports: descriptors,
        }))
    }
}
