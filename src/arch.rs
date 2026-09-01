use std::convert;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum Architecture {
    X64,
    X32,
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X64 => write!(f, "AMD64"),
            Self::X32 => write!(f, "I386"),
        }
    }
}

impl convert::TryFrom<u16> for Architecture {
    type Error = &'static str;

    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        match value {
            0x8664 => Ok(Architecture::X64),
            0x14C => Ok(Architecture::X32),
            _ => todo!("Handle other architectures!"),
        }
    }
}
