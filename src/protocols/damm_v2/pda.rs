use std::sync::OnceLock;

use solana_sdk::pubkey::Pubkey;

use crate::protocols::damm_v2::constants::DAMM_V2_PROGRAM;

pub fn pool_authority() -> Pubkey {
    static CACHE: OnceLock<Pubkey> = OnceLock::new();
    *CACHE.get_or_init(|| Pubkey::find_program_address(&[b"pool_authority"], &DAMM_V2_PROGRAM).0)
}

pub fn event_authority() -> Pubkey {
    static CACHE: OnceLock<Pubkey> = OnceLock::new();
    *CACHE.get_or_init(|| Pubkey::find_program_address(&[b"__event_authority"], &DAMM_V2_PROGRAM).0)
}
