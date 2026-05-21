#[cfg(any(feature = "pumpfun", feature = "pumpswap", feature = "damm-v2", feature = "printr"))]
use solana_sdk::pubkey::Pubkey;

#[cfg(any(feature = "pumpfun", feature = "pumpswap", feature = "damm-v2", feature = "printr"))]
#[must_use]
pub fn ata(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address_with_program_id(owner, mint, token_program)
}
