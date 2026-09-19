#![allow(unused_unsafe)]

mod dos_header;
pub mod image;
mod import_table;
mod nt_header;
mod section_table;

#[macro_export]
macro_rules! convert_unsafe_cell_bytes {
    ($ptr:expr => $typ:ty) => {
        {
            use crate::convert_mut_ref;
            convert_mut_ref!((&mut *($ptr)) => $typ)
        }
    };
    ($ptr:expr => $typ:ty, $offset:ident) => {
        {
            use crate::convert_mut_ref;
            let slice = (&mut *$ptr).get_mut($offset..).unwrap();
            convert_mut_ref!(slice => $typ)
        }
    };
}

#[macro_export]
macro_rules! convert_mut_ref {
    ($ptr:expr => $typ:ty) => {
        unsafe { $ptr.as_mut_ptr().cast::<$typ>() }
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
