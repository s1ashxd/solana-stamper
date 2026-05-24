use tx_stamper::slot;
use tx_stamper::spec::slot::SlotKind;
use tx_stamper::stamp::values::SlotValue;

#[test]
fn slot_value_from_impls() {
    let pk = solana_sdk::pubkey::Pubkey::new_unique();
    let v: SlotValue = pk.into();
    assert!(matches!(v, SlotValue::Pubkey(_)));
    let v: SlotValue = 100u64.into();
    assert!(matches!(v, SlotValue::U64(_)));
    let v: SlotValue = 50u32.into();
    assert!(matches!(v, SlotValue::U32(_)));
}
use solana_sdk::pubkey::Pubkey;
use tx_stamper::spec::account::Acc;
use tx_stamper::spec::data::{DataPiece, DataSpec};
use tx_stamper::spec::instruction::InstructionSpec;
use tx_stamper::spec::lookup::LookupTable;
use tx_stamper::spec::prefix::{ComputeBudgetSlots, PrefixOptions};
use tx_stamper::spec::{MessageVersion, TemplateSpec};

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
    } else {
        panic!()
    }
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
fn lookup_table_construct() {
    let pk = solana_sdk::pubkey::Pubkey::new_unique();
    let addr = solana_sdk::pubkey::Pubkey::new_unique();
    let _ = LookupTable {
        address: pk,
        addresses: vec![addr],
    };
}

#[test]
fn template_spec_builder_minimal() {
    let payer = solana_sdk::pubkey::Pubkey::new_unique();
    let program = solana_sdk::pubkey::Pubkey::new_unique();
    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(InstructionSpec::new(program)
        .account(Acc::payer())
        .data(DataSpec::bytes(&[0xAB])));
    assert_eq!(spec.ixs.len(), 1);
    assert_eq!(spec.payer, payer);
}

#[test]
fn static_addresses_collects_prefix_ix_and_fixed_accounts_dedup() {
    use solana_sdk::pubkey::Pubkey;
    use tx_stamper::spec::account::Acc;
    use tx_stamper::spec::data::DataSpec;
    use tx_stamper::spec::instruction::InstructionSpec;
    use tx_stamper::spec::prefix::{ComputeBudgetSlots, PrefixOptions, TipTransferSlots};
    use tx_stamper::spec::{MessageVersion, TemplateSpec};

    let payer = Pubkey::new_unique();
    let program_a = Pubkey::new_unique();
    let program_b = Pubkey::new_unique();
    let shared_fixed = Pubkey::new_unique();
    let only_in_b = Pubkey::new_unique();

    let spec = TemplateSpec::new(payer, MessageVersion::V0)
        .prefix(
            PrefixOptions::default()
                .with_compute_budget(ComputeBudgetSlots {
                    limit_slot: "cu_limit",
                    price_slot: "cu_price",
                })
                .with_tip_transfer(TipTransferSlots {
                    account_slot: "tip",
                    lamports_slot: "lam",
                    per_provider: true,
                }),
        )
        .ix(
            InstructionSpec::new(program_a)
                .account(Acc::fixed(shared_fixed))
                .account(Acc::payer())
                .account(Acc::slot("a_slot"))
                .data(DataSpec::bytes(&[1])),
        )
        .ix(
            InstructionSpec::new(program_b)
                .account(Acc::fixed(shared_fixed))
                .account(Acc::fixed(only_in_b))
                .data(DataSpec::bytes(&[2])),
        );

    let addrs = spec.static_addresses();
    assert!(addrs.contains(&solana_system_interface::program::ID), "system program from prefix tip");
    assert!(addrs.iter().any(|pk| pk.to_string() == "ComputeBudget111111111111111111111111111111"), "compute budget program from prefix");
    assert!(addrs.contains(&program_a), "ix program a");
    assert!(addrs.contains(&program_b), "ix program b");
    assert!(addrs.contains(&shared_fixed), "shared fixed account");
    assert!(addrs.contains(&only_in_b), "only_in_b fixed account");
    assert!(!addrs.contains(&payer), "payer must not appear");
    let shared_count = addrs.iter().filter(|pk| **pk == shared_fixed).count();
    assert_eq!(shared_count, 1, "dedup of shared_fixed");
    let sysprog_count = addrs.iter().filter(|pk| **pk == solana_system_interface::program::ID).count();
    assert_eq!(sysprog_count, 1, "dedup of system_program (no nonce, only tip_transfer)");
}

#[test]
fn addresses_missing_from_filters_existing() {
    use solana_sdk::pubkey::Pubkey;
    use tx_stamper::spec::account::Acc;
    use tx_stamper::spec::data::DataSpec;
    use tx_stamper::spec::instruction::InstructionSpec;
    use tx_stamper::spec::{MessageVersion, TemplateSpec};

    let payer = Pubkey::new_unique();
    let prog = Pubkey::new_unique();
    let a = Pubkey::new_unique();
    let b = Pubkey::new_unique();
    let c = Pubkey::new_unique();

    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(
        InstructionSpec::new(prog)
            .account(Acc::fixed(a))
            .account(Acc::fixed(b))
            .account(Acc::fixed(c))
            .data(DataSpec::bytes(&[0])),
    );

    let existing = vec![a, c];
    let missing = spec.addresses_missing_from(&existing);
    assert_eq!(missing.len(), 2);
    assert!(missing.contains(&prog));
    assert!(missing.contains(&b));
    assert!(!missing.contains(&a));
    assert!(!missing.contains(&c));
}
