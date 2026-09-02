use crate::*;
use anyhow::Result;
use std::fmt;

const RELOCATION_DIRECTORY_TABLE_INDEX: usize = 5;

#[repr(C)]
struct BaseReloc {
    rva: u32,
    block_size: u32,
}

#[derive(Debug)]
struct RelocationBlock {
    page_rva: Rva,
    block_size: usize,
    type_offsets: Vec<TypeOffset>,
}

impl RelocationBlock {
    pub fn new(page_rva: u32, block_size: u32, type_offsets: &[TypeOffset]) -> Result<Self> {
        Ok(Self {
            page_rva: Rva::try_from(page_rva)?,
            block_size: to_usize!(block_size),
            type_offsets: type_offsets.to_vec(),
        })
    }
}

#[repr(C)]
#[derive(Clone)]
struct TypeOffset(pub u16);

impl fmt::Debug for TypeOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "Offset: 0x{:X} Type: {:X}", self.offset(), self.typ())
    }
}

impl TypeOffset {
    pub fn typ(&self) -> u16 {
        self.0 >> 12
    }

    pub fn offset(&self) -> u16 {
        self.0 & 0x0FFF
    }
}

#[derive(Debug)]
pub struct RelocationTable {
    blocks: Vec<RelocationBlock>,
}

impl RelocationTable {
    pub fn from_image(image: &impl Image) -> Result<Self> {
        let relocation_directory_image = image
            .headers()
            .nt_header()
            .optional_header()
            .data_directory()[RELOCATION_DIRECTORY_TABLE_INDEX];

        let base = Rva::try_from(relocation_directory_image.virtual_address)?;
        let size = to_usize!(relocation_directory_image.size);

        let mut relocation_blocks = Vec::new();
        let mut offset = 0;

        while offset < size {
            let base_reloc = image.read_from_rva::<BaseReloc>(base + offset)?;

            let count = (to_usize!(base_reloc.block_size) - std::mem::size_of::<BaseReloc>()) / 2;

            let type_offsets = image.read_slice_from_rva::<TypeOffset>(
                base + std::mem::size_of::<BaseReloc>() + offset,
                count,
            )?;

            relocation_blocks.push(RelocationBlock::new(
                base_reloc.rva,
                base_reloc.block_size,
                type_offsets,
            )?);

            offset += to_usize!(base_reloc.block_size);
        }

        Ok(Self {
            blocks: relocation_blocks,
        })
    }
}
