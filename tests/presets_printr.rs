#![cfg(feature = "printr")]

use solana_sdk::pubkey::Pubkey;
use tx_stamper::protocols::common::TokenProgram;
use tx_stamper::protocols::printr::buy_spec;
use tx_stamper::template::Template;

#[test]
fn printr_buy_spec_compiles_with_lut() {
    let payer = Pubkey::new_unique();
    let spec = buy_spec(payer, TokenProgram::Legacy);
    let tpl = Template::compile(spec).expect("compile failed");
    let names: Vec<&str> = tpl.slot_names().collect();
    for expected in [
        "printr_lut",
        "mint",
        "input_wallet",
        "output_wallet",
        "dbc_pool_authority",
        "dbc_config",
        "dbc_pool",
        "dbc_telecoin_vault",
        "dbc_quote_vault",
        "dbc_migration_metadata",
        "damm_partner_config",
        "damm_pool",
        "damm_pool_authority",
        "damm_telecoin_vault",
        "damm_quote_vault",
        "damm_position_nft_mint",
        "damm_position_nft_account",
        "damm_position",
        "amount",
        "sqrt_price",
        "cu_limit",
        "cu_price",
        "tip_account",
        "tip_lamports",
    ] {
        assert!(names.contains(&expected), "missing slot {expected}, got: {names:?}");
    }
}
