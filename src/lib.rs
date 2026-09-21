#![allow(unused_unsafe)]

mod base_reloc;
mod dos_header;
mod exception_table;
mod exports;
pub mod image;
mod imports;
mod nt_header;
mod section_table;

#[macro_export]
macro_rules! convert_mut_slice_to_ptr {
    ($ptr:expr => $typ:ty) => {
        unsafe { $ptr.as_mut_ptr().cast::<$typ>() }
    };
    ($ptr:expr => $typ:ty, $offset:expr) => {
        unsafe { $ptr.as_mut_ptr().byte_offset(crate::to_prim!($offset => isize)).cast::<$typ>() }
    };
}

#[macro_export]
macro_rules! to_prim {
    ($name:ident as $typ:ty) => {{
        <$typ>::try_from($name)
            .map_err(|err| eprintln!("{err} {:#?}", $name))
            .unwrap()
    }};
    ($name:expr => $typ:ty) => {{
        <$typ>::try_from($name)
            .map_err(|err| eprintln!("{err} {:#?}", $name))
            .unwrap()
    }};
}
