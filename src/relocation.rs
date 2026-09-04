use super::pe::ImageOwned;
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

#[derive(Debug)]
enum RelocType {
    ImageRelBasedAbsolute,
    ImageRelBasedHigh,
    ImageRelBasedLow,
    ImageRelBasedHighlow,
    ImageRelBasedHighadj,
    ImageRelBasedMipsJmpaddr,
    ImageRelBasedThumbMov32,
    ImageRelBasedRiscvLow12s,
    ImageRelBasedMipsJmpaddr16,
    ImageRelBasedDir64,
    Unknown,
}

impl RelocType {
    pub const fn new(typ: u8) -> Self {
        match typ {
            0 => Self::ImageRelBasedAbsolute,
            1 => Self::ImageRelBasedHigh,
            2 => Self::ImageRelBasedLow,
            3 => Self::ImageRelBasedHighlow,
            4 => Self::ImageRelBasedHighadj,
            5 => Self::ImageRelBasedMipsJmpaddr,
            7 => Self::ImageRelBasedThumbMov32,
            8 => Self::ImageRelBasedRiscvLow12s,
            9 => Self::ImageRelBasedMipsJmpaddr16,
            10 => Self::ImageRelBasedDir64,
            _ => Self::Unknown,
        }
    }
}

#[repr(C)]
#[derive(Clone)]
struct TypeOffset(pub u16);

impl fmt::Debug for TypeOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "Offset: 0x{:X} Type: {:?}", self.offset(), self.typ())
    }
}

impl TypeOffset {
    pub fn typ(&self) -> RelocType {
        RelocType::new((self.0 >> 12) as _)
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

    /*pub fn update_base_relocs(
        &self,
        bytes: &mut [u8],
        old_image_base: Va,
        new_image_base: Va,
    ) -> Result<()> {
        for block in self.blocks.iter() {
            let page_rva = block.page_rva;

            for type_offset in block.type_offsets.iter() {
                let typ = type_offset.typ();
                let offset = type_offset.offset();

                let base_reloc_rva = page_rva + to_usize!(offset);

                eprintln!("Found reloc {:#X?}", base_reloc_rva);

                let base_reloc = image.read_from_rva::<Va>(base_reloc_rva)?;

                eprintln!("Read reloc {:#X?}", base_reloc);

                let new_base_reloc = base_reloc - old_image_base + new_image_base;

                image.write_to_rva(base_reloc_rva, new_base_reloc)?;

                eprintln!("Wrote new reloc {:#X?}", new_base_reloc);
            }
        }

        Ok(())
    }*/
}
