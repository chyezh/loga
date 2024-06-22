use super::Error;

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

impl TryFrom<u8> for Magic {
    type Error = super::Error;

    fn try_from(value: u8) -> super::Result<Self> {
        match value {
            0x01 => Ok(Self::V1),
            _ => Err(Error::InvalidMagic),
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

macro_rules! copy_slice_with_multi_stage {
    ($src:expr, $dst:expr, $stage_offset:expr, $dst_offset:expr) => {
        if $dst_offset == $dst.len() {
            return $dst_offset;
        } else if $stage_offset < $src.len() {
            let tmp_n = copy_slice(&$src[$stage_offset..], &mut $dst[$dst_offset..]);
            $dst_offset += tmp_n;
            if $dst_offset == $dst.len() {
                return $dst_offset;
            } else {
                $stage_offset = $stage_offset + tmp_n - $src.len();
            }
        } else {
            $stage_offset -= $src.len();
        }
    };
    () => {};
}
pub(super) use copy_slice_with_multi_stage;

macro_rules! customize_copy_slice_with_multi_stage {
    ($custom_copy:expr, $src_len:expr, $dst:expr, $stage_offset:expr, $dst_offset:expr) => {
        if $dst_offset == $dst.len() {
            return $dst_offset;
        } else if $stage_offset < $src_len {
            let tmp_n = $custom_copy;
            $dst_offset += tmp_n;
            if $dst_offset == $dst.len() {
                return $dst_offset;
            } else {
                $stage_offset = $stage_offset + tmp_n - $src_len;
            }
        } else {
            $stage_offset -= $src_len;
        }
    };
    () => {};
}
pub(super) use customize_copy_slice_with_multi_stage;
