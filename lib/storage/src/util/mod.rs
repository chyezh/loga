pub fn copy_slice(src: &[u8], dst: &mut [u8]) -> usize {
    let n = std::cmp::min(src.len(), dst.len());
    dst[..n].copy_from_slice(&src[..n]);
    n
}
