use solana_sdk::pubkey::Pubkey;

use crate::protocols::common::{TOKEN_PROGRAM, WSOL_MINT, TokenProgram};
use crate::spec::account::Acc;
use crate::spec::data::DataSpec;
use crate::spec::instruction::InstructionSpec;
use crate::spec::lookup::{AddressSource, LookupTableSpec};
use crate::spec::prefix::{ComputeBudgetSlots, PrefixOptions, TipTransferSlots};
use crate::spec::{MessageVersion, TemplateSpec};

use super::constants::{DAMM_V2_PROGRAM, DBC_PROGRAM, PRINTR_ALT_KEYS, PRINTR_ALT_TABLE, PRINTR_PROGRAM, PRINTR_SWAP_DISC, SWAP_INTENT_SPEND_QUOTE};
use super::pda::{damm_event_authority, dbc_event_authority, printr_authority};

#[must_use]
pub fn buy_spec(payer: Pubkey, base_token_program: TokenProgram) -> TemplateSpec {
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

    let mut prefix_bytes = [0u8; 9];
    prefix_bytes[..8].copy_from_slice(&PRINTR_SWAP_DISC);
    prefix_bytes[8] = SWAP_INTENT_SPEND_QUOTE;

    let data = DataSpec::bytes(&prefix_bytes)
        .u64_slot("amount")
        .u128_slot("sqrt_price");

    let ix = InstructionSpec::new(PRINTR_PROGRAM)
        .account(Acc::payer())
        .account(Acc::fixed(printr_authority()).writable())
        .account(Acc::slot_w("input_wallet"))
        .account(Acc::slot_w("output_wallet"))
        .account(Acc::slot_w("mint"))
        .account(Acc::fixed(WSOL_MINT).writable())
        .account(Acc::slot_w("dbc_pool_authority"))
        .account(Acc::slot("dbc_config"))
        .account(Acc::slot_w("dbc_pool"))
        .account(Acc::slot_w("dbc_telecoin_vault"))
        .account(Acc::slot_w("dbc_quote_vault"))
        .account(Acc::fixed(dbc_event_authority()))
        .account(Acc::slot_w("dbc_migration_metadata"))
        .account(Acc::slot("damm_partner_config"))
        .account(Acc::slot_w("damm_pool"))
        .account(Acc::slot("damm_pool_authority"))
        .account(Acc::slot_w("damm_telecoin_vault"))
        .account(Acc::slot_w("damm_quote_vault"))
        .account(Acc::slot_w("damm_position_nft_mint"))
        .account(Acc::slot_w("damm_position_nft_account"))
        .account(Acc::slot_w("damm_position"))
        .account(Acc::fixed(damm_event_authority()))
        .account(Acc::fixed(base_tp))
        .account(Acc::fixed(TOKEN_PROGRAM))
        .account(Acc::fixed(DBC_PROGRAM))
        .account(Acc::fixed(DAMM_V2_PROGRAM))
        .account(Acc::fixed(solana_sdk::pubkey!("11111111111111111111111111111111")))
        .data(data);

    let lut = LookupTableSpec {
        address: AddressSource::Fixed(PRINTR_ALT_TABLE),
        keys: PRINTR_ALT_KEYS.iter().map(|pk| Acc::fixed(*pk)).collect(),
    };

    TemplateSpec::new(payer, MessageVersion::V0)
        .prefix(prefix)
        .ix(ix)
        .lut(lut)
}
