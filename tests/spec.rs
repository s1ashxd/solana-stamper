use tx_stamper::slot;
use tx_stamper::spec::slot::SlotKind;
use tx_stamper::spec::account::{Acc, AccountFlags};
use tx_stamper::spec::data::{DataSpec, DataPiece};
use tx_stamper::spec::prefix::{ComputeBudgetSlots, NonceConfig, PrefixOptions, TipTransferSlots, AuthoritySource};
use tx_stamper::spec::lookup::{AddressSource, LookupTableSpec};
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

#[test]
fn data_spec_chained_builders() {
    let d = DataSpec::bytes(&[0x01, 0x02])
        .u64_slot("sol_amount")
        .u64_slot("min_tokens_out");
    assert_eq!(d.0.len(), 3);
    assert!(matches!(d.0[0], DataPiece::Bytes(_)));
    assert!(matches!(d.0[1], DataPiece::U64Slot(_)));
}

#[test]
fn data_spec_disc_alias() {
    let d = DataSpec::disc(&[1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(d.0.len(), 1);
}

#[test]
fn prefix_defaults_to_none() {
    let p = PrefixOptions::default();
    assert!(p.advance_nonce.is_none());
    assert!(p.compute_budget.is_none());
    assert!(p.tip_transfer.is_none());
}

#[test]
fn prefix_with_compute_budget() {
    let p = PrefixOptions::default().with_compute_budget(ComputeBudgetSlots {
        limit_slot: "cu_limit",
        price_slot: "cu_price",
    });
    assert!(p.compute_budget.is_some());
}

#[test]
fn lookup_table_address_source() {
    let pk = solana_sdk::pubkey::Pubkey::new_unique();
    let _ = AddressSource::Fixed(pk);
    let _ = LookupTableSpec { address: AddressSource::Fixed(pk), keys: smallvec::SmallVec::new() };
}
