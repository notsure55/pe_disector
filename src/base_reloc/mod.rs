use crate::image::Rva;
use crate::to_prim;

mod relocation_table;

#[repr(C)]
#[derive(Debug)]
pub struct Block(u16);

struct BaseRelocationBlock {
    page_rva: Rva,
    blocks: Vec<*mut Block>,
}

impl BaseRelocationBlock {
    pub fn from_bytes<'a>(bytes: &'a mut [Block]) -> (Self, &'a mut [Block]) {
        let page_rva =
            to_prim!(unsafe { bytes.as_mut_ptr().cast::<u32>().read_unaligned() } => usize);

        let size = unsafe {
            bytes
                .as_mut_ptr()
                .byte_offset(to_prim!(std::mem::size_of::<u32>() => isize))
                .cast::<u32>()
                .read_unaligned()
        };

        let size_of_start = (std::mem::size_of::<u32>() + std::mem::size_of::<u32>()) / 2;

        let block_len = to_prim!(size as usize) / 2 - size_of_start;

        let bytes = bytes.get_mut(size_of_start..block_len).unwrap();

        let blocks: Vec<_> = bytes
            .iter_mut()
            .map_while(|block| {
                if block.0 == 0x0 {
                    None
                } else {
                    Some(block as *mut _)
                }
            })
            .collect();

        let bytes = bytes.get_mut(blocks.len()..).unwrap();

        (Self { page_rva, blocks }, bytes)
    }
}
