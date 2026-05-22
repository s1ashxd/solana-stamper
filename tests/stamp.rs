use tx_stamper::stamp::patch::{patch_pubkey, patch_u64};

#[test]
fn patch_pubkey_writes_32_bytes() {
    let mut buf = [0u8; 64];
    let value = [0xAB; 32];
    patch_pubkey(&mut buf, 16, &value);
    for i in 16..48 {
        assert_eq!(buf[i], 0xAB);
    }
    assert_eq!(buf[15], 0);
    assert_eq!(buf[48], 0);
}

#[test]
fn patch_u64_writes_8_le_bytes() {
    let mut buf = [0u8; 16];
    patch_u64(&mut buf, 4, 0xDEADBEEF_CAFEBABE);
    assert_eq!(&buf[4..12], &0xDEADBEEF_CAFEBABE_u64.to_le_bytes());
}
