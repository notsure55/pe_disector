use super::{BaseRelocationBlock, Block};
use crate::image::{Image, Rva, Va};
use crate::{convert_mut_slice_to_ptr, to_prim};
use anyhow::Result;
use ref_mut_field::RefMutFields;
use windows_types::IMAGE_BASE_RELOCATION;

const IMAGE_RELOCATION_DIRECTORY_INDEX: usize = 5;

#[derive(RefMutFields, Debug)]
pub struct RelocationTable {
    #[ref_mut]
    base_relocs: Vec<BaseRelocationBlock>,
}

impl RelocationTable {
    pub fn from_image(image: &Image) -> Result<Option<Self>> {
        let relocation_directory = image
            .get_nt_header()
            .get_optional_header()
            .get_data_directory()[IMAGE_RELOCATION_DIRECTORY_INDEX];

        if relocation_directory.virtual_address == 0 || relocation_directory.size == 0 {
            return Ok(None);
        }

        let mut size = to_prim!(relocation_directory.size => usize);

        let reloc_fo = image
            .rva_to_fo(to_prim!(relocation_directory.virtual_address => Rva))
            .unwrap();

        let mut bytes = image
            .bytes_mut()
            .get_mut(reloc_fo..reloc_fo + size)
            .unwrap();

        let mut base_relocs = Vec::new();

        while size > 0 {
            let base_reloc = convert_mut_slice_to_ptr!(bytes => IMAGE_BASE_RELOCATION);

            let base_reloc_ref = unsafe { base_reloc.as_ref().unwrap() };

            let base_reloc_fo = to_prim!(image
                                         .rva_to_fo(to_prim!(base_reloc_ref.virtual_address => Rva))
                                         .unwrap() => isize);

            let reloc_size = to_prim!(base_reloc_ref.size => usize);

            let blocks = unsafe { std::slice::from_raw_parts_mut(
                convert_mut_slice_to_ptr!(bytes => Block, std::mem::size_of::<IMAGE_BASE_RELOCATION>()),
                (reloc_size - std::mem::size_of::<IMAGE_BASE_RELOCATION>()) / 2,
            )} .into_iter().enumerate().filter_map(|(i, block)| {
                if block.0 == 0x0 {
                    None
                } else {
                    if block.typ() != 10 {
                        todo!("Handle this case for blocks! found an unreachable block {} in {base_reloc_ref:#X?} at index {i}", block.typ());
                    } else {
                        let offset = to_prim!(block.offset() => isize) + base_reloc_fo;

                        let va = convert_mut_slice_to_ptr!(image.bytes_mut() => Va, offset);

                        Some((block as *mut _, va))
                    }
                }
            }).collect();

            size -= reloc_size;
            bytes = &mut bytes[reloc_size..];

            base_relocs.push(BaseRelocationBlock {
                raw: base_reloc,
                blocks,
            })
        }

        Ok(Some(Self { base_relocs }))
    }
    pub fn update_relocs(&self, old_image_base: Va, new_image_base: Va) {
        for reloc in self.base_relocs.iter() {
            reloc.update_relocs(old_image_base, new_image_base);
        }
    }
}
