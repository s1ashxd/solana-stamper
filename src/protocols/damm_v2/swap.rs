use solana_sdk::pubkey::Pubkey;

use crate::protocols::common::{TOKEN_PROGRAM, WSOL_MINT};
use crate::protocols::common::TokenProgram;
use crate::spec::account::Acc;
use crate::spec::data::DataSpec;
use crate::spec::instruction::InstructionSpec;
use crate::spec::prefix::{ComputeBudgetSlots, PrefixOptions, TipTransferSlots};
use crate::spec::{MessageVersion, TemplateSpec};

use super::constants::{DAMM_V2_PROGRAM, DAMM_V2_SWAP_DISC};
use super::pda::{event_authority, pool_authority};

#[must_use]
pub fn swap_spec(payer: Pubkey, base_token_program: TokenProgram, base_is_a: bool) -> TemplateSpec {
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

    let ix = InstructionSpec::new(DAMM_V2_PROGRAM)
        .account(Acc::fixed(pool_authority()))
        .account(Acc::slot_w("pool"))
        .account(Acc::slot_w("user_input"))
        .account(Acc::slot_w("user_output"))
        .account(Acc::slot_w("token_a_vault"))
        .account(Acc::slot_w("token_b_vault"))
        .account(if base_is_a {
            Acc::slot("mint")
        } else {
            Acc::fixed(WSOL_MINT)
        })
        .account(if base_is_a {
            Acc::fixed(WSOL_MINT)
        } else {
            Acc::slot("mint")
        })
        .account(Acc::payer())
        .account(if base_is_a {
            Acc::fixed(base_tp)
        } else {
            Acc::fixed(TOKEN_PROGRAM)
        })
        .account(if base_is_a {
            Acc::fixed(TOKEN_PROGRAM)
        } else {
            Acc::fixed(base_tp)
        })
        .account(Acc::fixed(DAMM_V2_PROGRAM))
        .account(Acc::fixed(event_authority()))
        .account(Acc::fixed(DAMM_V2_PROGRAM))
        .data(
            DataSpec::disc(&DAMM_V2_SWAP_DISC)
                .u64_slot("amount_in")
                .u64_slot("min_amount_out"),
        );

    TemplateSpec::new(payer, MessageVersion::V0)
        .prefix(prefix)
        .ix(ix)
}
