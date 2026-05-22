use tx_stamper::slot;
use tx_stamper::spec::slot::SlotKind;
use tx_stamper::spec::account::{Acc, AccountFlags};
use solana_sdk::pubkey::Pubkey;

#[test]
fn slot_macro_yields_static_str() {
    let s: &'static str = slot!("mint");
    assert_eq!(s, "mint");
}

#[test]
fn slot_kind_equality() {
    assert_eq!(SlotKind::Pubkey, SlotKind::Pubkey);
    assert_ne!(SlotKind::Pubkey, SlotKind::U64);
}

#[test]
fn acc_constructors() {
    let pk = Pubkey::new_unique();
    assert!(matches!(Acc::fixed(pk), Acc::Fixed(_, _)));
    assert!(matches!(Acc::payer(), Acc::Payer(_)));
    assert!(matches!(Acc::slot("mint"), Acc::Slot { .. }));
    assert!(matches!(Acc::slot_w("mint"), Acc::Slot { .. }));
    let w = Acc::slot("mint").writable();
    if let Acc::Slot { flags, .. } = w {
        assert!(flags.writable);
    } else { panic!() }
}
