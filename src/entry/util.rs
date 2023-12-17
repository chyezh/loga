/// The `Attr` is used to identify the type of the entry.
#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub struct Attr(i32);

impl From<i32> for Attr {
    fn from(attr: i32) -> Self {
        Self(attr)
    }
}

impl From<Attr> for i32 {
    fn from(attr: Attr) -> Self {
        attr.0
    }
}

/// The `Magic` is used to identify the version of the entry.
/// For backward compatibility, we need to keep the old version.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum Magic {
    V1 = 0x01,
}

impl From<u8> for Magic {
    fn from(magic: u8) -> Self {
        match magic {
            0x01 => Self::V1,
            _ => panic!("unknown magic: {}", magic),
        }
    }
}

impl From<Magic> for u8 {
    fn from(magic: Magic) -> Self {
        match magic {
            Magic::V1 => 0x01,
        }
    }
}
