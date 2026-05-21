use solana_sdk::pubkey::Pubkey;

use crate::protocols::common::TokenProgram;
use crate::protocols::pda;
use crate::spec::account::Acc;
use crate::spec::data::DataSpec;
use crate::spec::instruction::InstructionSpec;
use crate::spec::prefix::{ComputeBudgetSlots, PrefixOptions, TipTransferSlots};
use crate::spec::{MessageVersion, TemplateSpec};

use super::constants::{
    PUMPFUN_EVENT_AUTHORITY, PUMPFUN_FEE_CONFIG, PUMPFUN_FEE_PROGRAM, PUMPFUN_FEE_RECIPIENT,
    PUMPFUN_GLOBAL, PUMPFUN_MAYHEM_FEE_RECIPIENT, PUMPFUN_PROGRAM, SELL_DISC,
};

#[must_use]
pub fn sell_v2_spec(payer: Pubkey, token_program: TokenProgram) -> TemplateSpec {
    let tp = token_program.id();
    let fee_recipient = match token_program {
        TokenProgram::Legacy => PUMPFUN_FEE_RECIPIENT,
        TokenProgram::Token2022 => PUMPFUN_MAYHEM_FEE_RECIPIENT,
    };

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

    let ix = InstructionSpec::new(PUMPFUN_PROGRAM)
        .account(Acc::fixed(PUMPFUN_GLOBAL))
        .account(Acc::fixed(fee_recipient).writable())
        .account(Acc::slot("mint"))
        .account(Acc::slot_w("bonding_curve"))
        .account(Acc::slot_w("associated_bonding_curve"))
        .account(
            Acc::derived("user_ata", &["mint"], move |slots, payer| {
                let mint = slots.try_pubkey("mint")?;
                Ok(pda::ata(payer, &mint, &tp))
            })
            .writable(),
        )
        .account(Acc::payer())
        .account(Acc::fixed(solana_sdk::pubkey!("11111111111111111111111111111111")))
        .account(Acc::slot_w("creator_vault"))
        .account(Acc::fixed(token_program.id()))
        .account(Acc::fixed(PUMPFUN_EVENT_AUTHORITY))
        .account(Acc::fixed(PUMPFUN_PROGRAM))
        .account(Acc::fixed(PUMPFUN_FEE_CONFIG))
        .account(Acc::fixed(PUMPFUN_FEE_PROGRAM))
        .account(Acc::slot("bonding_curve_v2"))
        .data(
            DataSpec::disc(&SELL_DISC)
                .u64_slot("token_amount")
                .u64_slot("min_sol_out"),
        );

    TemplateSpec::new(payer, MessageVersion::V0)
        .prefix(prefix)
        .ix(ix)
}
