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
    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(
        InstructionSpec::new(program)
            .account(Acc::slot("mint"))
            .account(Acc::slot("mint")),
    );
    let err = validate_spec(&spec).unwrap_err();
    assert!(matches!(err, StamperError::DuplicateSlotName { name } if name == "mint"));
}

#[test]
fn validate_missing_dep_rejected() {
    let payer = Pubkey::new_unique();
    let program = Pubkey::new_unique();
    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(
        InstructionSpec::new(program)
            .account(Acc::derived("ata", &["mint"], |_, _| Ok(Pubkey::new_unique()))),
    );
    let err = validate_spec(&spec).unwrap_err();
    assert!(matches!(err, StamperError::MissingDependency { .. }));
}

#[test]
fn validate_cycle_rejected() {
    let payer = Pubkey::new_unique();
    let program = Pubkey::new_unique();
    let spec = TemplateSpec::new(payer, MessageVersion::V0).ix(
        InstructionSpec::new(program)
            .account(Acc::derived("a", &["b"], |_, _| Ok(Pubkey::new_unique())))
            .account(Acc::derived("b", &["a"], |_, _| Ok(Pubkey::new_unique()))),
    );
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
    let ix = Instruction::new_with_bytes(Pubkey::new_unique(), &huge_data, vec![AccountMeta::new_readonly(payer, true)]);
    let blockhash = Hash::new_from_array([0xB0; 32]);
    let err = serialize_placeholder_tx(&payer, blockhash, &[ix], &[], 1).unwrap_err();
    assert!(matches!(err, tx_stamper::error::StamperError::TransactionTooLarge { .. }));
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
