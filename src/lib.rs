#![allow(unused)]

use std::path::Path;

mod arch;
mod exception;
mod export;
mod import;
pub mod pe;
mod pe_headers;
mod section;
pub mod windows_types;

use pe::*;

#[test]
fn va_to_fo_check() {
    const MAIN_START_ADDRESS: usize = 0x140001090;

    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples\\sample.exe");

    let image = PeImage::from_path(sample_path).unwrap();

    let fo = image.va_to_fo(Va::from(MAIN_START_ADDRESS)).unwrap();
    let first_ins: [u8; 4] = image.read_from_fo(fo);

    assert_eq!(first_ins, [0x48, 0x83, 0xec, 0x28]);
}

#[macro_export]
macro_rules! c_str {
    ($str:expr) => {{
        use std::ffi::CStr;

        CStr::from_bytes_until_nul($str)
            .unwrap()
            .to_string_lossy()
            .into()
    }};
}

#[macro_export]
macro_rules! to_usize {
    ($var:expr) => {
        usize::try_from($var)?
    };
}

use std::convert;
use std::ops;

#[macro_export]
macro_rules! impl_arithmetic_traits_for_wrappers {
    ($name:ident, $type:ty) => {
        #[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
        pub struct $name($type);

        impl convert::From<$type> for $name {
            fn from(value: $type) -> Self {
                Self { 0: value }
            }
        }
        impl ops::Add<$type> for $name {
            type Output = Self;

            fn add(self, other: $type) -> Self {
                Self(self.0 + other)
            }
        }
        impl ops::BitOr<$type> for $name {
            type Output = Self;

            // rhs is the "right-hand side" of the expression `a | b`
            fn bitor(self, rhs: $type) -> Self::Output {
                Self(self.0 | rhs)
            }
        }
        impl ops::BitAnd<$type> for $name {
            type Output = Self;

            // rhs is the "right-hand side" of the expression `a | b`
            fn bitand(self, rhs: $type) -> Self::Output {
                Self(self.0 & rhs)
            }
        }
        impl ops::Deref for $name {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl convert::TryFrom<u32> for $name {
            type Error = anyhow::Error;

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                Ok(Self {
                    0: usize::try_from(value)?,
                })
            }
        }
    };
}
