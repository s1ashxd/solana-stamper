use solana_sdk::pubkey::Pubkey;

use crate::protocols::common::{TOKEN_PROGRAM, WSOL_MINT};
use crate::protocols::common::{ASSOCIATED_TOKEN_PROGRAM, TokenProgram};
use crate::protocols::pda;
use crate::spec::account::Acc;
use crate::spec::data::DataSpec;
use crate::spec::instruction::InstructionSpec;
use crate::spec::prefix::{ComputeBudgetSlots, PrefixOptions, TipTransferSlots};
use crate::spec::{MessageVersion, TemplateSpec};

use super::constants::{
    BUY_EXACT_QUOTE_IN_DISC, PUMPFUN_FEE_PROGRAM, PUMPSWAP_EVENT_AUTHORITY, PUMPSWAP_FEE_CONFIG,
    PUMPSWAP_GLOBAL_CONFIG, PUMPSWAP_GLOBAL_VOLUME_ACC, PUMPSWAP_PROGRAM,
};

#[must_use]
pub fn buy_v2_spec(payer: Pubkey, base_token_program: TokenProgram) -> TemplateSpec {
    let base_tp = base_token_program.id();

    let prefix = PrefixOptions::default()
        .with_compute_budget(ComputeBudgetSlots {
            limit_slot: "cu_limit",
            price_slot: "cu_price",
        })
        .with_tip_transfer(TipTransferSlots {
            account_slot: "tip_account",
            lamports_slot: "tip_lamports",
            per_provider: true,
        });

    let ix = InstructionSpec::new(PUMPSWAP_PROGRAM)
        .account(Acc::slot_w("pool"))
        .account(Acc::payer())
        .account(Acc::fixed(PUMPSWAP_GLOBAL_CONFIG))
        .account(Acc::slot("base_mint"))
        .account(Acc::fixed(WSOL_MINT))
        .account(
            Acc::derived("user_base_ata", &["base_mint"], move |slots, payer| {
                let base_mint = slots.try_pubkey("base_mint")?;
                Ok(pda::ata(payer, &base_mint, &base_tp))
            })
            .writable(),
        )
        .account(
            Acc::derived("user_quote_ata", &[], move |_slots, payer| {
                Ok(pda::ata(payer, &WSOL_MINT, &TOKEN_PROGRAM))
            })
            .writable(),
        )
        .account(Acc::slot_w("pool_base_ta"))
        .account(Acc::slot_w("pool_quote_ta"))
        .account(Acc::slot("fee_recipient"))
        .account(Acc::slot_w("fee_recipient_ata"))
        .account(Acc::fixed(base_tp))
        .account(Acc::fixed(TOKEN_PROGRAM))
        .account(Acc::fixed(solana_sdk::pubkey!("11111111111111111111111111111111")))
        .account(Acc::fixed(ASSOCIATED_TOKEN_PROGRAM))
        .account(Acc::fixed(PUMPSWAP_EVENT_AUTHORITY))
        .account(Acc::fixed(PUMPSWAP_PROGRAM))
        .account(Acc::slot_w("coin_creator_vault_ata"))
        .account(Acc::slot("creator_vault_authority"))
        .account(Acc::fixed(PUMPSWAP_GLOBAL_VOLUME_ACC).writable())
        .account(Acc::slot_w("user_vol"))
        .account(Acc::fixed(PUMPSWAP_FEE_CONFIG))
        .account(Acc::fixed(PUMPFUN_FEE_PROGRAM))
        .account(Acc::slot("pool_v2"))
        .data(
            DataSpec::disc(&BUY_EXACT_QUOTE_IN_DISC)
                .u64_slot("quote_amount_in")
                .u64_slot("min_base_amount_out"),
        );

    TemplateSpec::new(payer, MessageVersion::V0)
        .prefix(prefix)
        .ix(ix)
}
