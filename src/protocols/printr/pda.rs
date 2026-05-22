use std::sync::OnceLock;

use solana_sdk::pubkey::Pubkey;

use super::constants::{DAMM_V2_PROGRAM, DBC_PROGRAM, PRINTR_PROGRAM};

pub fn printr_authority() -> Pubkey {
    static CACHE: OnceLock<Pubkey> = OnceLock::new();
    *CACHE.get_or_init(|| Pubkey::find_program_address(&[b"printr_authority"], &PRINTR_PROGRAM).0)
}

pub fn dbc_event_authority() -> Pubkey {
    static CACHE: OnceLock<Pubkey> = OnceLock::new();
    *CACHE.get_or_init(|| Pubkey::find_program_address(&[b"__event_authority"], &DBC_PROGRAM).0)
}

pub fn damm_event_authority() -> Pubkey {
    static CACHE: OnceLock<Pubkey> = OnceLock::new();
    *CACHE.get_or_init(|| Pubkey::find_program_address(&[b"__event_authority"], &DAMM_V2_PROGRAM).0)
}
