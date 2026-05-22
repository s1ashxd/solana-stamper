use tx_stamper::error::StamperError;

#[test]
fn stamper_error_variants_exist() {
    let e = StamperError::DuplicateSlotName { name: "x".into() };
    assert!(format!("{e}").contains("duplicate"));
    let e = StamperError::TransactionTooLarge { size: 1300 };
    assert!(format!("{e}").contains("1300"));
}
