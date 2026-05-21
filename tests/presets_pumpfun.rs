#![cfg(feature = "pumpfun")]

use solana_sdk::pubkey::Pubkey;
use tx_stamper::protocols::common::TokenProgram;
use tx_stamper::protocols::pumpfun::buy_v2_spec;
use tx_stamper::template::Template;

#[test]
fn pumpfun_buy_v2_spec_compiles() {
    let payer = Pubkey::new_unique();
    let spec = buy_v2_spec(payer, TokenProgram::Legacy);
    let tpl = Template::compile(spec).unwrap();
    let names: Vec<&str> = tpl.slot_names().collect();
    for expected in [
        "mint",
        "bonding_curve",
        "associated_bonding_curve",
        "creator_vault",
        "bonding_curve_v2",
        "user_vol",
        "sol_amount",
        "min_tokens_out",
        "cu_limit",
        "cu_price",
        "tip_account",
        "tip_lamports",
    ] {
        assert!(names.contains(&expected), "missing slot {expected}, got: {names:?}");
    }
}
