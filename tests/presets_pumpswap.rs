#![cfg(feature = "pumpswap")]

use solana_sdk::pubkey::Pubkey;
use tx_stamper::protocols::common::TokenProgram;
use tx_stamper::protocols::pumpswap::buy_v2_spec;
use tx_stamper::template::Template;

#[test]
fn pumpswap_buy_v2_spec_compiles() {
    let payer = Pubkey::new_unique();
    let spec = buy_v2_spec(payer, TokenProgram::Legacy);
    let tpl = Template::compile(spec).unwrap();
    let names: Vec<&str> = tpl.slot_names().collect();
    for expected in [
        "base_mint",
        "pool",
        "pool_base_ta",
        "pool_quote_ta",
        "fee_recipient",
        "fee_recipient_ata",
        "coin_creator_vault_ata",
        "creator_vault_authority",
        "user_vol",
        "pool_v2",
        "quote_amount_in",
        "min_base_amount_out",
        "cu_limit",
        "cu_price",
        "tip_account",
        "tip_lamports",
    ] {
        assert!(names.contains(&expected), "missing slot {expected}, got: {names:?}");
    }
}
