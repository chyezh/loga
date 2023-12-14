/// The `Attr` is used to identify the type of the entry.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Attr(i32);

/// The `Magic` is used to identify the version of the entry.
/// For backward compatibility, we need to keep the old version.
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Magic {
    V1 = 0x01,
}
