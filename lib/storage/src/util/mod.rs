pub fn copy_slice(src: &[u8], dst: &mut [u8]) -> usize {
    let n = std::cmp::min(src.len(), dst.len());
    dst[..n].copy_from_slice(&src[..n]);
    n
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
