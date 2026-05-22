use solana_sdk::pubkey::Pubkey;

pub trait Signer {
    fn pubkey(&self) -> Pubkey;

    fn sign(&self, message: &[u8]) -> [u8; 64];
}
