use crate::image::Va;
use std::ops::Deref;
use std::ops::DerefMut;
use windows_types::IMAGE_BASE_RELOCATION;

use std::fmt;
pub mod relocation_table;

#[repr(C)]
pub struct Block(u16);

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "offset: {:X}, typ: {:X},", self.offset(), self.typ())
    }
}

impl Block {
    fn offset(&self) -> u16 {
        self.0 & 0xFFF
    }
    fn typ(&self) -> u16 {
        self.0 >> 12
    }
}

struct BaseRelocationBlock {
    raw: *mut IMAGE_BASE_RELOCATION,
    // reloc block, with its virtual address that needs to be updated
    blocks: Vec<(*mut Block, *mut Va)>,
}

impl BaseRelocationBlock {
    pub fn update_relocs(&self, old_image_base: Va, new_image_base: Va) {
        for (_, va) in self.blocks.iter() {
            unsafe { va.write_unaligned(va.read_unaligned() - old_image_base + new_image_base) };
        }
    }
}

impl DerefMut for BaseRelocationBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut_unchecked() }
    }
}

impl Deref for BaseRelocationBlock {
    type Target = IMAGE_BASE_RELOCATION;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref_unchecked() }
    }
}

impl fmt::Debug for BaseRelocationBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:#X?}", unsafe { self.raw.as_ref().unwrap() })?;

        for (block, va) in self.blocks.iter() {
            writeln!(
                f,
                "{:#X?} VA: {:X?}",
                unsafe { block.as_ref().unwrap() },
                unsafe { va.read_unaligned() }
            )?;
        }

        Ok(())
    }
}
