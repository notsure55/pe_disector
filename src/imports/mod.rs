use windows_types::{IMAGE_IMPORT_BY_NAME, IMAGE_IMPORT_BY_ORDINAL};

mod dll_import;
pub mod import_table;

enum Import {
    Name(*mut IMAGE_IMPORT_BY_NAME),
    Ordinal(*mut IMAGE_IMPORT_BY_ORDINAL),
}

#[derive(Debug)]
pub enum ImportSimple {
    Name(String),
    Ordinal(u16),
}

impl PartialEq<&Import> for ImportSimple {
    fn eq(&self, other: &&Import) -> bool {
        match self {
            Self::Name(name) => {
                if let Import::Name(ptr) = other {
                    unsafe { ptr.as_ref().unwrap() }.name() == name.as_ref()
                } else {
                    false
                }
            }
            Self::Ordinal(ord) => {
                if let Import::Ordinal(ptr) = other {
                    *ord == unsafe { ptr.as_ref().unwrap() }.ordinal_number
                } else {
                    false
                }
            }
        }
    }
}

impl From<u16> for ImportSimple {
    fn from(value: u16) -> Self {
        Self::Ordinal(value)
    }
}
impl From<&str> for ImportSimple {
    fn from(value: &str) -> Self {
        Self::Name(value.to_string())
    }
}
impl From<String> for ImportSimple {
    fn from(value: String) -> Self {
        Self::Name(value)
    }
}

use std::fmt;

impl fmt::Debug for Import {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(ptr) => writeln!(f, "{:?}", unsafe { ptr.as_ref().unwrap() }),
            Self::Ordinal(ptr) => writeln!(f, "{:?}", unsafe { ptr.as_ref().unwrap() }),
        }
    }
}
