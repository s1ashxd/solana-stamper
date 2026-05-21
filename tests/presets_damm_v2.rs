#![cfg(feature = "damm-v2")]

use solana_sdk::pubkey::Pubkey;
use tx_stamper::protocols::common::TokenProgram;
use tx_stamper::protocols::damm_v2::swap_spec;
use tx_stamper::template::Template;

#[test]
fn damm_v2_swap_spec_compiles_base_is_a() {
    let payer = Pubkey::new_unique();
    let spec = swap_spec(payer, TokenProgram::Legacy, true);
    let tpl = Template::compile(spec).unwrap();
    let names: Vec<&str> = tpl.slot_names().collect();
    for expected in [
        "pool",
        "token_a_vault",
        "token_b_vault",
        "mint",
        "amount_in",
        "min_amount_out",
        "cu_limit",
        "cu_price",
        "tip_account",
        "tip_lamports",
    ] {
        assert!(names.contains(&expected), "missing slot {expected}, got: {names:?}");
    }
}

#[test]
fn damm_v2_swap_spec_compiles_base_is_b() {
    let payer = Pubkey::new_unique();
    let spec = swap_spec(payer, TokenProgram::Legacy, false);
    let tpl = Template::compile(spec).unwrap();
    let names: Vec<&str> = tpl.slot_names().collect();
    assert!(names.contains(&"mint"));
}
