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

use solana_sdk::pubkey::Pubkey;
use tx_stamper::compile::validate::validate_spec;
use tx_stamper::error::StamperError;
use tx_stamper::spec::account::Acc;
use tx_stamper::spec::data::DataSpec;
use tx_stamper::spec::instruction::InstructionSpec;
use tx_stamper::spec::{MessageVersion, TemplateSpec};

#[test]
fn validate_duplicate_slot_rejected() {
    let payer = Pubkey::new_unique();
    let program = Pubkey::new_unique();
    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(InstructionSpec::new(program)
        .account(Acc::slot("mint"))
        .account(Acc::slot("mint")));
    let err = validate_spec(&spec).unwrap_err();
    assert!(matches!(err, StamperError::DuplicateSlotName { name } if name == "mint"));
}

#[test]
fn validate_missing_dep_rejected() {
    let payer = Pubkey::new_unique();
    let program = Pubkey::new_unique();
    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(InstructionSpec::new(program)
        .account(Acc::derived("ata", &["mint"], |_, _| {
            Ok(Pubkey::new_unique())
        })));
    let err = validate_spec(&spec).unwrap_err();
    assert!(matches!(err, StamperError::MissingDependency { .. }));
}

#[test]
fn validate_cycle_rejected() {
    let payer = Pubkey::new_unique();
    let program = Pubkey::new_unique();
    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(InstructionSpec::new(program)
        .account(Acc::derived("a", &["b"], |_, _| Ok(Pubkey::new_unique())))
        .account(Acc::derived("b", &["a"], |_, _| Ok(Pubkey::new_unique()))));
    let err = validate_spec(&spec).unwrap_err();
    assert!(matches!(err, StamperError::CyclicComputed { .. }));
}

use tx_stamper::compile::depgraph::topo_sort;

#[test]
fn topo_sort_chain() {
    let mut graph = std::collections::BTreeMap::new();
    graph.insert("c", vec!["b"]);
    graph.insert("b", vec!["a"]);
    graph.insert("a", vec![]);
    let sorted = topo_sort(&graph);
    assert_eq!(sorted, vec!["a", "b", "c"]);
}

#[test]
fn topo_sort_diamond() {
    let mut graph = std::collections::BTreeMap::new();
    graph.insert("d", vec!["b", "c"]);
    graph.insert("b", vec!["a"]);
    graph.insert("c", vec!["a"]);
    graph.insert("a", vec![]);
    let sorted = topo_sort(&graph);
    let pos_a = sorted.iter().position(|s| s == "a").unwrap();
    let pos_b = sorted.iter().position(|s| s == "b").unwrap();
    let pos_c = sorted.iter().position(|s| s == "c").unwrap();
    let pos_d = sorted.iter().position(|s| s == "d").unwrap();
    assert!(pos_a < pos_b);
    assert!(pos_a < pos_c);
    assert!(pos_b < pos_d);
    assert!(pos_c < pos_d);
}

use tx_stamper::compile::resolve::{ResolveContext, resolve_account};

#[test]
fn resolve_payer_returns_spec_payer() {
    let payer = Pubkey::new_unique();
    let mut ctx = ResolveContext::new(payer);
    let mut alloc = MarkerAllocator::new();
    let pk = resolve_account(&Acc::payer(), &mut ctx, &mut alloc).unwrap();
    assert_eq!(pk, payer);
}

#[test]
fn resolve_slot_uses_fresh_sentinel() {
    let payer = Pubkey::new_unique();
    let mut ctx = ResolveContext::new(payer);
    let mut alloc = MarkerAllocator::new();
    let a1 = resolve_account(&Acc::slot("mint"), &mut ctx, &mut alloc).unwrap();
    let a2 = resolve_account(&Acc::slot("bonding"), &mut ctx, &mut alloc).unwrap();
    assert_ne!(a1, a2);
    assert_eq!(a1.to_bytes()[0], 0xC0);
    assert_eq!(a2.to_bytes()[0], 0xC0);
}

#[test]
fn resolve_slot_same_name_returns_same_sentinel() {
    let payer = Pubkey::new_unique();
    let mut ctx = ResolveContext::new(payer);
    let mut alloc = MarkerAllocator::new();
    let a1 = resolve_account(&Acc::slot("mint"), &mut ctx, &mut alloc).unwrap();
    let a2 = resolve_account(&Acc::slot("mint"), &mut ctx, &mut alloc).unwrap();
    assert_eq!(a1, a2);
}

use solana_sdk::hash::Hash;
use solana_sdk::instruction::{AccountMeta, Instruction};
use tx_stamper::compile::serialize::serialize_placeholder_tx;

#[test]
fn serialize_simple_transfer_fits_in_max_tx_size() {
    let payer = Pubkey::new_unique();
    let recipient = Pubkey::new_unique();
    let ix = solana_system_interface::instruction::transfer(&payer, &recipient, 1_000);
    let blockhash = Hash::new_from_array([0xB0; 32]);
    let bytes = serialize_placeholder_tx(&payer, blockhash, &[ix], &[], 1).unwrap();
    assert!(bytes.len() <= tx_stamper::compile::MAX_TX_SIZE);
    assert_eq!(bytes[0], 1);
    assert_eq!(&bytes[1..65], &[0xAA; 64]);
}

#[test]
fn serialize_oversize_returns_error() {
    let payer = Pubkey::new_unique();
    let huge_data = vec![0u8; 2000];
    let ix = Instruction::new_with_bytes(
        Pubkey::new_unique(),
        &huge_data,
        vec![AccountMeta::new_readonly(payer, true)],
    );
    let blockhash = Hash::new_from_array([0xB0; 32]);
    let err = serialize_placeholder_tx(&payer, blockhash, &[ix], &[], 1).unwrap_err();
    assert!(matches!(
        err,
        tx_stamper::error::StamperError::TransactionTooLarge { .. }
    ));
}

use tx_stamper::compile::scan::{find_all, find_unique};

#[test]
fn find_unique_returns_offset() {
    let haystack = b"\x00\x01\x02\xC0\xC0\xC0\xC0\x05";
    let offset = find_unique(haystack, &[0xC0, 0xC0, 0xC0, 0xC0]).unwrap();
    assert_eq!(offset, 3);
}

#[test]
fn find_all_returns_all_offsets() {
    let haystack = b"\x01\x02\x01\x02\x03\x01\x02";
    let offs = find_all(haystack, &[0x01, 0x02]);
    assert_eq!(offs, vec![0, 2, 5]);
}

#[test]
fn find_unique_errors_on_multiple_matches() {
    let haystack = b"\xC0\xC0\xC0\xC0";
    assert!(find_unique(haystack, &[0xC0, 0xC0]).is_err());
}

use smallvec::smallvec;
use tx_stamper::spec::slot::SlotKind;
use tx_stamper::template::{PatchKind, PatchOp, PatchSlot};

#[test]
fn patch_op_size_correct() {
    let op = PatchOp {
        offset: 100,
        kind: PatchKind::Pubkey32,
    };
    assert_eq!(op.len(), 32);
    let op = PatchOp {
        offset: 100,
        kind: PatchKind::U64,
    };
    assert_eq!(op.len(), 8);
}

#[test]
fn patch_slot_aggregates_patches() {
    let slot = PatchSlot {
        kind: SlotKind::Pubkey,
        patches: smallvec![
            PatchOp {
                offset: 64,
                kind: PatchKind::Pubkey32
            },
            PatchOp {
                offset: 128,
                kind: PatchKind::Pubkey32
            },
        ],
        per_provider: false,
    };
    assert_eq!(slot.patches.len(), 2);
}

#[test]
fn compile_simple_transfer_spec() {
    let payer = Pubkey::new_unique();
    let mut data_prefix = [0u8; 4];
    data_prefix.copy_from_slice(&2u32.to_le_bytes());

    let spec =
        TemplateSpec::new(payer, MessageVersion::V0).ix(InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&data_prefix).u64_slot("amount")));
    let tpl = tx_stamper::template::Template::compile(spec).unwrap();
    assert!(tpl.slot_names().any(|n| n == "recipient"));
    assert!(tpl.slot_names().any(|n| n == "amount"));
}

use tx_stamper::spec::prefix::{
    AuthoritySource, ComputeBudgetSlots, NonceConfig, PrefixOptions, TipTransferSlots,
};

#[test]
fn compile_with_compute_budget_prefix() {
    let payer = Pubkey::new_unique();
    let spec = TemplateSpec::new(payer, MessageVersion::V0)
        .prefix(
            PrefixOptions::default().with_compute_budget(ComputeBudgetSlots {
                limit_slot: "cu_limit",
                price_slot: "cu_price",
            }),
        )
        .ix(InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")));
    let tpl = tx_stamper::template::Template::compile(spec).unwrap();
    assert!(tpl.slot_names().any(|n| n == "cu_limit"));
    assert!(tpl.slot_names().any(|n| n == "cu_price"));
}

#[test]
fn compile_with_tip_transfer_prefix() {
    let payer = Pubkey::new_unique();
    let spec = TemplateSpec::new(payer, MessageVersion::V0)
        .prefix(
            PrefixOptions::default().with_tip_transfer(TipTransferSlots {
                account_slot: "tip_account",
                lamports_slot: "tip_lamports",
                per_provider: true,
            }),
        )
        .ix(InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")));
    let tpl = tx_stamper::template::Template::compile(spec).unwrap();
    assert!(tpl.slot_names().any(|n| n == "tip_account"));
}

#[test]
fn compile_with_nonce_prefix() {
    let payer = Pubkey::new_unique();
    let spec = TemplateSpec::new(payer, MessageVersion::V0)
        .prefix(PrefixOptions::default().with_advance_nonce(NonceConfig {
            account_slot: "nonce_account",
            authority: AuthoritySource::Payer,
        }))
        .ix(InstructionSpec::new(Pubkey::default())
            .account(Acc::payer())
            .account(Acc::slot_w("recipient"))
            .data(DataSpec::bytes(&[2, 0, 0, 0]).u64_slot("amount")));
    let tpl = tx_stamper::template::Template::compile(spec).unwrap();
    assert!(tpl.slot_names().any(|n| n == "nonce_account"));
}

#[test]
fn u8_sentinel_collision_detected() {
    let payer = Pubkey::new_unique();
    let program = Pubkey::new_unique();
    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(InstructionSpec::new(program)
        .account(Acc::payer())
        .data(DataSpec::bytes(&[0xE0]).u8_slot("x")));
    let err = tx_stamper::template::Template::compile(spec).err().unwrap();
    assert!(
        matches!(err, tx_stamper::error::StamperError::MarkerCollision { .. })
            || matches!(
                err,
                tx_stamper::error::StamperError::SentinelNotFound { .. }
            )
    );
}

#[test]
fn fixed_pubkey_matching_sentinel_is_rejected() {
    let payer = Pubkey::new_unique();
    let mut bad_bytes = [0xC0u8; 32];
    bad_bytes[31] = 0;
    let bad_program = Pubkey::new_from_array(bad_bytes);
    let spec = tx_stamper::spec::TemplateSpec::new(payer, tx_stamper::spec::MessageVersion::V0).ix(
        tx_stamper::spec::instruction::InstructionSpec::new(bad_program)
            .account(Acc::payer())
            .account(Acc::slot_w("x"))
            .data(tx_stamper::spec::data::DataSpec::bytes(&[1])),
    );
    let err = tx_stamper::template::Template::compile(spec).err().unwrap();
    assert!(matches!(
        err,
        tx_stamper::error::StamperError::FixedPubkeyHitsSentinel { .. }
    ));
}

#[test]
fn compile_resolves_lut_with_fixed_address_and_fixed_keys() {
    use solana_sdk::pubkey::Pubkey;
    use tx_stamper::spec::account::Acc;
    use tx_stamper::spec::instruction::InstructionSpec;
    use tx_stamper::spec::lookup::{AddressSource, LookupTableSpec};
    use tx_stamper::spec::{MessageVersion, TemplateSpec};
    use tx_stamper::template::Template;

    let payer = Pubkey::new_unique();
    let static_addr_in_lut = Pubkey::new_unique();
    let lut_address = Pubkey::new_unique();
    let program = Pubkey::new_unique();

    let spec = TemplateSpec::new(payer, MessageVersion::V0)
        .ix(
            InstructionSpec::new(program)
                .account(Acc::fixed(static_addr_in_lut))
                .data(tx_stamper::spec::data::DataSpec::bytes(&[1, 2, 3])),
        )
        .lut(LookupTableSpec {
            address: AddressSource::Fixed(lut_address),
            keys: vec![Acc::fixed(static_addr_in_lut)],
        });

    let tpl = Template::compile(spec).expect("compile failed");
    let _ = tpl.slot_names();
}

#[test]
fn compile_resolves_lut_with_slot_address() {
    use solana_sdk::pubkey::Pubkey;
    use tx_stamper::spec::account::Acc;
    use tx_stamper::spec::instruction::InstructionSpec;
    use tx_stamper::spec::lookup::{AddressSource, LookupTableSpec};
    use tx_stamper::spec::{MessageVersion, TemplateSpec};
    use tx_stamper::template::Template;

    let payer = Pubkey::new_unique();
    let cached_addr = Pubkey::new_unique();
    let program = Pubkey::new_unique();

    let spec = TemplateSpec::new(payer, MessageVersion::V0)
        .ix(
            InstructionSpec::new(program)
                .account(Acc::fixed(cached_addr))
                .data(tx_stamper::spec::data::DataSpec::bytes(&[7, 7])),
        )
        .lut(LookupTableSpec {
            address: AddressSource::Slot("my_lut"),
            keys: vec![Acc::fixed(cached_addr)],
        });

    let tpl = Template::compile(spec).expect("compile failed");
    let names: Vec<&str> = tpl.slot_names().collect();
    assert!(names.contains(&"my_lut"), "lut slot not registered, got: {names:?}");
}
