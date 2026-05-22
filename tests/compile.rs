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
