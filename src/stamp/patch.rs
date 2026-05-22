pub fn patch_pubkey(buf: &mut [u8], offset: usize, value: &[u8; 32]) {
    buf[offset..offset + 32].copy_from_slice(value);
}

pub fn patch_hash(buf: &mut [u8], offset: usize, value: &[u8; 32]) {
    buf[offset..offset + 32].copy_from_slice(value);
}

pub fn patch_u8(buf: &mut [u8], offset: usize, value: u8) {
    buf[offset] = value;
}

pub fn patch_u16(buf: &mut [u8], offset: usize, value: u16) {
    buf[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

pub fn patch_u32(buf: &mut [u8], offset: usize, value: u32) {
    buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub fn patch_u64(buf: &mut [u8], offset: usize, value: u64) {
    buf[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

pub fn patch_sig(buf: &mut [u8], offset: usize, value: &[u8; 64]) {
    buf[offset..offset + 64].copy_from_slice(value);
}
