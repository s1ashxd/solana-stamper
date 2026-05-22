use tx_stamper::compile::markers::MarkerAllocator;

#[test]
fn allocator_yields_unique_pubkey_sentinels() {
    let mut a = MarkerAllocator::new();
    let s1 = a.pubkey_sentinel();
    let s2 = a.pubkey_sentinel();
    assert_ne!(s1, s2);
    assert_eq!(s1[0], 0xC0);
    assert_eq!(s2[0], 0xC0);
}

#[test]
fn allocator_yields_unique_u64_magics() {
    let mut a = MarkerAllocator::new();
    let m1 = a.u64_magic();
    let m2 = a.u64_magic();
    assert_ne!(m1, m2);
}
